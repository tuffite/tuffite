// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_DISPATCH_RENDERER_DISPATCH_BINDINGS_H_
#define TUFFITE_COMPONENTS_DISPATCH_RENDERER_DISPATCH_BINDINGS_H_
#include "content/public/renderer/render_frame_observer.h"
#include "mojo/public/cpp/bindings/receiver.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/dispatch/common/dispatcher.mojom.h"
#include "v8/include/v8-persistent-handle.h"
namespace tuffite {
class DispatchBindings final : public content::RenderFrameObserver,
                               public mojom::DispatchClient {
 public:
  static void Create(content::RenderFrame*);
  void DidCreateScriptContext(v8::Local<v8::Context>, int32_t) override;
  void WillReleaseScriptContext(v8::Local<v8::Context>, int32_t) override;
  void OnDestruct() override;
  void DispatchChannelEvent(const std::string&,
                            const std::vector<uint8_t>&) override;

 private:
  explicit DispatchBindings(content::RenderFrame*);
  ~DispatchBindings() override;
  void Reset();
  mojo::Remote<mojom::EventSource> events_;
  mojo::Receiver<mojom::DispatchClient> receiver_{this};
  v8::Global<v8::Context> context_;
  v8::Global<v8::Function> channel_sink_;
};
}  // namespace tuffite
#endif
