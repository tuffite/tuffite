// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_EXPLORER_BROWSER_FRAME_ANCESTRY_H_
#define TUFFITE_EXPLORER_BROWSER_FRAME_ANCESTRY_H_
#include <optional>
#include <vector>

#include "content/common/content_export.h"
#include "third_party/blink/public/common/frame/frame_policy.h"

namespace content {
class RenderFrameHost;
}

namespace tuffite {
// The first item describes the source, followed by its ownership ancestors.
CONTENT_EXPORT std::vector<bool> ExplorerFrameIsolationBoundaries(
    content::RenderFrameHost&);
CONTENT_EXPORT std::vector<std::optional<blink::FramePolicy>>
ExplorerFrameOwnerPolicies(content::RenderFrameHost&);
}  // namespace tuffite
#endif
