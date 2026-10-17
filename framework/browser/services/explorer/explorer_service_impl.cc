// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/explorer/explorer_service_impl.h"

#include <algorithm>

#include "base/check.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/explorer/browser/explorer_partition.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/explorer/explorer_navigation_throttle.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/common/access.h"

namespace tuffite {
ExplorerServiceImpl::ExplorerServiceImpl(const ManifestService& manifest)
    : manifest_(manifest) {}
bool ExplorerServiceImpl::IsApp(const content::WebContents* contents) const {
  return !stopped_ &&
         WebContentsRoleData::GetRole(contents) == WebContentsRole::kApp;
}
bool ExplorerServiceImpl::IsSurfaceAllowed(
    content::RenderFrameHost& frame) const {
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  auto role = WebContentsRoleData::GetRole(contents);
  return !stopped_ && contents && contents->GetPrimaryMainFrame() == &frame &&
         role &&
         (*role == WebContentsRole::kApp ||
          *role == WebContentsRole::kExplorer);
}
bool ExplorerServiceImpl::IsOwnerAllowed(
    content::RenderFrameHost& frame) const {
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  auto role = WebContentsRoleData::GetRole(contents);
  if (stopped_ || !contents || !role ||
      !manifest_->IsExplorerOriginAllowed(frame.GetLastCommittedOrigin())) {
    return false;
  }
  return IsApp(contents) || WebContentsRoleData::GetExplorerPolicy(contents);
}
bool ExplorerServiceImpl::NavigationAllowed(content::RenderFrameHost& owner,
                                            const GURL& url) const {
  return !stopped_ && manifest_->IsExplorerNavigationAllowed(
                          owner.GetLastCommittedOrigin(), url);
}

bool ExplorerServiceImpl::ExecuteJavaScriptAllowed(
    content::RenderFrameHost& owner,
    const url::Origin& origin,
    const ExplorerPolicy& ceiling) const {
  if (stopped_) {
    return false;
  }
  const auto current =
      manifest_->ExecuteJavaScriptOrigins(owner.GetLastCommittedOrigin());
  auto matches = [&](const auto& patterns) {
    return std::ranges::any_of(patterns, [&](const auto& pattern) {
      return access::MatchesOrigin(origin, pattern);
    });
  };
  return matches(current) && matches(ceiling.execute_javascript_origins);
}

bool ExplorerServiceImpl::Resolve(content::RenderFrameHost& frame,
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

void ExplorerServiceImpl::AddNavigationThrottles(
    content::NavigationThrottleRegistry& registry) {
  if (!stopped_) {
    AddExplorerNavigationThrottles(registry, manifest_->app_info().scheme);
  }
}
void ExplorerServiceImpl::Shutdown() {
  stopped_ = true;
}
}  // namespace tuffite
