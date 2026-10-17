// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_RENDERER_EXPLORER_SURFACE_BINDINGS_H_
#define TUFFITE_COMPONENTS_EXPLORER_RENDERER_EXPLORER_SURFACE_BINDINGS_H_

#include <string>
#include <unordered_map>

#include "base/memory/weak_ptr.h"
#include "content/public/renderer/render_frame_observer.h"
#include "mojo/public/cpp/bindings/receiver.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/explorer/common/explorer.mojom.h"
#include "v8/include/v8-persistent-handle.h"
#include "v8/include/v8-promise.h"

namespace tuffite {

class ExplorerSurfaceBindings final : public content::RenderFrameObserver,
                                      public mojom::ExplorerSurfaceClient {
 public:
  static void Create(content::RenderFrame* frame);

  void DidCreateScriptContext(v8::Local<v8::Context> context,
                              int32_t world_id) override;
  void WillReleaseScriptContext(v8::Local<v8::Context> context,
                                int32_t world_id) override;
  void OnDestruct() override;
  void DidFinishRequest(const std::string&, const std::string&, bool) override;
  void DidChangeAncestors(const std::vector<blink::FrameToken>&,
                          const std::optional<blink::FramePolicy>&,
                          DidChangeAncestorsCallback) override;
  void DidReceiveOpen(const std::string& event_type,
                      const std::string& request_json) override;
  void DidReceiveDialog(uint64_t request_id,
                        const std::string& type,
                        const std::string& message,
                        const std::string& default_value,
                        bool is_reload,
                        const std::string& from) override;

 private:
  struct PendingPost {
    v8::Global<v8::Context> context;
    v8::Global<v8::Promise::Resolver> resolver;
  };

  explicit ExplorerSurfaceBindings(content::RenderFrame* frame);
  ~ExplorerSurfaceBindings() override;

  static void RequestAction(const v8::FunctionCallbackInfo<v8::Value>&);
  static void ResetRequestSink(const v8::FunctionCallbackInfo<v8::Value>&);
  static void ResetOpenSink(const v8::FunctionCallbackInfo<v8::Value>& info);
  static void ResetDialogSink(const v8::FunctionCallbackInfo<v8::Value>& info);
  static void RespondToDialog(const v8::FunctionCallbackInfo<v8::Value>& info);
  void ApplyAncestors(std::vector<blink::FrameToken>,
                      std::optional<blink::FramePolicy>,
                      DidChangeAncestorsCallback,
                      unsigned attempt);
  void ResetOpenSinkImpl(const v8::FunctionCallbackInfo<v8::Value>& info);
  void ResetDialogSinkImpl(const v8::FunctionCallbackInfo<v8::Value>& info);
  void RespondToDialogImpl(const v8::FunctionCallbackInfo<v8::Value>& info);
  void OnPosted(uint64_t request_id, bool accepted);
  void OnDialogResponded(uint64_t request_id, bool accepted);
  void Reset();
  mojom::ExplorerSurfaceHost& GetRemote();

  mojo::Remote<mojom::ExplorerSurfaceHost> remote_;
  mojo::Receiver<mojom::ExplorerSurfaceClient> receiver_{this};
  v8::Global<v8::Context> context_;
  v8::Global<v8::Function> request_sink_;
  v8::Global<v8::Function> open_sink_;
  v8::Global<v8::Function> dialog_sink_;
  std::unordered_map<uint64_t, PendingPost> pending_posts_;
  std::unordered_map<uint64_t, PendingPost> pending_dialog_responses_;
  uint64_t next_request_id_ = 1;
  base::WeakPtrFactory<ExplorerSurfaceBindings> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_RENDERER_EXPLORER_SURFACE_BINDINGS_H_
