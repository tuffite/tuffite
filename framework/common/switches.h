// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_SWITCHES_H_
#define TUFFITE_FRAMEWORK_COMMON_SWITCHES_H_

#include "tuffite/components/manifest/bootstrap/process_configuration.h"

namespace tuffite {
inline constexpr char kDevUrlSwitch[] = "tuffite-dev-url";
inline constexpr char kDevTokenSwitch[] = "tuffite-dev-token";
inline constexpr char kSmokeTestSwitch[] = "tuffite-smoke-test";
inline constexpr char kWindowSurfaceTestSwitch[] =
    "tuffite-window-surface-test";

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_COMMON_SWITCHES_H_
