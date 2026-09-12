// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_COMMON_MESSAGE_H_
#define TUFFITE_COMPONENTS_DISPATCH_COMMON_MESSAGE_H_
#include <cstddef>
#include <cstdint>
#include <vector>
namespace tuffite::rpc {
inline constexpr size_t kMaxPayloadBytes = 16 * 1024 * 1024;
struct Request {
  uint64_t request_id = 0;
  uint64_t command_id = 0;
  uint64_t schema_fingerprint = 0;
  std::vector<uint8_t> payload;
};
struct Response {
  uint64_t request_id = 0;
  uint16_t status = 0;
  std::vector<uint8_t> payload;
};
struct Cancel {
  uint64_t request_id = 0;
};
}  // namespace tuffite::rpc

#endif  // TUFFITE_COMPONENTS_DISPATCH_COMMON_MESSAGE_H_
