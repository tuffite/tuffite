// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/renderer/blink/explorer_new_window.h"

#include "third_party/blink/renderer/core/dom/abort_controller.h"
#include "tuffite/components/explorer/renderer/blink/html_explorer_element.h"
namespace blink {
namespace {
class Attached final : public ThenCallable<IDLAny, Attached, IDLBoolean> {
 public:
  bool React(ScriptState*, ScriptValue) { return true; }
};
class AttachFailed final
    : public ThenCallable<IDLAny, AttachFailed, IDLPromise<IDLBoolean>> {
 public:
  AttachFailed(HTMLExplorerElement* owner, String token)
      : owner_(owner), token_(std::move(token)) {}
  ScriptPromise<IDLBoolean> React(ScriptState* state, ScriptValue error) {
    owner_->DiscardWindow(state, token_);
    return ScriptPromise<IDLBoolean>::Reject(state, error);
  }
  void Trace(Visitor* visitor) const override {
    visitor->Trace(owner_);
    ThenCallable<IDLAny, AttachFailed, IDLPromise<IDLBoolean>>::Trace(visitor);
  }

 private:
  Member<HTMLExplorerElement> owner_;
  const String token_;
};
}  // namespace
ExplorerNewWindow::ExplorerNewWindow(ScriptState* state,
                                     HTMLExplorerElement& owner,
                                     String token,
                                     String url,
                                     String disposition,
                                     String from,
                                     bool user_gesture)
    : owner_(owner),
      controller_(AbortController::Create(state)),
      token_(std::move(token)),
      url_(std::move(url)),
      disposition_(std::move(disposition)),
      from_(std::move(from)),
      user_gesture_(user_gesture) {}
AbortSignal* ExplorerNewWindow::signal() const {
  return controller_->signal();
}
ExplorerNewWindow::~ExplorerNewWindow() = default;
void ExplorerNewWindow::Finish(ScriptState* state, bool canceled) {
  responded_ = true;
  if (canceled) {
    controller_->abort(state);
  }
}
ScriptPromise<IDLBoolean> ExplorerNewWindow::attach(
    ScriptState* state,
    HTMLExplorerElement* target) {
  if (responded_) {
    return ToResolvedPromise<IDLBoolean>(state, false);
  }
  responded_ = true;
  auto attached = target->AttachToken(state, token_);
  // ThenCallable's undefined-input branch discards primitive return values.
  // Treat the fulfilled value as any to propagate the controller's boolean.
  return ScriptPromise<IDLAny>::FromV8Promise(state->GetIsolate(),
                                              attached.V8Promise())
      .Then(state, MakeGarbageCollected<Attached>())
      .Catch(state, MakeGarbageCollected<AttachFailed>(owner_, token_));
}
ScriptPromise<IDLBoolean> ExplorerNewWindow::discard(ScriptState* state) {
  if (responded_ || !owner_) {
    return ToResolvedPromise<IDLBoolean>(state, false);
  }
  responded_ = true;
  return owner_->DiscardWindow(state, token_);
}
void ExplorerNewWindow::Trace(Visitor* visitor) const {
  visitor->Trace(owner_);
  visitor->Trace(controller_);
  ScriptWrappable::Trace(visitor);
}
}  // namespace blink
