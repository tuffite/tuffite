// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_CONSTANTS_H_
#define TUFFITE_FRAMEWORK_COMMON_CONSTANTS_H_

namespace tuffite {

// The current application locale is en-US; use the same ordered languages in
// network headers, document preferences and workers.
inline constexpr char kAcceptLanguages[] = "en-US,en";

inline constexpr char kAppSchemeSwitch[] = "tuffite-app-scheme";
inline constexpr char kShellApiOriginsSwitch[] = "tuffite-shell-api-origins";
inline constexpr char kExplorerOriginsSwitch[] = "tuffite-explorer-origins";
inline constexpr char kDevUrlSwitch[] = "tuffite-dev-url";
inline constexpr char kDevTokenSwitch[] = "tuffite-dev-token";
inline constexpr char kDevTokenEnvironment[] = "TUFFITE_DEV_TOKEN";
inline constexpr char kSmokeTestSwitch[] = "tuffite-smoke-test";
inline constexpr char kWindowSurfaceTestSwitch[] =
    "tuffite-window-surface-test";
inline constexpr char kNetworkTestUrlSwitch[] = "tuffite-network-test-url";

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_COMMON_CONSTANTS_H_
