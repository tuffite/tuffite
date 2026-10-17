// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_NAVIGATION_THROTTLE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_NAVIGATION_THROTTLE_H_

#include <string>

namespace content {
class NavigationThrottleRegistry;
}
namespace tuffite {
void AddExplorerNavigationThrottles(content::NavigationThrottleRegistry&,
                                    std::string app_scheme);
}
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_NAVIGATION_THROTTLE_H_
