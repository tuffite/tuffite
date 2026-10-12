// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/remoting/remoting_service_impl.h"

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <limits>
#include <map>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "base/check.h"
#include "base/containers/span.h"
#include "base/functional/bind.h"
#include "base/location.h"
#include "base/memory/ref_counted_delete_on_sequence.h"
#include "base/memory/weak_ptr.h"
#include "base/message_loop/message_pump_type.h"
#include "base/strings/string_number_conversions.h"
#include "base/task/bind_post_task.h"
#include "base/task/sequenced_task_runner.h"
#include "base/task/single_thread_task_runner_thread_mode.h"
#include "base/task/task_traits.h"
#include "base/task/thread_pool.h"
#include "base/threading/thread.h"
#include "base/time/time.h"
#include "build/build_config.h"
#include "crypto/hash.h"
#include "crypto/hmac.h"
#include "remoting/base/errors.h"
#include "remoting/base/session_options.h"
#include "remoting/base/source_location.h"
#include "remoting/host/input_injector.h"
#include "remoting/proto/event.pb.h"
#include "remoting/protocol/audio_sample_info.h"
#include "remoting/protocol/channel_dispatcher_base.h"
#include "remoting/protocol/desktop_capturer_proxy.h"
#include "remoting/protocol/desktop_capturer_wrapper.h"
#include "remoting/protocol/ice_config.h"
#include "remoting/protocol/ice_config_fetcher.h"
#include "remoting/protocol/input_event_tracker.h"
#include "remoting/protocol/input_stub.h"
#include "remoting/protocol/message_pipe.h"
#include "remoting/protocol/network_settings.h"
#include "remoting/protocol/sdp_message.h"
#include "remoting/protocol/transport.h"
#include "remoting/protocol/video_stream.h"
#include "remoting/protocol/webrtc_connection_to_client.h"
#include "remoting/protocol/webrtc_video_encoder_factory.h"
#include "remoting/signaling/jingle_data_structures.h"
#include "third_party/webrtc/api/candidate.h"
#include "third_party/webrtc/modules/desktop_capture/desktop_capture_options.h"
#include "third_party/webrtc/modules/desktop_capture/desktop_capture_types.h"
#include "third_party/webrtc/modules/desktop_capture/desktop_capturer.h"
#include "third_party/webrtc/modules/desktop_capture/desktop_frame.h"
#include "tuffite/framework/app/services/remoting/remoting_service.h"
#include "tuffite/framework/app/services/remoting/sequenced_input_dispatcher.h"
#include "ui/gfx/codec/png_codec.h"
#include "ui/gfx/geometry/size.h"

namespace tuffite::remoting {
namespace {

namespace api = ::tuffite::remoting;
namespace protocol = ::remoting::protocol;

constexpr size_t kMaxSessions = 4;
constexpr size_t kMaxSignalsPerSession = 256;
constexpr size_t kMaxAuthKeyBytes = 1024;
constexpr size_t kMaxSdpBytes = 1024 * 1024;
constexpr size_t kMaxCandidateBytes = 16 * 1024;
constexpr size_t kMaxClipboardBytes = 1024 * 1024;
constexpr int kDefaultTargetFramerate = 60;
constexpr std::string_view kPointerEventChannelName = "pointer-latest";
constexpr std::string_view kReliableEventChannelName = "input-sequenced";

class StaticIceConfigFetcher final : public protocol::IceConfigFetcher {
 public:
  explicit StaticIceConfigFetcher(protocol::IceConfig config)
      : config_(std::move(config)) {}

  void GetIceConfig(OnIceConfigCallback callback) override {
    std::move(callback).Run(config_);
  }

 private:
  protocol::IceConfig config_;
};

class DiscardClipboardStub final : public protocol::ClipboardStub {
 public:
  void InjectClipboardEvent(const protocol::ClipboardEvent&) override {}
};

std::optional<protocol::MouseEvent::MouseButton> MouseButton(
    std::string_view button) {
  if (button == "left") {
    return protocol::MouseEvent::BUTTON_LEFT;
  }
  if (button == "middle") {
    return protocol::MouseEvent::BUTTON_MIDDLE;
  }
  if (button == "right") {
    return protocol::MouseEvent::BUTTON_RIGHT;
  }
  if (button == "back") {
    return protocol::MouseEvent::BUTTON_BACK;
  }
  if (button == "forward") {
    return protocol::MouseEvent::BUTTON_FORWARD;
  }
  return std::nullopt;
}

bool InjectInputEvent(protocol::InputStub* input_stub,
                      protocol::ClipboardStub* clipboard_stub,
                      api::InputEvent event) {
  if (!input_stub || !clipboard_stub) {
    return false;
  }
  if (event.kind == "key" && event.pressed.has_value() &&
      event.usb_keycode.has_value()) {
    protocol::KeyEvent key;
    key.set_pressed(*event.pressed);
    key.set_usb_keycode(*event.usb_keycode);
    input_stub->InjectKeyEvent(key);
    return true;
  }
  if (event.kind == "text" && event.text && !event.text->empty() &&
      event.text->size() <= 4096) {
    protocol::TextEvent text;
    text.set_text(std::move(*event.text));
    input_stub->InjectTextEvent(text);
    return true;
  }
  if (event.kind == "mouse") {
    protocol::MouseEvent mouse;
    bool populated = false;
    if (event.x && event.y) {
      mouse.set_x(*event.x);
      mouse.set_y(*event.y);
      populated = true;
    }
    if (event.button || event.button_down) {
      if (!event.button || !event.button_down) {
        return false;
      }
      const auto button = MouseButton(*event.button);
      if (!button) {
        return false;
      }
      mouse.set_button(*button);
      mouse.set_button_down(*event.button_down);
      populated = true;
    }
    if (event.wheel_delta_x || event.wheel_delta_y || event.wheel_ticks_x ||
        event.wheel_ticks_y) {
      mouse.set_wheel_delta_x(
          static_cast<float>(event.wheel_delta_x.value_or(0)));
      mouse.set_wheel_delta_y(
          static_cast<float>(event.wheel_delta_y.value_or(0)));
      mouse.set_wheel_ticks_x(
          static_cast<float>(event.wheel_ticks_x.value_or(0)));
      mouse.set_wheel_ticks_y(
          static_cast<float>(event.wheel_ticks_y.value_or(0)));
      populated = true;
    }
    if (!populated) {
      return false;
    }
    input_stub->InjectMouseEvent(mouse);
    return true;
  }
  if (event.kind == "clipboard" && event.clipboard_mime_type &&
      event.clipboard_data &&
      event.clipboard_data->size() <= kMaxClipboardBytes) {
    protocol::ClipboardEvent clipboard;
    clipboard.set_mime_type(std::move(*event.clipboard_mime_type));
    clipboard.set_data(event.clipboard_data->data(),
                       event.clipboard_data->size());
    clipboard_stub->InjectClipboardEvent(clipboard);
    return true;
  }
  return false;
}

class OneShotDesktopCapture final
    : public webrtc::DesktopCapturer::Callback,
      public base::RefCountedDeleteOnSequence<OneShotDesktopCapture> {
 public:
  static void Start(std::optional<int64_t> screen_id,
                    scoped_refptr<base::SequencedTaskRunner> capture_runner,
                    api::SnapshotCallback callback) {
    auto capture = base::WrapRefCounted(new OneShotDesktopCapture(
        capture_runner, base::SequencedTaskRunner::GetCurrentDefault(),
        std::move(callback)));
    capture_runner->PostTask(
        FROM_HERE,
        base::BindOnce(&OneShotDesktopCapture::StartOnCaptureSequence, capture,
                       screen_id));
  }

  void OnCaptureResult(webrtc::DesktopCapturer::Result result,
                       std::unique_ptr<webrtc::DesktopFrame> frame) override {
    api::Status status = api::Status::kUnavailable;
    api::Snapshot snapshot;
    if (result == webrtc::DesktopCapturer::Result::SUCCESS && frame &&
        frame->data() && frame->size().width() > 0 &&
        frame->size().height() > 0) {
      auto encoded = gfx::PNGCodec::Encode(
          frame->data(), gfx::PNGCodec::FORMAT_BGRA,
          gfx::Size(frame->size().width(), frame->size().height()),
          frame->stride(), true, {});
      if (encoded) {
        status = api::Status::kOk;
        snapshot.width = static_cast<uint32_t>(frame->size().width());
        snapshot.height = static_cast<uint32_t>(frame->size().height());
        snapshot.mime_type = "image/png";
        snapshot.data = std::move(*encoded);
      }
    }
    origin_runner_->PostTask(
        FROM_HERE,
        base::BindOnce(std::move(callback_), status, std::move(snapshot)));
    owning_task_runner()->PostTask(
        FROM_HERE, base::BindOnce(&OneShotDesktopCapture::Finish,
                                  base::WrapRefCounted(this)));
    keep_alive_.reset();
  }

 private:
  friend class base::RefCountedThreadSafe<OneShotDesktopCapture>;
  friend class base::RefCountedDeleteOnSequence<OneShotDesktopCapture>;
  friend class base::DeleteHelper<OneShotDesktopCapture>;

  OneShotDesktopCapture(scoped_refptr<base::SequencedTaskRunner> capture_runner,
                        scoped_refptr<base::SequencedTaskRunner> origin_runner,
                        api::SnapshotCallback callback)
      : base::RefCountedDeleteOnSequence<OneShotDesktopCapture>(
            std::move(capture_runner)),
        origin_runner_(std::move(origin_runner)),
        callback_(std::move(callback)) {}

  ~OneShotDesktopCapture() override = default;

  void StartOnCaptureSequence(std::optional<int64_t> screen_id) {
    keep_alive_ = this;
    capturer_ = webrtc::DesktopCapturer::CreateScreenCapturer(
        webrtc::DesktopCaptureOptions::CreateDefault());
    const webrtc::ScreenId source =
        screen_id.value_or(static_cast<int64_t>(webrtc::kFullDesktopScreenId));
    if (!capturer_ || !capturer_->SelectSource(source)) {
      OnCaptureResult(webrtc::DesktopCapturer::Result::ERROR_PERMANENT,
                      nullptr);
      return;
    }
    capturer_->Start(this);
    capturer_->CaptureFrame();
  }

  void Finish() { capturer_.reset(); }

  scoped_refptr<base::SequencedTaskRunner> origin_runner_;
  api::SnapshotCallback callback_;
  std::unique_ptr<webrtc::DesktopCapturer> capturer_;
  scoped_refptr<OneShotDesktopCapture> keep_alive_;
};

std::optional<protocol::IceConfig> MakeIceConfig(
    const api::StartOptions& options) {
  protocol::IceConfig config;
  config.expiration_time = base::Time::Now() + base::Hours(24);
  config.max_bitrate_kbps = options.max_bitrate_kbps.value_or(0);
  for (const auto& server : options.ice_servers) {
    if (server.urls.empty()) {
      return std::nullopt;
    }
    for (const std::string& url : server.urls) {
      const bool ok =
          url.starts_with("stun:")
              ? config.AddStunServer(url)
              : config.AddServer(url, server.username, server.credential);
      if (!ok) {
        return std::nullopt;
      }
    }
  }
  return config;
}

uint32_t NetworkFlags(const api::StartOptions& options) {
  uint32_t flags = protocol::NetworkSettings::NAT_TRAVERSAL_OUTGOING;
  for (const auto& server : options.ice_servers) {
    for (const std::string& url : server.urls) {
      if (url.starts_with("stun:")) {
        flags |= protocol::NetworkSettings::NAT_TRAVERSAL_STUN;
      } else if (url.starts_with("turn:") || url.starts_with("turns:")) {
        flags |= protocol::NetworkSettings::NAT_TRAVERSAL_RELAY;
      }
    }
  }
  return flags;
}

std::vector<uint8_t> SignSdp(std::string_view auth_key,
                             std::string_view type,
                             std::string_view sdp) {
  protocol::SdpMessage normalized{std::string(sdp)};
  crypto::hmac::HmacSigner signer(crypto::hash::kSha256,
                                  base::as_byte_span(auth_key));
  signer.Update(base::as_byte_span(type));
  signer.Update(base::byte_span_from_cstring(" "));
  signer.Update(base::as_byte_span(normalized.NormalizedForSignature()));
  std::array<uint8_t, crypto::hash::kSha256Size> digest;
  signer.Finish(digest);
  return std::vector<uint8_t>(digest.begin(), digest.end());
}

std::optional<::remoting::SessionDescription::Type> DescriptionType(
    std::string_view type) {
  if (type == "offer") {
    return ::remoting::SessionDescription::Type::kOffer;
  }
  if (type == "answer") {
    return ::remoting::SessionDescription::Type::kAnswer;
  }
  return std::nullopt;
}

std::string DescriptionType(::remoting::SessionDescription::Type type) {
  switch (type) {
    case ::remoting::SessionDescription::Type::kOffer:
      return "offer";
    case ::remoting::SessionDescription::Type::kAnswer:
      return "answer";
    case ::remoting::SessionDescription::Type::kUnspecified:
      return {};
  }
}

std::unique_ptr<::remoting::DesktopCapturer> CreateDesktopCapturer(
    webrtc::ScreenId screen_id,
    scoped_refptr<base::SequencedTaskRunner> capture_task_runner) {
  auto proxy =
      std::make_unique<::remoting::DesktopCapturerProxy>(capture_task_runner);
  auto options = webrtc::DesktopCaptureOptions::CreateDefault();
  options.set_detect_updated_region(true);
#if BUILDFLAG(IS_WIN)
  options.set_allow_directx_capturer(true);
#elif BUILDFLAG(IS_MAC)
  options.set_allow_sck_capturer(true);
#endif
  proxy->CreateCapturer(base::BindOnce(
      [](webrtc::DesktopCaptureOptions options, webrtc::ScreenId screen_id)
          -> std::unique_ptr<::remoting::DesktopCapturer> {
        auto capturer = webrtc::DesktopCapturer::CreateScreenCapturer(options);
        if (!capturer || !capturer->SelectSource(screen_id)) {
          return nullptr;
        }
        return std::make_unique<::remoting::DesktopCapturerWrapper>(
            std::move(capturer));
      },
      std::move(options), screen_id));
  return proxy;
}

class HostSession final : public protocol::ConnectionToClient::EventHandler,
                          public protocol::InputStub,
                          public protocol::ChannelDispatcherBase::EventHandler {
 public:
  HostSession(api::StartOptions options,
              protocol::IceConfig ice_config,
              scoped_refptr<base::SingleThreadTaskRunner> ui_task_runner)
      : options_(std::move(options)),
        ui_task_runner_(std::move(ui_task_runner)),
        capture_task_runner_(base::ThreadPool::CreateSingleThreadTaskRunner(
            {base::TaskPriority::USER_BLOCKING, base::MayBlock()},
            base::SingleThreadTaskRunnerThreadMode::DEDICATED)),
        input_task_runner_(base::ThreadPool::CreateSingleThreadTaskRunner(
            {base::TaskPriority::USER_BLOCKING, base::MayBlock()},
            base::SingleThreadTaskRunnerThreadMode::DEDICATED)),
        connection_(std::make_unique<protocol::WebrtcConnectionToClient>(
            std::make_unique<StaticIceConfigFetcher>(std::move(ice_config)))) {
    connection_->SetEventHandler(this);
  }

  ~HostSession() override { Stop(); }

  bool Start() {
    if (state_ != "starting") {
      return false;
    }
    input_injector_ =
        ::remoting::InputInjector::Create(input_task_runner_, ui_task_runner_);
    if (!input_injector_) {
      state_ = "failed";
      error_ = "input injector is unavailable";
      return false;
    }
    input_injector_->Start(std::make_unique<DiscardClipboardStub>());
    input_tracker_ = std::make_unique<protocol::InputEventTracker>(this);
    connection_->set_input_stub(input_tracker_.get());
    connection_->set_clipboard_stub(input_injector_.get());
    ::remoting::SessionOptions session_options;
    session_options.detect_updated_region = true;
    session_options.capture_video_on_dedicated_thread = true;
    session_options.vp9_encoder_speed = 9;
#if BUILDFLAG(IS_WIN)
    session_options.allow_dxgi_capturer = true;
#elif BUILDFLAG(IS_MAC)
    session_options.enable_sck_capturer = true;
#endif
    connection_->ApplySessionOptions(session_options);
    connection_->ApplyNetworkSettings(
        protocol::NetworkSettings(NetworkFlags(options_)));
    connection_->transport()->Start(
        options_.auth_key, base::BindRepeating(&HostSession::OnSignal,
                                               weak_factory_.GetWeakPtr()));
    connection_->Start();
    return true;
  }

  bool ProcessSignal(api::Signal signal) {
    if (!connection_ || state_ == "stopped" || state_ == "failed") {
      return false;
    }
    ::remoting::JingleTransportInfo info;
    if (signal.description) {
      if (signal.description->sdp.empty() ||
          signal.description->sdp.size() > kMaxSdpBytes) {
        return false;
      }
      const auto type = DescriptionType(signal.description->type);
      if (!type) {
        return false;
      }
      ::remoting::SessionDescription description;
      description.type = *type;
      description.sdp = std::move(signal.description->sdp);
      description.signature =
          SignSdp(options_.auth_key, DescriptionType(description.type),
                  description.sdp);
      info.session_description = std::move(description);
    }
    for (auto& candidate : signal.candidates) {
      if (candidate.mid.empty() || candidate.candidate.empty() ||
          candidate.candidate.size() > kMaxCandidateBytes ||
          candidate.m_line_index < 0) {
        return false;
      }
      auto parsed =
          webrtc::Candidate::ParseCandidateString(candidate.candidate);
      if (!parsed.ok()) {
        return false;
      }
      info.candidates.emplace_back(candidate.mid, parsed.value(),
                                   candidate.m_line_index);
    }
    return connection_->transport()->ProcessTransportInfo(info);
  }

  std::vector<api::Signal> TakeSignals(uint64_t after_sequence) const {
    std::vector<api::Signal> result;
    for (const auto& signal : signals_) {
      if (signal.sequence > after_sequence) {
        result.push_back(signal);
      }
    }
    return result;
  }

  bool SendInput(api::InputEvent event) {
    if (!input_tracker_ || state_ == "stopped" || state_ == "failed") {
      return false;
    }
    const bool injected = InjectInputEvent(
        input_tracker_.get(), input_injector_.get(), std::move(event));
    return injected;
  }

  void Stop() {
    if (state_ == "stopped") {
      return;
    }
    state_ = "stopped";
    weak_factory_.InvalidateWeakPtrs();
    if (input_tracker_) {
      input_tracker_->ReleaseAll();
    }
    video_stream_.reset();
    pointer_event_dispatcher_.reset();
    reliable_event_dispatcher_.reset();
    if (connection_) {
      connection_->Disconnect(::remoting::ErrorCode::OK, "session stopped",
                              FROM_HERE);
    }
    connection_.reset();
    input_tracker_.reset();
    input_injector_.reset();
    signals_.clear();
  }

  api::Stats stats() const {
    return {.state = state_, .route = route_, .transport = transport_};
  }

 private:
  void BoostVideo() {
    if (video_stream_) {
      const auto now = base::TimeTicks::Now();
      const int fps = std::clamp(
          options_.target_framerate.value_or(kDefaultTargetFramerate), 15, 120);
      const auto interval = base::Microseconds(1000000 / fps);
      if (now - last_input_boost_ < interval) {
        return;
      }
      last_input_boost_ = now;
      video_stream_->BoostFramerate(interval, base::Milliseconds(100));
    }
  }

  void OnSignal(std::unique_ptr<::remoting::JingleTransportInfo> info) {
    if (!info) {
      return;
    }
    api::Signal signal;
    signal.sequence = next_signal_sequence_++;
    if (info->session_description) {
      const std::string type = DescriptionType(info->session_description->type);
      if (!type.empty()) {
        signal.description = api::SessionDescription{
            .type = type, .sdp = info->session_description->sdp};
      }
    }
    signal.candidates.reserve(info->candidates.size());
    for (const auto& candidate : info->candidates) {
      if (!candidate.sdp_m_line_index) {
        continue;
      }
      signal.candidates.push_back(
          {.mid = candidate.name,
           .m_line_index = *candidate.sdp_m_line_index,
           .candidate = candidate.candidate.ToCandidateAttribute(true)});
    }
    if (signals_.size() == kMaxSignalsPerSession) {
      signals_.erase(signals_.begin());
    }
    signals_.push_back(std::move(signal));
  }

  // protocol::ConnectionToClient::EventHandler:
  void CreateMediaStreams() override {
    if (video_stream_) {
      return;
    }
    const webrtc::ScreenId screen_id = options_.screen_id.value_or(
        static_cast<int64_t>(webrtc::kFullDesktopScreenId));
    auto capturer = CreateDesktopCapturer(screen_id, capture_task_runner_);
    if (!capturer) {
      state_ = "failed";
      error_ = "desktop capturer is unavailable";
      return;
    }
    video_stream_ =
        connection_->StartVideoStream(screen_id, std::move(capturer));
    video_stream_->SetTargetFramerate(std::clamp(
        options_.target_framerate.value_or(kDefaultTargetFramerate), 15, 120));
    state_ = "connecting";
  }

  void OnConnectionChannelsConnected() override { state_ = "connected"; }

  // protocol::InputStub. WebRTC event-channel input bypasses the Link RPC
  // path, so forward it directly and wake capture immediately after input.
  void InjectKeyEvent(const protocol::KeyEvent& event) override {
    if (input_injector_) {
      input_injector_->InjectKeyEvent(event);
      BoostVideo();
    }
  }

  void InjectTextEvent(const protocol::TextEvent& event) override {
    if (input_injector_) {
      input_injector_->InjectTextEvent(event);
      BoostVideo();
    }
  }

  void InjectMouseEvent(const protocol::MouseEvent& event) override {
    if (input_injector_) {
      input_injector_->InjectMouseEvent(event);
      BoostVideo();
    }
  }

  void InjectTouchEvent(const protocol::TouchEvent& event) override {
    if (input_injector_) {
      input_injector_->InjectTouchEvent(event);
      BoostVideo();
    }
  }

  void OnTransportProtocolChange(const std::string& protocol_name) override {
    transport_ = protocol_name;
  }

  void OnRouteChange(const std::string&,
                     const protocol::TransportRoute& route) override {
    route_ = protocol::TransportRoute::GetTypeString(route.type);
  }

  void OnIncomingDataChannel(
      const std::string& channel_name,
      std::unique_ptr<protocol::MessagePipe> pipe) override {
    auto* dispatcher = channel_name == kPointerEventChannelName
                           ? &pointer_event_dispatcher_
                       : channel_name == kReliableEventChannelName
                           ? &reliable_event_dispatcher_
                           : nullptr;
    if (!dispatcher || *dispatcher || !input_tracker_) {
      return;
    }
    *dispatcher = std::make_unique<api::SequencedInputDispatcher>(
        channel_name == kPointerEventChannelName, &input_sequence_,
        input_tracker_.get());
    (*dispatcher)->Init(std::move(pipe), this);
  }

  void OnChannelInitialized(protocol::ChannelDispatcherBase*) override {}

  void OnChannelClosed(protocol::ChannelDispatcherBase*) override {}

  void OnIncomingAudioFormatChanged(
      const protocol::AudioSampleInfo&,
      base::OnceCallback<void(bool)> done) override {
    std::move(done).Run(false);
  }

  void OnConnectionClosed(::remoting::ErrorCode error,
                          std::string_view details,
                          const ::remoting::SourceLocation&) override {
    if (state_ != "stopped") {
      if (input_tracker_) {
        input_tracker_->ReleaseAll();
      }
      pointer_event_dispatcher_.reset();
      reliable_event_dispatcher_.reset();
      video_stream_.reset();
      input_tracker_.reset();
      input_injector_.reset();
      state_ = error == ::remoting::ErrorCode::OK ? "stopped" : "failed";
      error_ = std::string(details);
    }
  }

  api::StartOptions options_;
  scoped_refptr<base::SingleThreadTaskRunner> ui_task_runner_;
  std::string state_ = "starting";
  std::string route_ = "unknown";
  std::string transport_ = "unknown";
  std::string error_;
  uint64_t next_signal_sequence_ = 1;
  std::vector<api::Signal> signals_;
  scoped_refptr<base::SequencedTaskRunner> capture_task_runner_;
  scoped_refptr<base::SingleThreadTaskRunner> input_task_runner_;
  std::unique_ptr<::remoting::InputInjector> input_injector_;
  std::unique_ptr<protocol::InputEventTracker> input_tracker_;
  std::unique_ptr<protocol::WebrtcConnectionToClient> connection_;
  std::unique_ptr<protocol::VideoStream> video_stream_;
  api::InputSequence input_sequence_;
  base::TimeTicks last_input_boost_;
  std::unique_ptr<api::SequencedInputDispatcher> pointer_event_dispatcher_;
  std::unique_ptr<api::SequencedInputDispatcher> reliable_event_dispatcher_;
  base::WeakPtrFactory<HostSession> weak_factory_{this};
};

class RemotingServiceImpl final {
 public:
  explicit RemotingServiceImpl(
      scoped_refptr<base::SingleThreadTaskRunner> ui_task_runner)
      : ui_task_runner_(std::move(ui_task_runner)),
        input_task_runner_(base::ThreadPool::CreateSingleThreadTaskRunner(
            {base::TaskPriority::USER_BLOCKING, base::MayBlock()},
            base::SingleThreadTaskRunnerThreadMode::DEDICATED)),
        capture_task_runner_(base::ThreadPool::CreateSingleThreadTaskRunner(
            {base::TaskPriority::USER_BLOCKING, base::MayBlock()},
            base::SingleThreadTaskRunnerThreadMode::DEDICATED)) {}

  static api::CapabilitiesResult GetCapabilities() {
    protocol::WebrtcVideoEncoderFactory encoder_factory;
    std::vector<std::string> codecs;
    for (const auto& format : encoder_factory.GetSupportedFormats()) {
      if (std::find(codecs.begin(), codecs.end(), format.name) ==
          codecs.end()) {
        codecs.push_back(format.name);
      }
    }
    return {.status = api::Status::kOk,
            .value = {.available = true,
                      .protocol = "crd-webrtc",
                      .video_codecs = std::move(codecs),
                      .input = true,
                      .clipboard = true,
                      .reconnect = true,
                      .snapshot = true}};
  }

  api::SessionResult Start(api::StartOptions options) {
    // Reclaim completed sessions before enforcing capacity, including clients
    // that vanished without ever delivering Stop.
    std::erase_if(sessions_, [](const auto& entry) {
      const auto state = entry.second->stats().state;
      return state == "stopped" || state == "failed";
    });
    if (options.auth_key.empty() ||
        options.auth_key.size() > kMaxAuthKeyBytes ||
        sessions_.size() >= kMaxSessions ||
        (options.max_bitrate_kbps && (*options.max_bitrate_kbps < 128 ||
                                      *options.max_bitrate_kbps > 100000)) ||
        (options.target_framerate &&
         (*options.target_framerate < 15 || *options.target_framerate > 120))) {
      return {.status = api::Status::kInvalidArgument};
    }
    auto ice_config = MakeIceConfig(options);
    if (!ice_config || next_session_id_ == 0) {
      return {.status = api::Status::kInvalidArgument};
    }
    const uint64_t id = next_session_id_++;
    auto session = std::make_unique<HostSession>(
        std::move(options), std::move(*ice_config), ui_task_runner_);
    if (!session->Start()) {
      return {.status = api::Status::kUnavailable};
    }
    api::SessionInfo info{.id = id, .state = "connecting"};
    sessions_.emplace(id, std::move(session));
    return {.status = api::Status::kOk, .value = std::move(info)};
  }

  api::EmptyResult ProcessSignal(uint64_t session_id, api::Signal signal) {
    HostSession* session = Find(session_id);
    if (!session) {
      return {.status = api::Status::kInvalidHandle};
    }
    return {.status = session->ProcessSignal(std::move(signal))
                          ? api::Status::kOk
                          : api::Status::kInvalidArgument,
            .value = true};
  }

  api::SignalsResult TakeSignals(uint64_t session_id, uint64_t after_sequence) {
    HostSession* session = Find(session_id);
    if (!session) {
      return {.status = api::Status::kInvalidHandle};
    }
    return {.status = api::Status::kOk,
            .value = session->TakeSignals(after_sequence)};
  }

  api::EmptyResult SendInput(uint64_t session_id, api::InputEvent event) {
    HostSession* session = Find(session_id);
    if (!session) {
      return {.status = api::Status::kInvalidHandle};
    }
    return {.status = session->SendInput(std::move(event))
                          ? api::Status::kOk
                          : api::Status::kInvalidArgument,
            .value = true};
  }

  api::EmptyResult InjectInput(api::InputEvent event) {
    if (!direct_input_injector_) {
      direct_input_injector_ = ::remoting::InputInjector::Create(
          input_task_runner_, ui_task_runner_);
      if (!direct_input_injector_) {
        return {.status = api::Status::kUnavailable};
      }
      direct_input_injector_->Start(std::make_unique<DiscardClipboardStub>());
    }
    return {.status =
                InjectInputEvent(direct_input_injector_.get(),
                                 direct_input_injector_.get(), std::move(event))
                    ? api::Status::kOk
                    : api::Status::kInvalidArgument,
            .value = true};
  }

  void Capture(std::optional<int64_t> screen_id,
               api::SnapshotCallback callback) {
    OneShotDesktopCapture::Start(screen_id, capture_task_runner_,
                                 std::move(callback));
  }

  api::EmptyResult Stop(uint64_t session_id) {
    auto found = sessions_.find(session_id);
    if (found == sessions_.end()) {
      return {.status = api::Status::kOk, .value = true};
    }
    found->second->Stop();
    sessions_.erase(found);
    return {.status = api::Status::kOk, .value = true};
  }

  api::StatsResult GetStats(uint64_t session_id) const {
    const HostSession* session = Find(session_id);
    if (!session) {
      return {.status = api::Status::kInvalidHandle};
    }
    return {.status = api::Status::kOk, .value = session->stats()};
  }

  void Shutdown() {
    auto sessions = std::move(sessions_);
    sessions_.clear();
    for (auto& [id, session] : sessions) {
      session->Stop();
    }
    direct_input_injector_.reset();
  }

 private:
  HostSession* Find(uint64_t id) {
    const auto found = sessions_.find(id);
    return found == sessions_.end() ? nullptr : found->second.get();
  }

  const HostSession* Find(uint64_t id) const {
    const auto found = sessions_.find(id);
    return found == sessions_.end() ? nullptr : found->second.get();
  }

  scoped_refptr<base::SingleThreadTaskRunner> ui_task_runner_;
  uint64_t next_session_id_ = 1;
  std::map<uint64_t, std::unique_ptr<HostSession>> sessions_;
  scoped_refptr<base::SingleThreadTaskRunner> input_task_runner_;
  scoped_refptr<base::SequencedTaskRunner> capture_task_runner_;
  std::unique_ptr<::remoting::InputInjector> direct_input_injector_;
};

// CRD's sockets and WebRTC observers need an IO message pump. Keep the entire
// connection lifecycle on that sequence and reply asynchronously to the UI;
// neither blocking the UI nor pumping a reentrant nested UI loop is necessary.
class IoRemotingService final : public api::RemotingService {
 public:
  IoRemotingService()
      : ui_task_runner_(base::SingleThreadTaskRunner::GetCurrentDefault()),
        io_thread_("TuffiteRemotingIO") {
    base::Thread::Options options;
    options.message_pump_type = base::MessagePumpType::IO;
    CHECK(io_thread_.StartWithOptions(std::move(options)));
    Post([this] {
      impl_ = std::make_unique<RemotingServiceImpl>(ui_task_runner_);
    });
  }

  ~IoRemotingService() override {
    Post([this] {
      impl_->Shutdown();
      impl_.reset();
    });
    io_thread_.Stop();
  }

  api::CapabilitiesResult GetCapabilities() const override {
    // Codec discovery is pure metadata and does not access connection state.
    return RemotingServiceImpl::GetCapabilities();
  }

  void Start(api::StartOptions options,
             api::SessionCallback callback) override {
    Call(
        [options = std::move(options)](RemotingServiceImpl* impl) mutable {
          return impl->Start(std::move(options));
        },
        std::move(callback));
  }

  void ProcessSignal(uint64_t id,
                     api::Signal signal,
                     api::EmptyCallback callback) override {
    Call(
        [id, signal = std::move(signal)](RemotingServiceImpl* impl) mutable {
          return impl->ProcessSignal(id, std::move(signal));
        },
        std::move(callback));
  }

  void TakeSignals(uint64_t id,
                   uint64_t after,
                   api::SignalsCallback callback) override {
    Call(
        [id, after](RemotingServiceImpl* impl) {
          return impl->TakeSignals(id, after);
        },
        std::move(callback));
  }

  void SendInput(uint64_t id,
                 api::InputEvent event,
                 api::EmptyCallback callback) override {
    Call(
        [id, event = std::move(event)](RemotingServiceImpl* impl) mutable {
          return impl->SendInput(id, std::move(event));
        },
        std::move(callback));
  }

  void InjectInput(api::InputEvent event,
                   api::EmptyCallback callback) override {
    Call(
        [event = std::move(event)](RemotingServiceImpl* impl) mutable {
          return impl->InjectInput(std::move(event));
        },
        std::move(callback));
  }

  void Capture(std::optional<int64_t> screen_id,
               api::SnapshotCallback callback) override {
    auto reply = base::BindPostTask(ui_task_runner_, std::move(callback));
    Post([this, screen_id, reply = std::move(reply)]() mutable {
      impl_->Capture(screen_id, std::move(reply));
    });
  }

  void Stop(uint64_t id, api::EmptyCallback callback) override {
    Call([id](RemotingServiceImpl* impl) { return impl->Stop(id); },
         std::move(callback));
  }

  void GetStats(uint64_t id, api::StatsCallback callback) const override {
    Call([id](RemotingServiceImpl* impl) { return impl->GetStats(id); },
         std::move(callback));
  }

  void Shutdown() override {
    Post([this] { impl_->Shutdown(); });
  }

 private:
  template <typename Function>
  void Post(Function function) const {
    CHECK(io_thread_.task_runner()->PostTask(
        FROM_HERE,
        base::BindOnce([](Function function) { std::move(function)(); },
                       std::move(function))));
  }

  template <typename Function, typename Callback>
  void Call(Function function, Callback callback) const {
    auto reply = base::BindPostTask(ui_task_runner_, std::move(callback));
    Post([this, function = std::move(function),
          reply = std::move(reply)]() mutable {
      std::move(reply).Run(std::move(function)(impl_.get()));
    });
  }

  scoped_refptr<base::SingleThreadTaskRunner> ui_task_runner_;
  base::Thread io_thread_;
  std::unique_ptr<RemotingServiceImpl> impl_;
};

}  // namespace

std::unique_ptr<::tuffite::remoting::RemotingService> CreateRemotingService() {
  return std::make_unique<IoRemotingService>();
}

}  // namespace tuffite::remoting
