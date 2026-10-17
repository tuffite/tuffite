// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_EXPLORER_HANDLE_SERIALIZATION_H_
#define TUFFITE_EXPLORER_HANDLE_SERIALIZATION_H_
#include <optional>

#include "third_party/blink/renderer/bindings/core/v8/serialization/serialized_script_value.h"
#include "third_party/blink/renderer/bindings/core/v8/serialization/transferables.h"
#include "tuffite/components/explorer/renderer/blink/explorer_handle.h"

namespace blink {
class ExceptionState;
class ExplorerHandleTransferList final
    : public GarbageCollected<ExplorerHandleTransferList>,
      public Transferables::TransferList {
 public:
  ExplorerHandleTransferList();
  ~ExplorerHandleTransferList() override;
  static const void* kTransferListKey;
  HeapVector<Member<ExplorerHandle>> handles;
  Vector<uint32_t> slots;
  void Trace(Visitor*) const override;
};

class CORE_EXPORT ExplorerHandleSerialization {
 public:
  static std::optional<bool> Extract(v8::Isolate*,
                                     v8::Local<v8::Value>,
                                     Transferables&,
                                     ExceptionState&);
  static bool IsHandle(ScriptWrappable*);
  static bool IsExposed(ExecutionContext*);
  static bool Prepare(ScriptWrappable*,
                      const Transferables*,
                      SerializedScriptValue&,
                      uint32_t&,
                      String&,
                      ExceptionState&);
  static void Finalize(const Transferables&,
                       SerializedScriptValue&,
                       ExceptionState&);
  static ExplorerHandle* Read(ExecutionContext*,
                              SerializedScriptValue&,
                              uint32_t,
                              const String&);
};
}  // namespace blink
#endif
