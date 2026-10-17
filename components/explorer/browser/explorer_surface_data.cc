// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_surface_data.h"

#include <cmath>

#include "base/check.h"
#include "content/public/browser/host_zoom_map.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "third_party/blink/public/common/page/page_zoom.h"

namespace tuffite {
ExplorerSurfaceData::ExplorerSurfaceData(content::WebContents* contents)
    : content::WebContentsUserData<ExplorerSurfaceData>(*contents),
      content::WebContentsObserver(contents) {}
ExplorerSurfaceData::~ExplorerSurfaceData() = default;
void ExplorerSurfaceData::SetWindowSource(content::WebContents* contents,
                                          content::RenderFrameHost* source) {
  CreateForWebContents(contents);
  FromWebContents(contents)->window_source_ = source->GetWeakDocumentPtr();
}
content::RenderFrameHost* ExplorerSurfaceData::GetWindowSource(
    content::WebContents* contents) {
  const auto* data = FromWebContents(contents);
  return data ? data->window_source_.AsRenderFrameHostIfValid() : nullptr;
}
void ExplorerSurfaceData::MarkInitialWindowNavigation(
    content::WebContents* contents) {
  CreateForWebContents(contents);
  FromWebContents(contents)->initial_window_navigation_ = true;
}
bool ExplorerSurfaceData::IsInitialWindowNavigation(
    content::WebContents* contents) {
  auto* data = FromWebContents(contents);
  return data && data->initial_window_navigation_;
}
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
bool ExplorerSurfaceData::SetZoomFactor(content::WebContents* contents,
                                        double factor) {
  if (!std::isfinite(factor) || factor < 0.25 || factor > 5.0) {
    return false;
  }
  CreateForWebContents(contents);
  auto* data = FromWebContents(contents);
  data->zoom_factor_ = factor;
  data->ApplyZoom();
  return true;
}
double ExplorerSurfaceData::GetZoomFactor(content::WebContents* contents) {
  return blink::ZoomLevelToZoomFactor(
      content::HostZoomMap::GetZoomLevel(contents));
}
void ExplorerSurfaceData::ApplyZoom() {
  auto* contents = web_contents();
  auto* frame = contents->GetPrimaryMainFrame();
  // Temporary per-frame zoom keeps sibling tabs and the app shell unchanged.
  content::HostZoomMap::GetForWebContents(contents)->SetTemporaryZoomLevel(
      frame->GetGlobalId(), blink::ZoomFactorToZoomLevel(zoom_factor_));
}
void ExplorerSurfaceData::DidFinishNavigation(
    content::NavigationHandle* handle) {
  if (!handle->IsInPrimaryMainFrame() || !handle->HasCommitted() ||
      handle->IsSameDocument()) {
    return;
  }
  favicons_.clear();
  if (!handle->GetURL().IsAboutBlank()) {
    initial_window_navigation_ = false;
  }
  ApplyZoom();
}
void ExplorerSurfaceData::SetFavicons(content::WebContents* contents,
                                      base::ListValue candidates) {
  CreateForWebContents(contents);
  FromWebContents(contents)->favicons_ = std::move(candidates);
}
base::ListValue ExplorerSurfaceData::GetFavicons(
    const content::WebContents* contents) {
  const auto* data = FromWebContents(contents);
  return data ? data->favicons_.Clone() : base::ListValue();
}
WEB_CONTENTS_USER_DATA_KEY_IMPL(ExplorerSurfaceData);
}  // namespace tuffite
