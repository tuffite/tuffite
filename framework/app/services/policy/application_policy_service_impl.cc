// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/policy/application_policy_service_impl.h"

#include <algorithm>

#include "base/check.h"
#include "base/command_line.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/navigation_throttle.h"
#include "content/public/browser/navigation_throttle_registry.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "content/public/common/content_switches.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/browser/explorer_partition.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/common/app_manifest.h"
#include "tuffite/framework/common/capability_pattern.h"

namespace tuffite {
namespace {

class ExplorerNavigationThrottle final : public content::NavigationThrottle {
 public:
  ExplorerNavigationThrottle(content::NavigationThrottleRegistry& registry,
                             const AppManifest& manifest)
      : content::NavigationThrottle(registry), manifest_(manifest) {}

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
    if (!policy ||
        !(url.IsAboutBlank() || url.SchemeIsHTTPOrHTTPS() ||
          url.SchemeIsFile() || url.scheme() == manifest_->app_scheme())) {
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
  const raw_ref<const AppManifest> manifest_;
};

}  // namespace

void ApplicationPolicyServiceImpl::AddNavigationThrottles(
    content::NavigationThrottleRegistry& registry) {
  auto& handle = registry.GetNavigationHandle();
  if (handle.IsInPrimaryMainFrame() &&
      WebContentsRoleData::GetRole(handle.GetWebContents()) ==
          WebContentsRole::kExplorer) {
    registry.AddThrottle(
        std::make_unique<ExplorerNavigationThrottle>(registry, *manifest_));
  }
}

bool ApplicationPolicyServiceImpl::ContextMenuAllowed(
    content::WebContents* web_contents) const {
  if (WebContentsRoleData::IsExplorerContextMenuEnabled(web_contents)) {
    return true;
  }
  if (WebContentsRoleData::GetRole(web_contents) != WebContentsRole::kApp) {
    return false;
  }
  // A normal packaged launch can opt into the root App context menu without
  // exposing a persistent debugging endpoint. InspectElement() starts a
  // loopback server on an ephemeral port only when the command is invoked.
  return manifest_->IsShellApiCommandAllowed(
             web_contents->GetPrimaryMainFrame()->GetLastCommittedOrigin(),
             "tuffite.devtools.inspectElement") ||
         base::CommandLine::ForCurrentProcess()->HasSwitch(
             switches::kRemoteDebuggingPort);
}

const AppManifest& ApplicationPolicyServiceImpl::manifest() const {
  return *manifest_;
}

bool ApplicationPolicyServiceImpl::IsApp(
    const content::WebContents* contents) const {
  return WebContentsRoleData::GetRole(contents) == WebContentsRole::kApp;
}
bool ApplicationPolicyServiceImpl::IsSurfaceAllowed(
    content::RenderFrameHost& frame) const {
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  auto role = WebContentsRoleData::GetRole(contents);
  return contents && contents->GetPrimaryMainFrame() == &frame && role &&
         (*role == WebContentsRole::kApp ||
          *role == WebContentsRole::kExplorer);
}
bool ApplicationPolicyServiceImpl::IsOwnerAllowed(
    content::RenderFrameHost& frame) const {
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  auto role = WebContentsRoleData::GetRole(contents);
  if (!contents || !role ||
      !manifest_->IsExplorerOriginAllowed(frame.GetLastCommittedOrigin())) {
    return false;
  }
  return IsApp(contents) || WebContentsRoleData::GetExplorerPolicy(contents);
}
bool ApplicationPolicyServiceImpl::NavigationAllowed(
    content::RenderFrameHost& owner,
    const GURL& url) const {
  return manifest_->IsExplorerNavigationAllowed(owner.GetLastCommittedOrigin(),
                                                url);
}

bool ApplicationPolicyServiceImpl::ExecuteJavaScriptAllowed(
    content::RenderFrameHost& owner,
    const url::Origin& origin,
    const ExplorerPolicy& ceiling) const {
  const auto current =
      manifest_->ExecuteJavaScriptOrigins(owner.GetLastCommittedOrigin());
  auto matches = [&](const auto& patterns) {
    return std::ranges::any_of(patterns, [&](const auto& pattern) {
      return capability_pattern::MatchesOrigin(origin, pattern);
    });
  };
  return matches(current) && matches(ceiling.execute_javascript_origins);
}

bool ApplicationPolicyServiceImpl::Resolve(content::RenderFrameHost& frame,
                                           const std::string& partition,
                                           ExplorerPolicy* policy) const {
  CHECK(policy);
  if (!IsOwnerAllowed(frame)) {
    return false;
  }
  const auto parsed = ExplorerPartition::Parse(partition);
  if (!parsed) {
    return false;
  }
  ExplorerPolicy resolved;
  resolved.partition = partition;
  resolved.inherit_partition_storage = parsed->inherit;
  resolved.execute_javascript_origins =
      manifest_->ExecuteJavaScriptOrigins(frame.GetLastCommittedOrigin());
  resolved.allow_execute_javascript =
      !resolved.execute_javascript_origins.empty();

  resolved.allow_dialogs = true;
  resolved.allow_parent_messages = true;
  resolved.allow_top_messages = true;
  // Binding/call grants come exclusively from the guest's committed origin.
  resolved.allow_shell_api = true;
  *policy = std::move(resolved);
  return true;
}

}  // namespace tuffite
