// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/remoting.h"

#include <cmath>
#include <limits>
#include <optional>
#include <string_view>
#include <utility>
#include <vector>

#include "base/functional/bind.h"
#include "base/notreached.h"
#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/remoting/remoting_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"

namespace tuffite::shell_api::remoting {
namespace {

namespace api = ::tuffite::remoting;
namespace wire = generated;

rpc::Status ToRpcStatus(api::Status status) {
  switch (status) {
    case api::Status::kOk:
      return kStatusOk;
    case api::Status::kInvalidArgument:
      return kStatusInvalidArgument;
    case api::Status::kAccessDenied:
      return kStatusAccessDenied;
    case api::Status::kInvalidHandle:
    case api::Status::kUnavailable:
      return kStatusUnavailable;
  }
  NOTREACHED();
}

api::RemotingService* ServiceFor(const dispatch::Context& context) {
  if (context.is_application()) {
    return context.native_application->GetService<api::RemotingService>();
  }
  return BrowserContext::GetFrom(context.frame())
      .application_context()
      .GetService<api::RemotingService>();
}

bool IsAllowed(const dispatch::Context& context, std::string_view method) {
  if (context.is_application()) {
    return true;  // Trusted application control plane.
  }
  auto& frame = context.frame();
  const ManifestService* manifest = ManifestFor(frame);
  return manifest && manifest->IsShellApiCommandAllowed(
                         frame.GetLastCommittedOrigin(), method);
}

wire::RemotingCapabilities ToWire(const api::Capabilities& value) {
  return {.available = value.available,
          .protocol = value.protocol,
          .video_codecs = value.video_codecs,
          .input = value.input,
          .clipboard = value.clipboard,
          .reconnect = value.reconnect,
          .snapshot = value.snapshot};
}

wire::RemotingSignal ToWire(const api::Signal& value) {
  wire::RemotingSignal result{.sequence = static_cast<int64_t>(value.sequence)};
  if (value.description) {
    result.description = wire::RemotingSessionDescription{
        .type = value.description->type, .sdp = value.description->sdp};
  }
  result.candidates.reserve(value.candidates.size());
  for (const auto& candidate : value.candidates) {
    result.candidates.push_back(
        {.mid = candidate.mid,
         .m_line_index = static_cast<double>(candidate.m_line_index),
         .candidate = candidate.candidate});
  }
  return result;
}

std::optional<uint64_t> SessionId(int64_t id) {
  return id > 0 ? std::optional(static_cast<uint64_t>(id)) : std::nullopt;
}

std::optional<int> Integer(double value) {
  if (!std::isfinite(value) || std::trunc(value) != value ||
      value < std::numeric_limits<int>::min() ||
      value > std::numeric_limits<int>::max()) {
    return std::nullopt;
  }
  return static_cast<int>(value);
}

}  // namespace

void Capabilities(const ::tuffite::dispatch::Context& context,
                  generated::remoting::CapabilitiesRequest&&,
                  generated::remoting::CapabilitiesCallback&& callback) {
  if (!IsAllowed(context, "tuffite.remoting.capabilities")) {
    std::move(callback).Run(kStatusOk, {});
    return;
  }
  const auto result = ServiceFor(context)->GetCapabilities();
  std::move(callback).Run(ToRpcStatus(result.status), ToWire(result.value));
}

void Start(const ::tuffite::dispatch::Context& context,
           generated::remoting::StartRequest&& request,
           generated::remoting::StartCallback&& callback) {
  if (!IsAllowed(context, "tuffite.remoting.start")) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  api::StartOptions options{.auth_key = std::move(request.options.auth_key)};
  options.max_bitrate_kbps = request.options.max_bitrate_kbps
                                 ? Integer(*request.options.max_bitrate_kbps)
                                 : std::nullopt;
  options.target_framerate = request.options.target_framerate
                                 ? Integer(*request.options.target_framerate)
                                 : std::nullopt;
  options.screen_id = request.options.screen_id;
  if ((request.options.max_bitrate_kbps && !options.max_bitrate_kbps) ||
      (request.options.target_framerate && !options.target_framerate)) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  options.ice_servers.reserve(request.options.ice_servers.size());
  for (auto& server : request.options.ice_servers) {
    options.ice_servers.push_back(
        {.urls = std::move(server.urls),
         .username = server.username.value_or(""),
         .credential = server.credential.value_or("")});
  }
  ServiceFor(context)->Start(
      std::move(options),
      base::BindOnce(
          [](generated::remoting::StartCallback callback,
             api::SessionResult result) {
            std::move(callback).Run(
                ToRpcStatus(result.status),
                {.id = static_cast<int64_t>(result.value.id),
                 .state = std::move(result.value.state)});
          },
          std::move(callback)));
}

void ProcessSignal(const ::tuffite::dispatch::Context& context,
                   generated::remoting::ProcessSignalRequest&& request,
                   generated::remoting::ProcessSignalCallback&& callback) {
  const auto id = SessionId(request.id);
  if (!IsAllowed(context, "tuffite.remoting.processSignal")) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  if (!id || request.signal.sequence < 0) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  api::Signal signal{.sequence =
                         static_cast<uint64_t>(request.signal.sequence)};
  if (request.signal.description) {
    signal.description = api::SessionDescription{
        .type = std::move(request.signal.description->type),
        .sdp = std::move(request.signal.description->sdp)};
  }
  signal.candidates.reserve(request.signal.candidates.size());
  for (auto& candidate : request.signal.candidates) {
    const auto m_line_index = Integer(candidate.m_line_index);
    if (!m_line_index) {
      std::move(callback).Run(kStatusInvalidArgument, {});
      return;
    }
    signal.candidates.push_back({.mid = std::move(candidate.mid),
                                 .m_line_index = *m_line_index,
                                 .candidate = std::move(candidate.candidate)});
  }
  ServiceFor(context)->ProcessSignal(
      *id, std::move(signal),
      base::BindOnce(
          [](generated::remoting::ProcessSignalCallback callback,
             api::EmptyResult result) {
            std::move(callback).Run(ToRpcStatus(result.status), {});
          },
          std::move(callback)));
}

void TakeSignals(const ::tuffite::dispatch::Context& context,
                 generated::remoting::TakeSignalsRequest&& request,
                 generated::remoting::TakeSignalsCallback&& callback) {
  const auto id = SessionId(request.id);
  if (!IsAllowed(context, "tuffite.remoting.takeSignals")) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  if (!id || (request.after_sequence && *request.after_sequence < 0)) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  ServiceFor(context)->TakeSignals(
      *id, static_cast<uint64_t>(request.after_sequence.value_or(0)),
      base::BindOnce(
          [](generated::remoting::TakeSignalsCallback callback,
             api::SignalsResult result) {
            std::vector<wire::RemotingSignal> signals;
            signals.reserve(result.value.size());
            for (const auto& signal : result.value) {
              signals.push_back(ToWire(signal));
            }
            std::move(callback).Run(ToRpcStatus(result.status),
                                    std::move(signals));
          },
          std::move(callback)));
}

namespace {
std::optional<api::InputEvent> DecodeInput(wire::RemotingInputEvent input) {
  api::InputEvent event{
      .kind = std::move(input.kind),
      .pressed = input.pressed,
      .text = std::move(input.text),
      .button = std::move(input.button),
      .button_down = input.button_down,
      .clipboard_mime_type = std::move(input.clipboard_mime_type),
      .clipboard_data = std::move(input.clipboard_data),
  };
  if (input.usb_keycode) {
    const auto value = Integer(*input.usb_keycode);
    if (!value || *value < 0) {
      return std::nullopt;
    }
    event.usb_keycode = static_cast<uint32_t>(*value);
  }
  auto assign_integer = [](const std::optional<double>& source,
                           std::optional<int>* destination) {
    if (!source) {
      return true;
    }
    *destination = Integer(*source);
    return destination->has_value();
  };
  if (!assign_integer(input.x, &event.x) ||
      !assign_integer(input.y, &event.y)) {
    return std::nullopt;
  }
  event.wheel_delta_x = input.wheel_delta_x;
  event.wheel_delta_y = input.wheel_delta_y;
  event.wheel_ticks_x = input.wheel_ticks_x;
  event.wheel_ticks_y = input.wheel_ticks_y;
  return event;
}
}  // namespace

void SendInput(const dispatch::Context& context,
               generated::remoting::SendInputRequest&& request,
               generated::remoting::SendInputCallback&& callback) {
  if (!IsAllowed(context, "tuffite.remoting.sendInput")) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  const auto id = SessionId(request.id);
  auto event = DecodeInput(std::move(request.event));
  if (!id || !event) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  ServiceFor(context)->SendInput(
      *id, std::move(*event),
      base::BindOnce(
          [](generated::remoting::SendInputCallback callback,
             api::EmptyResult result) {
            std::move(callback).Run(ToRpcStatus(result.status), {});
          },
          std::move(callback)));
}

void InjectInput(const dispatch::Context& context,
                 generated::remoting::InjectInputRequest&& request,
                 generated::remoting::InjectInputCallback&& callback) {
  if (!context.is_application()) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  auto event = DecodeInput(std::move(request.event));
  if (!event) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  ServiceFor(context)->InjectInput(
      std::move(*event),
      base::BindOnce(
          [](generated::remoting::InjectInputCallback callback,
             api::EmptyResult result) {
            std::move(callback).Run(ToRpcStatus(result.status), {});
          },
          std::move(callback)));
}

void Stop(const ::tuffite::dispatch::Context& context,
          generated::remoting::StopRequest&& request,
          generated::remoting::StopCallback&& callback) {
  const auto id = SessionId(request.id);
  if (!IsAllowed(context, "tuffite.remoting.stop")) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  if (!id) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  ServiceFor(context)->Stop(
      *id, base::BindOnce(
               [](generated::remoting::StopCallback callback,
                  api::EmptyResult result) {
                 std::move(callback).Run(ToRpcStatus(result.status), {});
               },
               std::move(callback)));
}

void Stats(const ::tuffite::dispatch::Context& context,
           generated::remoting::StatsRequest&& request,
           generated::remoting::StatsCallback&& callback) {
  const auto id = SessionId(request.id);
  if (!IsAllowed(context, "tuffite.remoting.stats")) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  if (!id) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  ServiceFor(context)->GetStats(
      *id, base::BindOnce(
               [](generated::remoting::StatsCallback callback,
                  api::StatsResult result) {
                 std::move(callback).Run(
                     ToRpcStatus(result.status),
                     {.state = result.value.state,
                      .route = result.value.route,
                      .transport = result.value.transport,
                      .latency_ms = result.value.latency_ms,
                      .jitter_ms = result.value.jitter_ms,
                      .available_bandwidth_kbps =
                          result.value.available_bandwidth_kbps,
                      .packet_loss_percent = result.value.packet_loss_percent});
               },
               std::move(callback)));
}

void Capture(const ::tuffite::dispatch::Context& context,
             generated::remoting::CaptureRequest&& request,
             generated::remoting::CaptureCallback&& callback) {
  if (!IsAllowed(context, "tuffite.remoting.capture")) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  ServiceFor(context)->Capture(
      request.screen_id,
      base::BindOnce(
          [](generated::remoting::CaptureCallback callback, api::Status status,
             api::Snapshot snapshot) {
            std::move(callback).Run(
                ToRpcStatus(status),
                {.width = static_cast<double>(snapshot.width),
                 .height = static_cast<double>(snapshot.height),
                 .mime_type = std::move(snapshot.mime_type),
                 .data = std::move(snapshot.data)});
          },
          std::move(callback)));
}

}  // namespace tuffite::shell_api::remoting
