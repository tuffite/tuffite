// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/renderer/blink/explorer_handle_serialization.h"

#include <limits>

#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_handle.h"
#include "third_party/blink/renderer/platform/bindings/exception_state.h"

namespace blink {
const void* ExplorerHandleTransferList::kTransferListKey = nullptr;
ExplorerHandleTransferList::ExplorerHandleTransferList() = default;
ExplorerHandleTransferList::~ExplorerHandleTransferList() = default;
void ExplorerHandleTransferList::Trace(Visitor* visitor) const {
  visitor->Trace(handles);
  Transferables::TransferList::Trace(visitor);
}
std::optional<bool> ExplorerHandleSerialization::Extract(
    v8::Isolate* isolate,
    v8::Local<v8::Value> value,
    Transferables& transferables,
    ExceptionState& exception) {
  auto* handle = V8ExplorerHandle::ToWrappable(isolate, value);
  if (!handle) {
    return std::nullopt;
  }
  auto* list =
      transferables.GetOrCreateTransferList<ExplorerHandleTransferList>();
  if (!handle->CanTransfer() || list->handles.Contains(handle)) {
    exception.ThrowDOMException(DOMExceptionCode::kDataCloneError,
                                "ExplorerHandle is unavailable or duplicated");
    return false;
  }
  list->handles.push_back(handle);
  list->slots.push_back(std::numeric_limits<uint32_t>::max());
  return true;
}
bool ExplorerHandleSerialization::IsHandle(ScriptWrappable* value) {
  return value->GetWrapperTypeInfo() ==
         ExplorerHandle::GetStaticWrapperTypeInfo();
}
bool ExplorerHandleSerialization::IsExposed(ExecutionContext* context) {
  return V8ExplorerHandle::IsExposed(context);
}
bool ExplorerHandleSerialization::Prepare(ScriptWrappable* value,
                                          const Transferables* transferables,
                                          SerializedScriptValue& serialized,
                                          uint32_t& slot,
                                          String& token,
                                          ExceptionState& exception) {
  auto* list =
      transferables
          ? transferables->GetTransferListIfExists<ExplorerHandleTransferList>()
          : nullptr;
  auto* handle = static_cast<ExplorerHandle*>(value);
  const auto index = list ? list->handles.Find(handle) : kNotFound;
  if (!list || index == kNotFound || !handle->CanTransfer()) {
    exception.ThrowDOMException(DOMExceptionCode::kDataCloneError,
                                "ExplorerHandle requires a transfer list");
    return false;
  }
  auto* writable = const_cast<ExplorerHandleTransferList*>(list);
  slot = serialized.MojoHandles().size();
  writable->slots[index] = slot;
  serialized.MojoHandles().push_back(mojo::ScopedHandle());
  token = handle->token();
  return true;
}
void ExplorerHandleSerialization::Finalize(const Transferables& transferables,
                                           SerializedScriptValue& serialized,
                                           ExceptionState& exception) {
  auto* list =
      transferables.GetTransferListIfExists<ExplorerHandleTransferList>();
  if (!list) {
    return;
  }
  for (auto& handle : list->handles) {
    if (!handle->CanTransfer()) {
      exception.ThrowDOMException(
          DOMExceptionCode::kDataCloneError,
          "ExplorerHandle changed during serialization");
      return;
    }
  }
  for (wtf_size_t i = 0; i < list->handles.size(); ++i) {
    auto endpoints = list->handles[i]->TakeForTransfer();
    if (list->slots[i] != std::numeric_limits<uint32_t>::max()) {
      serialized.MojoHandles()[list->slots[i]] = std::move(endpoints);
    }
  }
}
ExplorerHandle* ExplorerHandleSerialization::Read(
    ExecutionContext* context,
    SerializedScriptValue& serialized,
    uint32_t slot,
    const String& token) {
  auto& handles = serialized.MojoHandles();
  if (!IsExposed(context) || slot >= handles.size() ||
      !handles[slot].is_valid()) {
    return nullptr;
  }
  auto* result = MakeGarbageCollected<ExplorerHandle>(context);
  result->InitializeTransferred(
      token,
      mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferControl>(
          mojo::ScopedMessagePipeHandle::From(std::move(handles[slot])), 0));
  return result;
}
}  // namespace blink
