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

namespace tuffite::shell_api::remoting {
namespace {

namespace api = ::tuffite::remoting;
namespace wire = generated;

rpc::Status ToRpcStatus(api::Status status) {
  switch (status) {
    case api::Status::kOk:
      return rpc::Status::kOk;
    case api::Status::kInvalidArgument:
      return rpc::Status::kInvalidArgument;
    case api::Status::kAccessDenied:
      return rpc::Status::kAccessDenied;
    case api::Status::kInvalidHandle:
    case api::Status::kUnavailable:
      return rpc::Status::kUnavailable;
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
  return manifest &&
         manifest->IsShellApiCommandAllowed(frame.GetLastCommittedOrigin(),
                                            method) &&
         IsUnscopedAllowed(context, method);
}

wire::RemotingCapabilitiesPtr ToWire(const api::Capabilities& value) {
  return wire::RemotingCapabilities::New(
      value.available, value.protocol, value.video_codecs, value.input,
      value.clipboard, value.reconnect, value.snapshot);
}

wire::RemotingSignalPtr ToWire(const api::Signal& value) {
  auto result = wire::RemotingSignal::New();
  result->sequence = static_cast<int64_t>(value.sequence);
  if (value.description) {
    result->description = wire::RemotingSessionDescription::New(
        value.description->type, value.description->sdp);
  }
  result->candidates.reserve(value.candidates.size());
  for (const auto& candidate : value.candidates) {
    result->candidates.push_back(wire::RemotingIceCandidate::New(
        candidate.mid, static_cast<double>(candidate.m_line_index),
        candidate.candidate));
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

void RemotingImpl::Capabilities(
    const dispatch::Context& context,
    generated::Remoting::CapabilitiesRequestPtr,
    generated::Remoting::CapabilitiesCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.Capabilities")) {
    std::move(callback).Run(rpc::Status::kOk,
                            wire::RemotingCapabilities::New());
    return;
  }
  const auto result = ServiceFor(context)->GetCapabilities();
  std::move(callback).Run(ToRpcStatus(result.status), ToWire(result.value));
}

void RemotingImpl::Start(const dispatch::Context& context,
                         generated::Remoting::StartRequestPtr request,
                         generated::Remoting::StartCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.Start")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  api::StartOptions options{.auth_key = std::move(request->options->auth_key)};
  options.max_bitrate_kbps = request->options->max_bitrate_kbps
                                 ? Integer(*request->options->max_bitrate_kbps)
                                 : std::nullopt;
  options.target_framerate = request->options->target_framerate
                                 ? Integer(*request->options->target_framerate)
                                 : std::nullopt;
  options.screen_id = request->options->screen_id;
  if ((request->options->max_bitrate_kbps && !options.max_bitrate_kbps) ||
      (request->options->target_framerate && !options.target_framerate)) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  options.ice_servers.reserve(request->options->ice_servers.size());
  for (auto& server : request->options->ice_servers) {
    options.ice_servers.push_back(
        {.urls = std::move(server->urls),
         .username = server->username.value_or(""),
         .credential = server->credential.value_or("")});
  }
  ServiceFor(context)->Start(
      std::move(options), base::BindOnce(
                              [](generated::Remoting::StartCallback callback,
                                 api::SessionResult result) {
                                std::move(callback).Run(
                                    ToRpcStatus(result.status),
                                    wire::RemotingSession::New(
                                        static_cast<int64_t>(result.value.id),
                                        std::move(result.value.state)));
                              },
                              std::move(callback)));
}

void RemotingImpl::ProcessSignal(
    const dispatch::Context& context,
    generated::Remoting::ProcessSignalRequestPtr request,
    generated::Remoting::ProcessSignalCallback callback) {
  const auto id = SessionId(request->id);
  if (!IsAllowed(context, "tuffite.Remoting.ProcessSignal")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  if (!id || request->signal->sequence < 0) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  api::Signal signal{.sequence =
                         static_cast<uint64_t>(request->signal->sequence)};
  if (request->signal->description) {
    signal.description = api::SessionDescription{
        .type = std::move(request->signal->description->type),
        .sdp = std::move(request->signal->description->sdp)};
  }
  signal.candidates.reserve(request->signal->candidates.size());
  for (auto& candidate : request->signal->candidates) {
    const auto m_line_index = Integer(candidate->m_line_index);
    if (!m_line_index) {
      rpc::Reject(callback, rpc::Status::kInvalidArgument);
      return;
    }
    signal.candidates.push_back({.mid = std::move(candidate->mid),
                                 .m_line_index = *m_line_index,
                                 .candidate = std::move(candidate->candidate)});
  }
  ServiceFor(context)->ProcessSignal(
      *id, std::move(signal),
      base::BindOnce(
          [](generated::Remoting::ProcessSignalCallback callback,
             api::EmptyResult result) {
            std::move(callback).Run(ToRpcStatus(result.status));
          },
          std::move(callback)));
}

void RemotingImpl::TakeSignals(
    const dispatch::Context& context,
    generated::Remoting::TakeSignalsRequestPtr request,
    generated::Remoting::TakeSignalsCallback callback) {
  const auto id = SessionId(request->id);
  if (!IsAllowed(context, "tuffite.Remoting.TakeSignals")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  if (!id || (request->after_sequence && *request->after_sequence < 0)) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  ServiceFor(context)->TakeSignals(
      *id, static_cast<uint64_t>(request->after_sequence.value_or(0)),
      base::BindOnce(
          [](generated::Remoting::TakeSignalsCallback callback,
             api::SignalsResult result) {
            std::vector<wire::RemotingSignalPtr> signals;
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
std::optional<api::InputEvent> DecodeInput(wire::RemotingInputEventPtr input) {
  api::InputEvent event{
      .kind = std::move(input->kind),
      .pressed = input->pressed,
      .text = std::move(input->text),
      .button = std::move(input->button),
      .button_down = input->button_down,
      .clipboard_mime_type = std::move(input->clipboard_mime_type),
      .clipboard_data = std::move(input->clipboard_data),
  };
  if (input->usb_keycode) {
    const auto value = Integer(*input->usb_keycode);
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
  if (!assign_integer(input->x, &event.x) ||
      !assign_integer(input->y, &event.y)) {
    return std::nullopt;
  }
  event.wheel_delta_x = input->wheel_delta_x;
  event.wheel_delta_y = input->wheel_delta_y;
  event.wheel_ticks_x = input->wheel_ticks_x;
  event.wheel_ticks_y = input->wheel_ticks_y;
  return event;
}
}  // namespace

void RemotingImpl::SendInput(const dispatch::Context& context,
                             generated::Remoting::SendInputRequestPtr request,
                             generated::Remoting::SendInputCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.SendInput")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  const auto id = SessionId(request->id);
  auto event = DecodeInput(std::move(request->event));
  if (!id || !event) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  ServiceFor(context)->SendInput(
      *id, std::move(*event),
      base::BindOnce(
          [](generated::Remoting::SendInputCallback callback,
             api::EmptyResult result) {
            std::move(callback).Run(ToRpcStatus(result.status));
          },
          std::move(callback)));
}

void RemotingImpl::InjectInput(
    const dispatch::Context& context,
    generated::Remoting::InjectInputRequestPtr request,
    generated::Remoting::InjectInputCallback callback) {
  if (!context.is_application()) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  auto event = DecodeInput(std::move(request->event));
  if (!event) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  ServiceFor(context)->InjectInput(
      std::move(*event),
      base::BindOnce(
          [](generated::Remoting::InjectInputCallback callback,
             api::EmptyResult result) {
            std::move(callback).Run(ToRpcStatus(result.status));
          },
          std::move(callback)));
}

void RemotingImpl::Stop(const dispatch::Context& context,
                        generated::Remoting::StopRequestPtr request,
                        generated::Remoting::StopCallback callback) {
  const auto id = SessionId(request->id);
  if (!IsAllowed(context, "tuffite.Remoting.Stop")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  if (!id) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  ServiceFor(context)->Stop(
      *id, base::BindOnce(
               [](generated::Remoting::StopCallback callback,
                  api::EmptyResult result) {
                 std::move(callback).Run(ToRpcStatus(result.status));
               },
               std::move(callback)));
}

void RemotingImpl::Stats(const dispatch::Context& context,
                         generated::Remoting::StatsRequestPtr request,
                         generated::Remoting::StatsCallback callback) {
  const auto id = SessionId(request->id);
  if (!IsAllowed(context, "tuffite.Remoting.Stats")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  if (!id) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  ServiceFor(context)->GetStats(
      *id, base::BindOnce(
               [](generated::Remoting::StatsCallback callback,
                  api::StatsResult result) {
                 std::move(callback).Run(
                     ToRpcStatus(result.status),
                     wire::RemotingStats::New(
                         result.value.state, result.value.route,
                         result.value.transport, result.value.latency_ms,
                         result.value.jitter_ms,
                         result.value.available_bandwidth_kbps,
                         result.value.packet_loss_percent));
               },
               std::move(callback)));
}

void RemotingImpl::Capture(const dispatch::Context& context,
                           generated::Remoting::CaptureRequestPtr request,
                           generated::Remoting::CaptureCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.Capture")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  ServiceFor(context)->Capture(
      request->screen_id,
      base::BindOnce(
          [](generated::Remoting::CaptureCallback callback, api::Status status,
             api::Snapshot snapshot) {
            std::move(callback).Run(
                ToRpcStatus(status),
                wire::RemotingSnapshot::New(snapshot.width, snapshot.height,
                                            std::move(snapshot.mime_type),
                                            std::move(snapshot.data)));
          },
          std::move(callback)));
}

}  // namespace tuffite::shell_api::remoting
