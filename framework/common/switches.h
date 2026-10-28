// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_SWITCHES_H_
#define TUFFITE_FRAMEWORK_COMMON_SWITCHES_H_

#include <string>
#include <vector>

#include "base/command_line.h"
#include "base/strings/string_split.h"
namespace tuffite {

inline constexpr char kAppSchemesSwitch[] = "tuffite-app-schemes";
inline std::vector<std::string> ApplicationSchemes() {
  return base::SplitString(
      base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
          kAppSchemesSwitch),
      ",", base::TRIM_WHITESPACE, base::SPLIT_WANT_NONEMPTY);
}
inline constexpr char kShellApiOriginsSwitch[] = "tuffite-shell-api-origins";
inline constexpr char kExplorerOriginsSwitch[] = "tuffite-explorer-origins";
inline constexpr char kDevUrlSwitch[] = "tuffite-dev-url";
inline constexpr char kDevTokenSwitch[] = "tuffite-dev-token";
inline constexpr char kSmokeTestSwitch[] = "tuffite-smoke-test";
inline constexpr char kWindowSurfaceTestSwitch[] =
    "tuffite-window-surface-test";
inline constexpr char kNetworkTestUrlSwitch[] = "tuffite-network-test-url";

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_COMMON_SWITCHES_H_
