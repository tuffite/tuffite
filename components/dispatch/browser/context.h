// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_CONTEXT_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_CONTEXT_H_

#include "base/check.h"
#include "base/memory/weak_ptr.h"
#include "base/unguessable_token.h"
#include "content/public/browser/weak_document_ptr.h"

namespace content {
class RenderFrameHost;
}

namespace tuffite {
class ApplicationContext;
}

namespace tuffite::dispatch {

// Explicit caller identity. Native callers never borrow or impersonate a frame.
struct Context {
  enum class Caller { kRenderer, kApplication };
  Context(content::RenderFrameHost&, base::UnguessableToken owner);
  Context(::tuffite::ApplicationContext&, base::UnguessableToken owner);
  Context(const Context&);
  Context& operator=(const Context&);
  Context(Context&&);
  Context& operator=(Context&&);
  ~Context();
  bool is_application() const { return caller == Caller::kApplication; }
  bool is_valid() const;
  content::RenderFrameHost* sender() const;
  content::RenderFrameHost& frame() const;
  Caller caller;
  base::WeakPtr<::tuffite::ApplicationContext> native_application;
  base::UnguessableToken document;

 private:
  content::WeakDocumentPtr sender_;
};

}  // namespace tuffite::dispatch
#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_CONTEXT_H_
