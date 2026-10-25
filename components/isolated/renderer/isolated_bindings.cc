// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/isolated/renderer/isolated_bindings.h"

#include <tuple>

#include "base/functional/bind.h"
#include "content/public/renderer/render_frame.h"
#include "third_party/blink/public/platform/browser_interface_broker_proxy.h"
#include "third_party/blink/public/platform/scheduler/web_agent_group_scheduler.h"
#include "third_party/blink/public/web/web_local_frame.h"
#include "tuffite/components/isolated/renderer/blink/isolated_bridge.h"
namespace tuffite {
void IsolatedBindings::Create(content::RenderFrame* frame) {
  new IsolatedBindings(frame);
}
IsolatedBindings::IsolatedBindings(content::RenderFrame* frame)
    : content::RenderFrameObserver(frame),
      content::RenderFrameObserverTracker<IsolatedBindings>(frame) {}
IsolatedBindings::~IsolatedBindings() = default;
void IsolatedBindings::OnDestruct() {
  delete this;
}
void IsolatedBindings::DidCreateScriptContext(v8::Local<v8::Context> context,
                                              int32_t world_id) {
  if (world_id || !render_frame()->IsMainFrame()) {
    return;
  }
  auto* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handles(isolate);
  v8::Context::Scope scope(context);
  auto function = v8::Function::New(context, Load).ToLocalChecked();
  blink::InstallIsolatedHost(context, function);
}
void IsolatedBindings::WillReleaseScriptContext(v8::Local<v8::Context>,
                                                int32_t world_id) {
  if (world_id) {
    return;
  }
  weak_.InvalidateWeakPtrs();
  pending_.clear();
  remote_.reset();
}
void IsolatedBindings::Load(const v8::FunctionCallbackInfo<v8::Value>& info) {
  auto* isolate = info.GetIsolate();
  auto context = isolate->GetCurrentContext();
  auto* frame = blink::WebLocalFrame::FrameForContext(context);
  auto* self = frame ? Get(content::RenderFrame::FromWebFrame(frame)) : nullptr;
  if (!self) {
    isolate->ThrowException(v8::Exception::Error(
        v8::String::NewFromUtf8Literal(isolate, "Document is gone")));
    return;
  }
  if (info.Length() != 1 || !info[0]->IsString() ||
      self->pending_.size() >= 32) {
    isolate->ThrowException(v8::Exception::TypeError(
        v8::String::NewFromUtf8Literal(isolate, "Invalid world lookup")));
    return;
  }
  auto resolver = v8::Promise::Resolver::New(context).ToLocalChecked();
  info.GetReturnValue().Set(resolver->GetPromise());
  const int id = ++self->next_request_;
  self->pending_.emplace(
      id, Pending{v8::Global<v8::Context>(isolate, context),
                  v8::Global<v8::Promise::Resolver>(isolate, resolver)});
  if (!self->remote_) {
    self->render_frame()->GetBrowserInterfaceBroker().GetInterface(
        self->remote_.BindNewPipeAndPassReceiver());
    self->remote_.set_disconnect_handler(base::BindOnce(
        [](base::WeakPtr<IsolatedBindings> bindings) {
          if (!bindings) {
            return;
          }
          while (!bindings->pending_.empty()) {
            bindings->OnLoaded(bindings->pending_.begin()->first, std::nullopt);
          }
          bindings->remote_.reset();
        },
        self->weak_.GetWeakPtr()));
  }
  v8::String::Utf8Value name(isolate, info[0]);
  self->remote_->Load(std::string(*name, name.length()),
                      base::BindOnce(&IsolatedBindings::OnLoaded,
                                     self->weak_.GetWeakPtr(), id));
}
void IsolatedBindings::OnLoaded(int id,
                                const std::optional<std::string>& source) {
  auto it = pending_.find(id);
  if (it == pending_.end()) {
    return;
  }
  auto pending = std::move(it->second);
  pending_.erase(it);
  auto* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handles(isolate);
  auto context = pending.context.Get(isolate);
  v8::Context::Scope scope(context);
  auto resolver = pending.resolver.Get(isolate);
  if (source) {
    std::ignore = resolver->Resolve(
        context, v8::String::NewFromUtf8(isolate, source->data(),
                                         v8::NewStringType::kNormal,
                                         static_cast<int>(source->size()))
                     .ToLocalChecked());
  } else {
    std::ignore = resolver->Reject(
        context,
        v8::Exception::Error(v8::String::NewFromUtf8Literal(
            isolate, "Isolated world package is unavailable or denied")));
  }
}
}  // namespace tuffite
