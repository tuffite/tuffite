// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_SERVICE_IMPL_H_

#include "base/memory/raw_ref.h"
#include "tuffite/framework/browser/services/explorer/explorer_service.h"

namespace tuffite {
class ManifestService;
class ExplorerServiceImpl final : public ExplorerService {
 public:
  explicit ExplorerServiceImpl(const ManifestService& manifest);
  void AddNavigationThrottles(content::NavigationThrottleRegistry&) override;
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

  void Shutdown() override;

 private:
  const raw_ref<const ManifestService> manifest_;
  bool stopped_ = false;
};
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXPLORER_SERVICE_IMPL_H_
