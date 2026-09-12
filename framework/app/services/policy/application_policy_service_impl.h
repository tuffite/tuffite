// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_IMPL_H_

#include "base/memory/raw_ref.h"
#include "tuffite/framework/app/services/policy/application_policy_service.h"

namespace tuffite {
class ApplicationPolicyServiceImpl final : public ApplicationPolicyService {
 public:
  explicit ApplicationPolicyServiceImpl(const AppManifest& manifest)
      : manifest_(manifest) {}
  void AddNavigationThrottles(
      content::NavigationThrottleRegistry& registry) override;
  const AppManifest& manifest() const override;
  bool ContextMenuAllowed(content::WebContents* contents) const override;
  bool IsSurfaceAllowed(content::RenderFrameHost& frame) const override;
  bool IsOwnerAllowed(content::RenderFrameHost& frame) const override;
  bool IsApp(const content::WebContents* contents) const override;
  bool Resolve(content::RenderFrameHost& frame,
               const std::string& partition,
               ExplorerPolicy* policy) const override;
  bool NavigationAllowed(content::RenderFrameHost& owner,
                         const GURL& url) const override;
  bool ExecuteJavaScriptAllowed(content::RenderFrameHost&,
                                const url::Origin&,
                                const ExplorerPolicy&) const override;

 private:
  const raw_ref<const AppManifest> manifest_;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_IMPL_H_
