// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_EXPLORER_MESSAGE_SERIALIZATION_H_
#define TUFFITE_EXPLORER_MESSAGE_SERIALIZATION_H_
#include "third_party/blink/renderer/core/messaging/blink_transferable_message.h"
namespace blink {
class ScriptState;
class ScriptValue;
class WindowPostMessageOptions;
class ExceptionState;
CORE_EXPORT BlinkTransferableMessage
SerializeExplorerMessage(ScriptState*,
                         const ScriptValue&,
                         const WindowPostMessageOptions*,
                         scoped_refptr<const SecurityOrigin>&,
                         ExceptionState&);
}  // namespace blink
#endif
