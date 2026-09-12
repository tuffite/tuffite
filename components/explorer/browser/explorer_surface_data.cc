// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_surface_data.h"

#include "base/check.h"

namespace tuffite {
ExplorerSurfaceData::ExplorerSurfaceData(content::WebContents* contents)
    : content::WebContentsUserData<ExplorerSurfaceData>(*contents) {}
ExplorerSurfaceData::~ExplorerSurfaceData() = default;
void ExplorerSurfaceData::SetNavigationDownload(content::WebContents* contents,
                                                std::optional<uint32_t> id) {
  CreateForWebContents(contents);
  FromWebContents(contents)->navigation_download_ = id;
}
std::optional<uint32_t> ExplorerSurfaceData::NavigationDownload(
    const content::WebContents* contents) {
  const auto* data = FromWebContents(contents);
  return data ? data->navigation_download_ : std::nullopt;
}
void ExplorerSurfaceData::SetExplorerPolicy(content::WebContents* contents,
                                            ExplorerPolicy policy) {
  CreateForWebContents(contents);
  auto* data = FromWebContents(contents);
  CHECK(!data->policy_);
  data->policy_ = std::move(policy);
}
const ExplorerPolicy* ExplorerSurfaceData::GetExplorerPolicy(
    const content::WebContents* contents) {
  const auto* data = contents ? FromWebContents(contents) : nullptr;
  return data && data->policy_ ? &*data->policy_ : nullptr;
}
void ExplorerSurfaceData::SetExplorerContextMenuEnabled(
    content::WebContents* contents,
    bool enabled) {
  auto* data = FromWebContents(contents);
  CHECK(data);
  data->menu_enabled_ = enabled;
}
bool ExplorerSurfaceData::IsExplorerContextMenuEnabled(
    const content::WebContents* contents) {
  const auto* data = contents ? FromWebContents(contents) : nullptr;
  return data && data->menu_enabled_;
}
WEB_CONTENTS_USER_DATA_KEY_IMPL(ExplorerSurfaceData);
}  // namespace tuffite
