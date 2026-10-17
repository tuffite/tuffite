// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_HANDLE_H_
#define TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_HANDLE_H_
#include "third_party/blink/renderer/bindings/core/v8/script_promise_resolver.h"
#include "third_party/blink/renderer/core/dom/events/event_target.h"
#include "third_party/blink/renderer/core/execution_context/execution_context_lifecycle_observer.h"
#include "third_party/blink/renderer/platform/mojo/heap_mojo_receiver.h"
#include "third_party/blink/renderer/platform/mojo/heap_mojo_remote.h"
#include "tuffite/components/explorer/common/explorer.mojom-blink.h"

namespace blink {
class CORE_EXPORT ExplorerHandle final
    : public EventTarget,
      public ExecutionContextLifecycleObserver,
      public tuffite::mojom::blink::ExplorerTransferClient {
  DEFINE_WRAPPERTYPEINFO();

 public:
  explicit ExplorerHandle(ExecutionContext*);
  const String& token() const { return token_; }
  const String& state() const { return state_; }
  ScriptPromise<IDLUndefined> close(ScriptState*);
  bool CanTransfer() const;
  mojo::ScopedHandle TakeForTransfer();
  void InitializeTransferred(
      String,
      mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferControl>);
  bool BeginAttach();
  void Consume();
  void RestorePending();
  EventListener* onstatechange() {
    return GetAttributeEventListener(AtomicString("statechange"));
  }
  void setOnstatechange(EventListener* listener) {
    SetAttributeEventListener(AtomicString("statechange"), listener);
  }
  mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferClient>
  BindClient();
  void Initialize(
      String token,
      mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferControl>
          control);
  void StateChanged(const String& state) override;
  const AtomicString& InterfaceName() const override;
  ExecutionContext* GetExecutionContext() const override;
  void Trace(Visitor*) const override;

 private:
  void ContextDestroyed() override;
  void OnDisconnected();
  String token_;
  String state_ = "pending";
  HeapMojoRemote<tuffite::mojom::blink::ExplorerTransferControl> control_;
  HeapMojoReceiver<tuffite::mojom::blink::ExplorerTransferClient,
                   ExplorerHandle>
      client_receiver_;
  Member<ScriptPromiseResolver<IDLUndefined>> close_resolver_;
};
}  // namespace blink
#endif  // TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_HANDLE_H_
