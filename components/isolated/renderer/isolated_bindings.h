// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_ISOLATED_RENDERER_ISOLATED_BINDINGS_H_
#define TUFFITE_COMPONENTS_ISOLATED_RENDERER_ISOLATED_BINDINGS_H_
#include <map>

#include "base/memory/weak_ptr.h"
#include "content/public/renderer/render_frame_observer.h"
#include "content/public/renderer/render_frame_observer_tracker.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/isolated/common/isolated.mojom.h"
#include "v8/include/v8.h"
namespace tuffite {
class IsolatedBindings final
    : public content::RenderFrameObserver,
      public content::RenderFrameObserverTracker<IsolatedBindings> {
 public:
  static void Create(content::RenderFrame*);
  void DidCreateScriptContext(v8::Local<v8::Context>, int32_t) override;
  void WillReleaseScriptContext(v8::Local<v8::Context>, int32_t) override;
  void OnDestruct() override;

 private:
  explicit IsolatedBindings(content::RenderFrame*);
  ~IsolatedBindings() override;
  static void Load(const v8::FunctionCallbackInfo<v8::Value>&);
  void OnLoaded(int, const std::optional<std::string>&);
  struct Pending {
    v8::Global<v8::Context> context;
    v8::Global<v8::Promise::Resolver> resolver;
  };
  std::map<int, Pending> pending_;
  int next_request_ = 0;
  mojo::Remote<mojom::IsolatedHost> remote_;
  base::WeakPtrFactory<IsolatedBindings> weak_{this};
};
}  // namespace tuffite
#endif
