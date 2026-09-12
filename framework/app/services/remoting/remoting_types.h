// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_TYPES_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_TYPES_H_

#include <cstdint>
#include <optional>
#include <string>
#include <vector>

#include "base/functional/callback.h"

namespace tuffite::remoting {

enum class Status {
  kOk,
  kInvalidArgument,
  kAccessDenied,
  kInvalidHandle,
  kUnavailable,
};

struct Capabilities {
  bool available = false;
  std::string protocol = "crd-webrtc";
  std::vector<std::string> video_codecs;
  bool input = false;
  bool clipboard = false;
  bool reconnect = false;
  bool snapshot = false;
};

struct Snapshot {
  uint32_t width = 0;
  uint32_t height = 0;
  std::string mime_type;
  std::vector<uint8_t> data;
};

struct IceServer {
  std::vector<std::string> urls;
  std::string username;
  std::string credential;
};

struct StartOptions {
  std::string auth_key;
  std::vector<IceServer> ice_servers;
  std::optional<int> max_bitrate_kbps;
  std::optional<int> target_framerate;
  std::optional<int64_t> screen_id;
};

struct SessionInfo {
  uint64_t id = 0;
  std::string state;
};

struct SessionDescription {
  std::string type;
  std::string sdp;
};

struct IceCandidate {
  std::string mid;
  int m_line_index = 0;
  std::string candidate;
};

struct Signal {
  uint64_t sequence = 0;
  std::optional<SessionDescription> description;
  std::vector<IceCandidate> candidates;
};

struct InputEvent {
  std::string kind;
  std::optional<bool> pressed;
  std::optional<uint32_t> usb_keycode;
  std::optional<std::string> text;
  std::optional<int> x;
  std::optional<int> y;
  std::optional<std::string> button;
  std::optional<bool> button_down;
  std::optional<double> wheel_delta_x;
  std::optional<double> wheel_delta_y;
  std::optional<double> wheel_ticks_x;
  std::optional<double> wheel_ticks_y;
  std::optional<std::string> clipboard_mime_type;
  std::optional<std::vector<uint8_t>> clipboard_data;
};

struct Stats {
  std::string state;
  std::string route;
  std::string transport;
  double latency_ms = 0;
  double jitter_ms = 0;
  double available_bandwidth_kbps = 0;
  double packet_loss_percent = 0;
};

template <typename Value>
struct Result {
  Status status = Status::kUnavailable;
  Value value;
};

using EmptyResult = Result<bool>;
using CapabilitiesResult = Result<Capabilities>;
using SessionResult = Result<SessionInfo>;
using SignalsResult = Result<std::vector<Signal>>;
using StatsResult = Result<Stats>;
using SnapshotCallback =
    base::OnceCallback<void(Status status, Snapshot snapshot)>;

}  // namespace tuffite::remoting

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_TYPES_H_
