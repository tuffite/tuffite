// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#include "tuffite/components/explorer/renderer/blink/explorer_dialog.h"

#include <utility>

#include "tuffite/components/explorer/renderer/blink/html_explorer_element.h"

namespace blink {

ExplorerDialog::ExplorerDialog(HTMLExplorerElement& owner,
                               uint64_t request_id,
                               String type,
                               String message,
                               String default_value,
                               bool is_reload,
                               String from)
    : owner_(owner),
      request_id_(request_id),
      type_(std::move(type)),
      message_(std::move(message)),
      default_value_(std::move(default_value)),
      is_reload_(is_reload),
      from_(std::move(from)) {}

ExplorerDialog::~ExplorerDialog() = default;

ScriptPromise<IDLBoolean> ExplorerDialog::accept(ScriptState* script_state,
                                                 const String& prompt_text) {
  return Respond(script_state, true, prompt_text);
}

ScriptPromise<IDLBoolean> ExplorerDialog::dismiss(ScriptState* script_state) {
  return Respond(script_state, false, String());
}

ScriptPromise<IDLBoolean> ExplorerDialog::Respond(ScriptState* script_state,
                                                  bool accept,
                                                  const String& prompt_text) {
  if (responded_ || !owner_) {
    return ToResolvedPromise<IDLBoolean>(script_state, false);
  }
  responded_ = true;
  return owner_->RespondToDialog(script_state, request_id_, accept,
                                 prompt_text);
}

void ExplorerDialog::Trace(Visitor* visitor) const {
  visitor->Trace(owner_);
  ScriptWrappable::Trace(visitor);
}

}  // namespace blink
