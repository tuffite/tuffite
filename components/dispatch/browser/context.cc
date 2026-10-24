// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/dispatch/browser/context.h"

#include "content/public/browser/render_frame_host.h"

namespace tuffite::dispatch {

Context::Context(const Context&) = default;
Context& Context::operator=(const Context&) = default;
Context::Context(Context&&) = default;
Context& Context::operator=(Context&&) = default;
Context::~Context() = default;

bool Context::is_valid() const {
  return is_application() ? !!native_application : sender() != nullptr;
}

Context::Context(content::RenderFrameHost& frame, base::UnguessableToken owner)
    : caller(Caller::kRenderer),
      document(owner),
      sender_(frame.GetWeakDocumentPtr()) {}

content::RenderFrameHost* Context::sender() const {
  return sender_.AsRenderFrameHostIfValid();
}

content::RenderFrameHost& Context::frame() const {
  CHECK(caller == Caller::kRenderer);
  auto* value = sender();
  CHECK(value);
  return *value;
}

}  // namespace tuffite::dispatch
