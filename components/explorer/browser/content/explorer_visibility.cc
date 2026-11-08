// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/content/explorer_visibility.h"

#include "content/browser/renderer_host/frame_tree.h"
#include "content/browser/renderer_host/render_frame_host_impl.h"
#include "content/public/browser/navigation_controller.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/explorer/browser/content/explorer_isolation.h"

namespace tuffite {
void NotifyDiscardedExplorerVisibility(
    content::RenderFrameHost* frame,
    content::RenderFrameProxyHost* proxy,
    blink::mojom::FrameVisibility visibility) {
  if (!frame || !proxy) {
    return;
  }
  auto* contents = content::WebContents::FromRenderFrameHost(frame);
  if (!GetExplorerIsolationId(contents) ||
      contents->GetPrimaryMainFrame() != frame ||
      !contents->GetOuterWebContents() ||
      !contents->GetController().NeedsReload()) {
    return;
  }
  // Reuse Chromium's visibility cap from the outer WebContents, including its
  // occluded state, rather than showing a discarded page unconditionally.
  static_cast<content::RenderFrameHostImpl*>(frame)
      ->frame_tree()
      ->delegate()
      ->OnRenderFrameProxyVisibilityChanged(proxy, visibility);
}
}  // namespace tuffite
