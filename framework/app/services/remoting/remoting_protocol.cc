// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/services/remoting/remoting_protocol.h"

#include <cmath>
#include <limits>
#include <optional>
#include <utility>
#include <vector>

#include "base/functional/bind.h"
#include "base/notreached.h"
namespace tuffite::remoting {
namespace {
namespace api = ::tuffite::remoting;
namespace wire = ::tuffite::shell_api::generated;
namespace generated = ::tuffite::shell_api::generated;
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
void Protocol::Capabilities(
    RemotingService& service,
    generated::Remoting::CapabilitiesRequestPtr,
    generated::Remoting::CapabilitiesCallback callback) {
  const auto result = service.GetCapabilities();
  std::move(callback).Run(ToRpcStatus(result.status), ToWire(result.value));
}

void Protocol::Start(RemotingService& service,
                     generated::Remoting::StartRequestPtr request,
                     generated::Remoting::StartCallback callback) {
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
  service.Start(std::move(options),
                base::BindOnce(
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

void Protocol::ProcessSignal(
    RemotingService& service,
    generated::Remoting::ProcessSignalRequestPtr request,
    generated::Remoting::ProcessSignalCallback callback) {
  const auto id = SessionId(request->id);
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
  service.ProcessSignal(
      *id, std::move(signal),
      base::BindOnce(
          [](generated::Remoting::ProcessSignalCallback callback,
             api::EmptyResult result) {
            std::move(callback).Run(ToRpcStatus(result.status));
          },
          std::move(callback)));
}

void Protocol::TakeSignals(RemotingService& service,
                           generated::Remoting::TakeSignalsRequestPtr request,
                           generated::Remoting::TakeSignalsCallback callback) {
  const auto id = SessionId(request->id);
  if (!id || (request->after_sequence && *request->after_sequence < 0)) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  service.TakeSignals(
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

void Protocol::SendInput(RemotingService& service,
                         generated::Remoting::SendInputRequestPtr request,
                         generated::Remoting::SendInputCallback callback) {
  const auto id = SessionId(request->id);
  auto event = DecodeInput(std::move(request->event));
  if (!id || !event) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  service.SendInput(*id, std::move(*event),
                    base::BindOnce(
                        [](generated::Remoting::SendInputCallback callback,
                           api::EmptyResult result) {
                          std::move(callback).Run(ToRpcStatus(result.status));
                        },
                        std::move(callback)));
}

void Protocol::InjectInput(RemotingService& service,
                           generated::Remoting::InjectInputRequestPtr request,
                           generated::Remoting::InjectInputCallback callback) {
  auto event = DecodeInput(std::move(request->event));
  if (!event) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  service.InjectInput(std::move(*event),
                      base::BindOnce(
                          [](generated::Remoting::InjectInputCallback callback,
                             EmptyResult result) {
                            std::move(callback).Run(ToRpcStatus(result.status));
                          },
                          std::move(callback)));
}

void Protocol::Stop(RemotingService& service,
                    generated::Remoting::StopRequestPtr request,
                    generated::Remoting::StopCallback callback) {
  const auto id = SessionId(request->id);
  if (!id) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  service.Stop(*id, base::BindOnce(
                        [](generated::Remoting::StopCallback callback,
                           api::EmptyResult result) {
                          std::move(callback).Run(ToRpcStatus(result.status));
                        },
                        std::move(callback)));
}

void Protocol::Stats(RemotingService& service,
                     generated::Remoting::StatsRequestPtr request,
                     generated::Remoting::StatsCallback callback) {
  const auto id = SessionId(request->id);
  if (!id) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  service.GetStats(
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

void Protocol::Capture(RemotingService& service,
                       generated::Remoting::CaptureRequestPtr request,
                       generated::Remoting::CaptureCallback callback) {
  service.Capture(
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

}  // namespace tuffite::remoting
