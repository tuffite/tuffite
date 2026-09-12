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
}

#endif  // TUFFITE_COMPONENTS_DISPATCH_COMMON_STATUS_H_
