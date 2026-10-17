// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/renderer/blink/explorer_handle.h"

#include "base/task/single_thread_task_runner.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_binding_for_core.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_handle.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_html_explorer_element.h"
#include "third_party/blink/renderer/core/dom/dom_exception.h"
#include "third_party/blink/renderer/core/dom/events/event.h"
#include "third_party/blink/renderer/core/execution_context/execution_context.h"
#include "third_party/blink/renderer/platform/wtf/functional.h"
#include "tuffite/components/explorer/renderer/blink/explorer_handle_bridge.h"
#include "tuffite/components/explorer/renderer/blink/html_explorer_element.h"

namespace blink {
ExplorerHandle::ExplorerHandle(ExecutionContext* context)
    : ExecutionContextLifecycleObserver(context),
      control_(context),
      client_receiver_(this, context) {}
const AtomicString& ExplorerHandle::InterfaceName() const {
  DEFINE_STATIC_LOCAL(AtomicString, name, ("ExplorerHandle"));
  return name;
}
ExecutionContext* ExplorerHandle::GetExecutionContext() const {
  return ExecutionContextLifecycleObserver::GetExecutionContext();
}
void ExplorerHandle::Trace(Visitor* visitor) const {
  visitor->Trace(control_);
  visitor->Trace(client_receiver_);
  visitor->Trace(close_resolver_);
  EventTarget::Trace(visitor);
  ExecutionContextLifecycleObserver::Trace(visitor);
}
mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferClient>
ExplorerHandle::BindClient() {
  return client_receiver_.BindNewPipeAndPassRemote(
      GetExecutionContext()->GetTaskRunner(TaskType::kInternalLoading));
}
void ExplorerHandle::Initialize(
    String token,
    mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferControl>
        control) {
  token_ = std::move(token);
  control_.Bind(std::move(control), GetExecutionContext()->GetTaskRunner(
                                        TaskType::kInternalLoading));
  control_->Accept();
  // State notifications and control responses use different pipes. Only the
  // ordered state pipe may declare transferred vs closed.
  client_receiver_.set_disconnect_handler(
      BindOnce(&ExplorerHandle::OnDisconnected, WrapWeakPersistent(this)));
}
void ExplorerHandle::StateChanged(const String& state) {
  if (state_ == state || state_ == "transferred" || state_ == "closed") {
    return;
  }
  state_ = state;
  if (state == "transferred" || state == "closed") {
    token_ = String();
  }
  if (GetExecutionContext() && !GetExecutionContext()->IsContextDestroyed()) {
    v8::HandleScope scope(GetExecutionContext()->GetIsolate());
    DispatchEvent(*Event::Create(AtomicString("statechange")));
  }
}
void ExplorerHandle::OnDisconnected() {
  if (state_ != "transferred") {
    StateChanged("closed");
  }
  control_.reset();
  if (close_resolver_) {
    close_resolver_->Resolve();
    close_resolver_.Clear();
  }
}
void ExplorerHandle::ContextDestroyed() {
  // Another same-origin document may retain this wrapper after its originating
  // document goes away. HeapMojo's context reset alone cannot update its state.
  OnDisconnected();
  client_receiver_.reset();
}
ScriptPromise<IDLUndefined> ExplorerHandle::close(ScriptState* script_state) {
  if (state_ == "transferred" || state_ == "closed") {
    return ToResolvedUndefinedPromise(script_state);
  }
  if (state_ == "attaching" || close_resolver_ || !control_.is_bound()) {
    return ScriptPromise<IDLUndefined>::RejectWithDOMException(
        script_state,
        MakeGarbageCollected<DOMException>(DOMExceptionCode::kInvalidStateError,
                                           "Explorer transfer is busy"));
  }
  close_resolver_ =
      MakeGarbageCollected<ScriptPromiseResolver<IDLUndefined>>(script_state);
  auto promise = close_resolver_->Promise();
  control_->Close(BindOnce(
      [](ExplorerHandle* transfer, const String& error) {
        if (!transfer->close_resolver_) {
          return;
        }
        auto* resolver = transfer->close_resolver_.Get();
        transfer->close_resolver_.Clear();
        if (!error.empty()) {
          resolver->RejectWithDOMException(DOMExceptionCode::kInvalidStateError,
                                           "Explorer transfer is busy");
          return;
        }
        transfer->StateChanged("closed");
        resolver->Resolve();
      },
      WrapPersistent(this)));
  return promise;
}
}  // namespace blink

namespace blink {
bool ExplorerHandle::BeginAttach() {
  if (state_ != "pending" || token_.empty()) {
    return false;
  }
  // Reserve without dispatching JS, which could remove the target reentrantly.
  state_ = "attaching";
  return true;
}
void ExplorerHandle::Consume() {
  StateChanged("transferred");
  control_.reset();
  client_receiver_.reset();
}
bool ExplorerHandle::CanTransfer() const {
  return state_ == "pending" && !token_.empty() && control_.is_bound() &&
         client_receiver_.is_bound() && !close_resolver_ &&
         GetExecutionContext() && !GetExecutionContext()->IsContextDestroyed();
}
mojo::ScopedHandle ExplorerHandle::TakeForTransfer() {
  CHECK(CanTransfer());
  control_->BeginTransfer();
  auto control = control_.Unbind();
  client_receiver_.reset();
  token_ = String();
  state_ = "transferred";
  GetExecutionContext()
      ->GetTaskRunner(TaskType::kInternalLoading)
      ->PostTask(
          FROM_HERE,
          BindOnce(
              [](ExplorerHandle* handle) {
                if (handle && handle->GetExecutionContext() &&
                    !handle->GetExecutionContext()->IsContextDestroyed()) {
                  v8::HandleScope scope(
                      handle->GetExecutionContext()->GetIsolate());
                  handle->DispatchEvent(
                      *Event::Create(AtomicString("statechange")));
                }
              },
              WrapWeakPersistent(this)));
  return mojo::ScopedHandle::From(control.PassPipe());
}
void ExplorerHandle::InitializeTransferred(
    String token,
    mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferControl>
        control) {
  auto client = BindClient();
  Initialize(std::move(token), std::move(control));
  control_->RebindClient(std::move(client));
}
void ExplorerHandle::RestorePending() {
  if (state_ == "attaching") {
    StateChanged("pending");
  }
}

void AttachExplorerWindow(const v8::FunctionCallbackInfo<v8::Value>& info) {
  auto* isolate = info.GetIsolate();
  auto* state = ScriptState::From(isolate, isolate->GetCurrentContext());
  auto* target = info.Length() > 1
                     ? V8HTMLExplorerElement::ToWrappable(isolate, info[1])
                     : nullptr;
  if (!target || !info[0]->IsString()) {
    isolate->ThrowException(v8::Exception::TypeError(
        v8::String::NewFromUtf8Literal(isolate, "Expected explorer target")));
    return;
  }
  v8::String::Utf8Value token(isolate, info[0]);
  info.GetReturnValue().Set(
      target->AttachToken(state, String::FromUtf8(*token)).V8Promise());
}
}  // namespace blink
