// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/explorer/explorer_navigation_throttle.h"

#include <memory>
#include <utility>

#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/navigation_throttle.h"
#include "content/public/browser/navigation_throttle_registry.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/common/external_protocol.h"
#include "tuffite/framework/browser/web_contents_role.h"

namespace tuffite {
namespace {
class ExplorerNavigationThrottle final : public content::NavigationThrottle {
 public:
  ExplorerNavigationThrottle(content::NavigationThrottleRegistry& registry,
                             std::vector<std::string> app_schemes)
      : content::NavigationThrottle(registry),
        app_schemes_(std::move(app_schemes)) {}

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
                     url.SchemeIsFile() ||
                     std::ranges::contains(app_schemes_, url.scheme()) ||
                     IsExternalProtocol(url, app_schemes_))) {
      return BLOCK_REQUEST;
    }
    if (IsExternalProtocol(url, app_schemes_)) {
      return PROCEED;
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
  const std::vector<std::string> app_schemes_;
};

}  // namespace

void AddExplorerNavigationThrottles(
    content::NavigationThrottleRegistry& registry,
    std::vector<std::string> app_schemes) {
  auto& handle = registry.GetNavigationHandle();
  if (handle.IsInPrimaryMainFrame() &&
      WebContentsRoleData::GetRole(handle.GetWebContents()) ==
          WebContentsRole::kExplorer) {
    registry.AddThrottle(std::make_unique<ExplorerNavigationThrottle>(
        registry, std::move(app_schemes)));
  }
}

}  // namespace tuffite
