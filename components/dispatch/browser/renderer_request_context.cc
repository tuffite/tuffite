// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/renderer_request_context.h"

#include "base/check.h"
#include "content/public/browser/render_frame_host.h"

namespace tuffite::dispatch {
RendererRequestContext::RendererRequestContext(
    content::RenderFrameHost& frame,
    base::UnguessableToken connection,
    uint64_t request_id)
    : document_(frame.GetWeakDocumentPtr()),
      connection_(connection),
      request_id_(request_id) {
  CHECK(!connection_.is_empty());
  CHECK_NE(request_id_, 0u);
}
RendererRequestContext::RendererRequestContext(const RendererRequestContext&) =
    default;
RendererRequestContext& RendererRequestContext::operator=(
    const RendererRequestContext&) = default;
RendererRequestContext::RendererRequestContext(RendererRequestContext&&) =
    default;
RendererRequestContext& RendererRequestContext::operator=(
    RendererRequestContext&&) = default;
RendererRequestContext::~RendererRequestContext() = default;
bool RendererRequestContext::is_valid() const {
  return sender() != nullptr;
}
content::RenderFrameHost* RendererRequestContext::sender() const {
  auto* frame = document_.AsRenderFrameHostIfValid();
  // A BFCache document can outlive navigation, but may no longer issue work.
  return frame && frame->IsActive() ? frame : nullptr;
}
content::RenderFrameHost& RendererRequestContext::frame() const {
  auto* value = sender();
  CHECK(value);
  return *value;
}
}  // namespace tuffite::dispatch
