// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/explorer/explorer_navigation_throttle.h"

#include <memory>
#include <utility>

#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/navigation_throttle.h"
#include "content/public/browser/navigation_throttle_registry.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/framework/browser/web_contents_role.h"

namespace tuffite {
namespace {
class ExplorerNavigationThrottle final : public content::NavigationThrottle {
 public:
  ExplorerNavigationThrottle(content::NavigationThrottleRegistry& registry,
                             std::string app_scheme)
      : content::NavigationThrottle(registry),
        app_scheme_(std::move(app_scheme)) {}

  ThrottleCheckResult WillStartRequest() override { return CheckUrl(); }
  ThrottleCheckResult WillRedirectRequest() override { return CheckUrl(); }

  const char* GetNameForLogging() override {
    return "TuffiteExplorerNavigationThrottle";
  }

 private:
  ThrottleCheckResult CheckUrl() {
    const ExplorerPolicy* policy = WebContentsRoleData::GetExplorerPolicy(
        navigation_handle()->GetWebContents());
    const auto& url = navigation_handle()->GetURL();
    if (!policy || !(url.IsAboutBlank() || url.SchemeIsHTTPOrHTTPS() ||
                     url.SchemeIsFile() || url.scheme() == app_scheme_)) {
      return BLOCK_REQUEST;
    }
    if (navigation_handle()->IsRendererInitiated() &&
        ExplorerHost::MaybeEmitNavigation(
            navigation_handle()->GetWebContents(),
            navigation_handle()->GetURL(),
            navigation_handle()->HasUserGesture())) {
      return CANCEL_AND_IGNORE;
    }
    return PROCEED;
  }
  const std::string app_scheme_;
};

}  // namespace

void AddExplorerNavigationThrottles(
    content::NavigationThrottleRegistry& registry,
    std::string app_scheme) {
  auto& handle = registry.GetNavigationHandle();
  if (handle.IsInPrimaryMainFrame() &&
      WebContentsRoleData::GetRole(handle.GetWebContents()) ==
          WebContentsRole::kExplorer) {
    registry.AddThrottle(std::make_unique<ExplorerNavigationThrottle>(
        registry, std::move(app_scheme)));
  }
}

}  // namespace tuffite
