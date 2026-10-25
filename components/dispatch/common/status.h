// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_COMMON_STATUS_H_
#define TUFFITE_COMPONENTS_DISPATCH_COMMON_STATUS_H_
#include <cstdint>
namespace tuffite::rpc {
enum class Status : int32_t {
  kOk = 0,
  kInvalidArgument = 1,
  kAccessDenied = 2,
  kUnavailable = 3,
  kCancelled = 4,
  kTimedOut = 5,
  kResourceExhausted = 6
};

// These are dispatch statuses, not tuffite_status_t ABI return values.
constexpr const char* ErrorCode(Status status) {
  switch (status) {
    case Status::kOk:
      return "ok";
    case Status::kInvalidArgument:
      return "invalid_argument";
    case Status::kAccessDenied:
      return "access_denied";
    case Status::kUnavailable:
      return "unavailable";
    case Status::kCancelled:
      return "cancelled";
    case Status::kTimedOut:
      return "timed_out";
    case Status::kResourceExhausted:
      return "resource_exhausted";
  }
  return "internal";
}
constexpr const char* ErrorMessage(Status status) {
  switch (status) {
    case Status::kOk:
      return "ShellAPI request succeeded";
    case Status::kInvalidArgument:
      return "Invalid ShellAPI request";
    case Status::kAccessDenied:
      return "ShellAPI access denied";
    case Status::kUnavailable:
      return "ShellAPI service unavailable";
    case Status::kCancelled:
      return "ShellAPI request cancelled";
    case Status::kTimedOut:
      return "ShellAPI request timed out";
    case Status::kResourceExhausted:
      return "ShellAPI resource limit exceeded";
  }
  return "Internal ShellAPI error";
}
}  // namespace tuffite::rpc

#endif  // TUFFITE_COMPONENTS_DISPATCH_COMMON_STATUS_H_
