// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/renderer/blink/explorer_transfer.h"

#include "base/task/single_thread_task_runner.h"
#include "third_party/blink/renderer/core/dom/dom_exception.h"
#include "third_party/blink/renderer/core/dom/events/event.h"
#include "third_party/blink/renderer/core/execution_context/execution_context.h"
#include "third_party/blink/renderer/platform/wtf/functional.h"

namespace blink {
ExplorerTransfer::ExplorerTransfer(ExecutionContext* context)
    : ExecutionContextLifecycleObserver(context),
      control_(context),
      client_receiver_(this, context) {}
const AtomicString& ExplorerTransfer::InterfaceName() const {
  DEFINE_STATIC_LOCAL(AtomicString, name, ("ExplorerTransfer"));
  return name;
}
ExecutionContext* ExplorerTransfer::GetExecutionContext() const {
  return ExecutionContextLifecycleObserver::GetExecutionContext();
}
void ExplorerTransfer::Trace(Visitor* visitor) const {
  visitor->Trace(control_);
  visitor->Trace(client_receiver_);
  visitor->Trace(close_resolver_);
  EventTarget::Trace(visitor);
  ExecutionContextLifecycleObserver::Trace(visitor);
}
mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferClient>
ExplorerTransfer::BindClient() {
  return client_receiver_.BindNewPipeAndPassRemote(
      GetExecutionContext()->GetTaskRunner(TaskType::kInternalLoading));
}
void ExplorerTransfer::Initialize(
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
      BindOnce(&ExplorerTransfer::OnDisconnected, WrapWeakPersistent(this)));
}
void ExplorerTransfer::StateChanged(const String& state) {
  if (state_ == state || state_ == "transferred" || state_ == "closed") {
    return;
  }
  state_ = state;
  if (GetExecutionContext() && !GetExecutionContext()->IsContextDestroyed()) {
    DispatchEvent(*Event::Create(AtomicString("statechange")));
  }
}
void ExplorerTransfer::OnDisconnected() {
  if (state_ != "transferred") {
    StateChanged("closed");
  }
  control_.reset();
  if (close_resolver_) {
    close_resolver_->Resolve();
    close_resolver_.Clear();
  }
}
void ExplorerTransfer::ContextDestroyed() {
  // Another same-origin document may retain this wrapper after its originating
  // document goes away. HeapMojo's context reset alone cannot update its state.
  OnDisconnected();
  client_receiver_.reset();
}
ScriptPromise<IDLUndefined> ExplorerTransfer::close(ScriptState* script_state) {
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
      [](ExplorerTransfer* transfer, const String& error) {
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
