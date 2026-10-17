// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_SERVICE_H_
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/components/explorer/browser/explorer_policy_provider.h"
namespace content {
class NavigationThrottleRegistry;
}
namespace tuffite {
class ExplorerService : public KeyedService, public ExplorerPolicyProvider {
 public:
  virtual void AddNavigationThrottles(content::NavigationThrottleRegistry&) = 0;
};
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_SERVICE_H_
