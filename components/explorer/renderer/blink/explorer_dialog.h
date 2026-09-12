// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#ifndef TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_DIALOG_H_
#define TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_DIALOG_H_

#include <cstdint>

#include "third_party/blink/renderer/bindings/core/v8/script_promise.h"
#include "third_party/blink/renderer/core/core_export.h"
#include "third_party/blink/renderer/platform/bindings/script_wrappable.h"
#include "third_party/blink/renderer/platform/heap/member.h"
#include "third_party/blink/renderer/platform/wtf/text/wtf_string.h"

namespace blink {

class HTMLExplorerElement;
class ScriptState;

class CORE_EXPORT ExplorerDialog final : public ScriptWrappable {
  DEFINE_WRAPPERTYPEINFO();

 public:
  ExplorerDialog(HTMLExplorerElement& owner,
                 uint64_t request_id,
                 String type,
                 String message,
                 String default_value,
                 bool is_reload,
                 String from);
  ~ExplorerDialog() final;

  const String& type() const { return type_; }
  const String& message() const { return message_; }
  const String& defaultValue() const { return default_value_; }
  bool isReload() const { return is_reload_; }
  const String& from() const { return from_; }

  ScriptPromise<IDLBoolean> accept(ScriptState*, const String& prompt_text);
  ScriptPromise<IDLBoolean> dismiss(ScriptState*);

  void Trace(Visitor*) const override;

 private:
  ScriptPromise<IDLBoolean> Respond(ScriptState*,
                                    bool accept,
                                    const String& prompt_text);

  Member<HTMLExplorerElement> owner_;
  const uint64_t request_id_;
  const String type_;
  const String message_;
  const String default_value_;
  const bool is_reload_;
  const String from_;
  bool responded_ = false;
};

}  // namespace blink

#endif  // TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_DIALOG_H_
