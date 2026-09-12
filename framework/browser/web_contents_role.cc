// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/web_contents_role.h"

#include "base/check.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/explorer/browser/explorer_surface_data.h"

namespace tuffite {

WebContentsRoleData::WebContentsRoleData(content::WebContents* web_contents,
                                         WebContentsRole role)
    : content::WebContentsUserData<WebContentsRoleData>(*web_contents),
      role_(role) {}

WebContentsRoleData::~WebContentsRoleData() = default;

// static
void WebContentsRoleData::SetRole(content::WebContents* web_contents,
                                  WebContentsRole role) {
  CHECK(web_contents);
  auto* existing = FromWebContents(web_contents);
  if (existing) {
    // Roles are immutable for the lifetime of a WebContents; re-assigning to
    // a different value would allow privilege escalation from browser code.
    CHECK(existing->role_ == role);
    return;
  }
  CreateForWebContents(web_contents, role);
}

// static
std::optional<WebContentsRole> WebContentsRoleData::GetRole(
    const content::WebContents* web_contents) {
  if (!web_contents) {
    return std::nullopt;
  }
  const auto* data = FromWebContents(web_contents);
  if (!data) {
    // Explorer creation attaches its immutable policy before the WebContents
    // is fully installed. During that narrow construction window the policy is
    // the authoritative role marker; all established contents use role_.
    if (ExplorerSurfaceData::GetExplorerPolicy(web_contents)) {
      return WebContentsRole::kExplorer;
    }
    return std::nullopt;
  }
  return data->role_;
}

// static
void WebContentsRoleData::SetExplorerPolicy(content::WebContents* contents,
                                            ExplorerPolicy policy) {
  CHECK(GetRole(contents) == WebContentsRole::kExplorer);
  ExplorerSurfaceData::SetExplorerPolicy(contents, std::move(policy));
}
const ExplorerPolicy* WebContentsRoleData::GetExplorerPolicy(
    const content::WebContents* contents) {
  return ExplorerSurfaceData::GetExplorerPolicy(contents);
}
void WebContentsRoleData::SetExplorerContextMenuEnabled(
    content::WebContents* contents,
    bool enabled) {
  ExplorerSurfaceData::SetExplorerContextMenuEnabled(contents, enabled);
}
bool WebContentsRoleData::IsExplorerContextMenuEnabled(
    const content::WebContents* contents) {
  return ExplorerSurfaceData::IsExplorerContextMenuEnabled(contents);
}

WEB_CONTENTS_USER_DATA_KEY_IMPL(WebContentsRoleData);

}  // namespace tuffite
