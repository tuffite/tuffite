// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_CONTENT_EXPLORER_VISIBILITY_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_CONTENT_EXPLORER_VISIBILITY_H_

#include "content/common/content_export.h"
#include "third_party/blink/public/mojom/frame/lifecycle.mojom-forward.h"

namespace content {
class RenderFrameHost;
class RenderFrameProxyHost;
}  // namespace content

namespace tuffite {
// A discarded renderer has no child view. Forward owner visibility through the
// existing inner-WebContents delegate so showing it can initiate restoration.
CONTENT_EXPORT void NotifyDiscardedExplorerVisibility(
    content::RenderFrameHost*,
    content::RenderFrameProxyHost*,
    blink::mojom::FrameVisibility);
}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_CONTENT_EXPLORER_VISIBILITY_H_
