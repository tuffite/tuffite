// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/dispatch/renderer/dispatch_bindings.h"

#include <algorithm>

#include "base/compiler_specific.h"
#include "content/public/renderer/render_frame.h"
#include "third_party/blink/public/mojom/browser_interface_broker.mojom.h"
#include "third_party/blink/public/platform/browser_interface_broker_proxy.h"
#include "third_party/blink/public/platform/scheduler/web_agent_group_scheduler.h"
#include "third_party/blink/public/platform/web_url.h"
#include "third_party/blink/public/web/web_document.h"
#include "third_party/blink/public/web/web_local_frame.h"
#include "third_party/blink/public/web/web_v8_features.h"
#include "tuffite/components/manifest/renderer/blink/document_configuration_reader.h"
#include "v8/include/v8-array-buffer.h"
#include "v8/include/v8-context.h"
#include "v8/include/v8-function.h"
#include "v8/include/v8-isolate.h"
#include "v8/include/v8-microtask-queue.h"
#include "v8/include/v8-script.h"
#include "v8/include/v8-typed-array.h"
namespace tuffite {
void DispatchBindings::Create(content::RenderFrame* frame) {
  new DispatchBindings(frame);
}
DispatchBindings::DispatchBindings(content::RenderFrame* frame)
    : RenderFrameObserver(frame) {}
DispatchBindings::~DispatchBindings() = default;
void DispatchBindings::DidCreateScriptContext(v8::Local<v8::Context> context,
                                              int32_t world) {
  if (world != 0) {
    return;
  }
  if (!render_frame()->IsMainFrame() || !IsShellApiExposed(context)) {
    const auto url = render_frame()->GetWebFrame()->GetDocument().Url();
    if (!url.ProtocolIs("devtools") && !url.ProtocolIs("chrome") &&
        !url.ProtocolIs("chrome-untrusted")) {
      blink::WebV8Features::EnableMojoJS(context, false);
    }
    return;
  }
  auto* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope scope(isolate);
  v8::Context::Scope context_scope(context);
  constexpr char source[] = R"JS((()=>{
    if (!globalThis.tuffite) Object.defineProperty(globalThis,'tuffite',{
      value:new EventTarget(),enumerable:true,configurable:false,writable:false});
    return (name,payload)=>globalThis.tuffite.dispatchEvent(new CustomEvent('channel',{
      detail:Object.freeze({channel:name,data:payload})}));
  })())JS";
  v8::Local<v8::String> text;
  v8::Local<v8::Script> script;
  v8::Local<v8::Value> sink;
  if (!v8::String::NewFromUtf8(isolate, source).ToLocal(&text) ||
      !v8::Script::Compile(context, text).ToLocal(&script) ||
      !script->Run(context).ToLocal(&sink) || !sink->IsFunction()) {
    return;
  }
  context_.Reset(isolate, context);
  channel_sink_.Reset(isolate, sink.As<v8::Function>());
  render_frame()->GetBrowserInterfaceBroker().GetInterface(
      events_.BindNewPipeAndPassReceiver());
  events_->BindClient(receiver_.BindNewPipeAndPassRemote());
}
void DispatchBindings::WillReleaseScriptContext(v8::Local<v8::Context> context,
                                                int32_t world) {
  if (world == 0 && !context_.IsEmpty() &&
      context_.Get(
          render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate()) ==
          context) {
    Reset();
  }
}
void DispatchBindings::OnDestruct() {
  Reset();
  delete this;
}
void DispatchBindings::Reset() {
  receiver_.reset();
  events_.reset();
  channel_sink_.Reset();
  context_.Reset();
}
void DispatchBindings::DispatchChannelEvent(
    const std::string& channel,
    const std::vector<uint8_t>& payload) {
  if (context_.IsEmpty() || channel_sink_.IsEmpty() ||
      payload.size() > 16 * 1024 * 1024) {
    return;
  }
  auto* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope scope(isolate);
  auto context = context_.Get(isolate);
  v8::Context::Scope context_scope(context);
  v8::MicrotasksScope microtasks_scope(context,
                                       v8::MicrotasksScope::kRunMicrotasks);
  v8::Local<v8::String> name;
  if (!v8::String::NewFromUtf8(isolate, channel.c_str()).ToLocal(&name)) {
    return;
  }
  auto buffer = v8::ArrayBuffer::New(isolate, payload.size());
  if (!payload.empty()) {
    // SAFETY: V8 owns the allocated backing store with the declared size.
    auto output = UNSAFE_BUFFERS(
        base::span(static_cast<uint8_t*>(buffer->GetBackingStore()->Data()),
                   payload.size()));
    output.copy_from(payload);
  }
  v8::Local<v8::Value> args[] = {
      name, v8::Uint8Array::New(buffer, 0, payload.size())};
  auto result =
      channel_sink_.Get(isolate)->Call(context, context->Global(), 2, args);
  (void)result;
}
}  // namespace tuffite
