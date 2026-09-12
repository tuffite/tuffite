// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_RENDERER_DISPATCH_BINDINGS_H_
#define TUFFITE_COMPONENTS_DISPATCH_RENDERER_DISPATCH_BINDINGS_H_

#include <string>
#include <unordered_map>
#include <vector>

#include "base/memory/weak_ptr.h"
#include "content/public/renderer/render_frame_observer.h"
#include "mojo/public/cpp/bindings/receiver.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/dispatch/common/dispatcher.mojom.h"
#include "v8/include/v8-persistent-handle.h"
#include "v8/include/v8-promise.h"

namespace tuffite {

class DispatchBindings final : public content::RenderFrameObserver,
                               public mojom::DispatchClient {
 public:
  static void Create(content::RenderFrame* frame);

  void DidCreateScriptContext(v8::Local<v8::Context> context,
                              int32_t world_id) override;
  void WillReleaseScriptContext(v8::Local<v8::Context> context,
                                int32_t world_id) override;
  void OnDestruct() override;
  void DispatchChannelEvent(const std::string& channel,
                            const std::vector<uint8_t>& payload) override;

 private:
  struct PendingDispatch {
    v8::Global<v8::Context> context;
    v8::Global<v8::Promise::Resolver> resolver;
  };

  explicit DispatchBindings(content::RenderFrame* frame);
  ~DispatchBindings() override;

  static void Dispatch(const v8::FunctionCallbackInfo<v8::Value>& info);
  static void ResetChannelSink(const v8::FunctionCallbackInfo<v8::Value>& info);
  void DispatchImpl(const v8::FunctionCallbackInfo<v8::Value>& info);
  void ResetChannelSinkImpl(const v8::FunctionCallbackInfo<v8::Value>& info);
  void OnDispatched(uint64_t request_id,
                    int32_t status,
                    const std::vector<uint8_t>& response);
  void OnRemoteDisconnected();
  void ResetBridge(const std::string& reason, bool reject_pending = true);
  mojom::Dispatcher& GetRemote();

  mojo::Remote<mojom::Dispatcher> remote_;
  mojo::Receiver<mojom::DispatchClient> receiver_{this};
  v8::Global<v8::Context> context_;
  v8::Global<v8::Function> channel_sink_;
  std::unordered_map<uint64_t, PendingDispatch> pending_dispatches_;
  uint64_t next_request_id_ = 1;
  base::WeakPtrFactory<DispatchBindings> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_DISPATCH_RENDERER_DISPATCH_BINDINGS_H_
