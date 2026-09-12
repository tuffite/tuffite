// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_CONTEXT_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_CONTEXT_H_

#include "base/check.h"
#include "base/memory/raw_ptr.h"
#include "base/unguessable_token.h"

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
  Context(content::RenderFrameHost& frame, base::UnguessableToken owner)
      : caller(Caller::kRenderer), sender(&frame), document(owner) {}
  Context(::tuffite::ApplicationContext& application,
          base::UnguessableToken owner)
      : caller(Caller::kApplication),
        native_application(&application),
        document(owner) {}
  bool is_application() const { return caller == Caller::kApplication; }
  content::RenderFrameHost& frame() const {
    CHECK(caller == Caller::kRenderer && sender);
    return *sender;
  }
  Caller caller;
  raw_ptr<content::RenderFrameHost> sender = nullptr;
  raw_ptr<::tuffite::ApplicationContext> native_application = nullptr;
  base::UnguessableToken document;
};

}  // namespace tuffite::dispatch
#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_CONTEXT_H_
