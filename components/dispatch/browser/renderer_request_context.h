// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_RENDERER_REQUEST_CONTEXT_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_RENDERER_REQUEST_CONTEXT_H_

#include <cstdint>

#include "base/unguessable_token.h"
#include "content/public/browser/weak_document_ptr.h"

namespace content {
class RenderFrameHost;
}
namespace tuffite::dispatch {

// Immutable renderer request identity. Native ABI calls never construct this
// context. A navigated, deleted or inactive document cannot issue work.
class RendererRequestContext {
 public:
  RendererRequestContext(content::RenderFrameHost&,
                         base::UnguessableToken connection,
                         uint64_t request_id);
  RendererRequestContext(const RendererRequestContext&);
  RendererRequestContext& operator=(const RendererRequestContext&);
  RendererRequestContext(RendererRequestContext&&);
  RendererRequestContext& operator=(RendererRequestContext&&);
  ~RendererRequestContext();
  bool is_valid() const;
  content::RenderFrameHost* sender() const;
  content::RenderFrameHost& frame() const;
  const base::UnguessableToken& connection() const { return connection_; }
  uint64_t request_id() const { return request_id_; }

 private:
  content::WeakDocumentPtr document_;
  base::UnguessableToken connection_;
  uint64_t request_id_;
};
}  // namespace tuffite::dispatch
#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_RENDERER_REQUEST_CONTEXT_H_
