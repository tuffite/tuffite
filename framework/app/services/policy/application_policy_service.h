// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_H_
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/components/explorer/browser/explorer_policy_provider.h"
namespace content {
class NavigationThrottleRegistry;
}

namespace tuffite {
class AppManifest;
class ApplicationPolicyService : public KeyedService,
                                 public ExplorerPolicyProvider {
 public:
  virtual void AddNavigationThrottles(
      content::NavigationThrottleRegistry& registry) = 0;
  virtual const AppManifest& manifest() const = 0;
  virtual bool ContextMenuAllowed(content::WebContents* contents) const = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_H_
