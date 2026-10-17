// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_NEW_WINDOW_H_
#define TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_NEW_WINDOW_H_
#include "third_party/blink/renderer/bindings/core/v8/script_promise.h"
#include "third_party/blink/renderer/core/core_export.h"
#include "third_party/blink/renderer/platform/bindings/script_wrappable.h"
#include "third_party/blink/renderer/platform/heap/member.h"
namespace blink {
class HTMLExplorerElement;
class AbortController;
class AbortSignal;
class CORE_EXPORT ExplorerNewWindow final : public ScriptWrappable {
  DEFINE_WRAPPERTYPEINFO();

 public:
  ExplorerNewWindow(ScriptState*,
                    HTMLExplorerElement&,
                    String token,
                    String url,
                    String disposition,
                    String from,
                    bool user_gesture);
  ~ExplorerNewWindow() override;
  const String& url() const { return url_; }
  const String& disposition() const { return disposition_; }
  const String& from() const { return from_; }
  bool userGesture() const { return user_gesture_; }
  AbortSignal* signal() const;
  bool responded() const { return responded_; }
  void Finish(ScriptState*, bool canceled);
  ScriptPromise<IDLBoolean> attach(ScriptState*, HTMLExplorerElement*);
  ScriptPromise<IDLBoolean> discard(ScriptState*);
  void Trace(Visitor*) const override;

 private:
  Member<HTMLExplorerElement> owner_;
  Member<AbortController> controller_;
  const String token_, url_, disposition_, from_;
  const bool user_gesture_;
  bool responded_ = false;
};
}  // namespace blink
#endif
