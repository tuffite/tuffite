// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/renderer/dispatch_bindings.h"

#include <algorithm>
#include <memory>
#include <string>
#include <string_view>
#include <utility>

#include "base/command_line.h"
#include "base/functional/bind.h"
#include "base/logging.h"
#include "base/strings/pattern.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_split.h"
#include "content/public/renderer/render_frame.h"
#include "third_party/blink/public/platform/browser_interface_broker_proxy.h"
#include "third_party/blink/public/platform/scheduler/web_agent_group_scheduler.h"
#include "third_party/blink/public/web/web_document.h"
#include "third_party/blink/public/web/web_local_frame.h"
#include "tuffite/components/dispatch/common/switches.h"
#include "url/gurl.h"
#include "url/origin.h"
#include "v8/include/v8-array-buffer.h"
#include "v8/include/v8-context.h"
#include "v8/include/v8-external.h"
#include "v8/include/v8-function.h"
#include "v8/include/v8-isolate.h"
#include "v8/include/v8-microtask-queue.h"
#include "v8/include/v8-object.h"
#include "v8/include/v8-promise.h"
#include "v8/include/v8-script.h"
#include "v8/include/v8-typed-array.h"

namespace tuffite {
namespace {

constexpr int32_t kMainWorldId = 0;
constexpr size_t kMaxPayloadBytes = 16 * 1024 * 1024;
// Cap on in-flight ShellAPI invocations per document. A well-behaved
// application spikes to at most a few dozen concurrent invocations during
// startup. Higher bounds are indistinguishable from a page trying to keep
// the browser process busy indefinitely, so we shed load with a TypeError
// once the ceiling is reached and let the page apply its own backpressure.
constexpr size_t kMaxPendingDispatches = 512;
constexpr v8::ExternalPointerTypeTag kBindingsPointerTag =
    v8::kExternalPointerTypeTagDefault;

constexpr char kBootstrapScript[] = R"JS(
(() => {
  'use strict';
  const native = globalThis.__tuffite__;
  if (!native) return;
  const root = globalThis.tuffite || new EventTarget();
  if (typeof root.addEventListener !== 'function' ||
      typeof root.removeEventListener !== 'function' ||
      typeof root.dispatchEvent !== 'function') return;
  for (const key of ['invoke', 'networkTestUrl']) {
    if (Reflect.has(native, key) && !Reflect.has(root, key)) {
      Object.defineProperty(root, key,
          Object.getOwnPropertyDescriptor(native, key));
    }
  }
  native.resetChannelSink((name, payload) => {
    root.dispatchEvent(new CustomEvent('channel', {
      detail: Object.freeze({channel: name, data: payload}),
    }));
  });
  if (!globalThis.tuffite) {
    Object.defineProperty(globalThis, 'tuffite', {
      configurable: false, enumerable: true, writable: false, value: root,
    });
  }
  delete globalThis.__tuffite__;
})();
)JS";

DispatchBindings* FromCallbackData(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (!info.Data()->IsExternal()) {
    return nullptr;
  }
  return static_cast<DispatchBindings*>(
      info.Data().As<v8::External>()->Value(kBindingsPointerTag));
}

v8::Local<v8::Uint8Array> ToUint8Array(v8::Isolate* isolate,
                                       base::span<const uint8_t> bytes) {
  v8::Local<v8::ArrayBuffer> buffer =
      v8::ArrayBuffer::New(isolate, bytes.size());
  if (!bytes.empty()) {
    auto backing = buffer->GetBackingStore();
    // SAFETY: V8 guarantees Data() addresses ByteLength() writable bytes for
    // the lifetime of the backing store.
    auto destination = UNSAFE_BUFFERS(base::span<uint8_t>(
        static_cast<uint8_t*>(backing->Data()), backing->ByteLength()));
    std::ranges::copy(bytes, destination.begin());
  }
  return v8::Uint8Array::New(buffer, 0, bytes.size());
}

bool IsOriginAllowed(const url::Origin& origin) {
  if (origin.opaque()) {
    return false;
  }
  const std::string allowlist =
      base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
          dispatch::kOriginsSwitch);
  for (const std::string& allowed : base::SplitString(
           allowlist, ",", base::TRIM_WHITESPACE, base::SPLIT_WANT_NONEMPTY)) {
    if (base::MatchPattern(origin.Serialize(), allowed)) {
      return true;
    }
  }
  return false;
}

}  // namespace

// static
void DispatchBindings::Create(content::RenderFrame* frame) {
  new DispatchBindings(frame);
}

DispatchBindings::DispatchBindings(content::RenderFrame* frame)
    : content::RenderFrameObserver(frame) {}

DispatchBindings::~DispatchBindings() = default;

void DispatchBindings::DidCreateScriptContext(v8::Local<v8::Context> context,
                                              int32_t world_id) {
  if (world_id != kMainWorldId || !render_frame()->IsMainFrame()) {
    return;
  }
  const GURL document_url(render_frame()->GetWebFrame()->GetDocument().Url());
  if (!IsOriginAllowed(url::Origin::Create(document_url))) {
    return;
  }

  v8::Isolate* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handle_scope(isolate);
  v8::Context::Scope context_scope(context);
  v8::Local<v8::External> data =
      v8::External::New(isolate, this, kBindingsPointerTag);
  v8::Local<v8::Object> native = v8::Object::New(isolate);
  v8::Local<v8::Function> invoke;
  v8::Local<v8::Function> reset_channel_sink;
  if (!v8::Function::New(context, &DispatchBindings::Dispatch, data)
           .ToLocal(&invoke) ||
      !v8::Function::New(context, &DispatchBindings::ResetChannelSink, data)
           .ToLocal(&reset_channel_sink) ||
      !native
           ->CreateDataProperty(
               context, v8::String::NewFromUtf8Literal(isolate, "invoke"),
               invoke)
           .FromMaybe(false) ||
      !native
           ->CreateDataProperty(
               context,
               v8::String::NewFromUtf8Literal(isolate, "resetChannelSink"),
               reset_channel_sink)
           .FromMaybe(false) ||
      !context->Global()
           ->CreateDataProperty(
               context, v8::String::NewFromUtf8Literal(isolate, "__tuffite__"),
               native)
           .FromMaybe(false)) {
    return;
  }
  const std::string network_test_url =
      base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
          dispatch::kNetworkTestUrlSwitch);
  if (!network_test_url.empty()) {
    v8::Local<v8::String> value;
    if (!v8::String::NewFromUtf8(isolate, network_test_url.c_str())
             .ToLocal(&value) ||
        !native
             ->CreateDataProperty(
                 context,
                 v8::String::NewFromUtf8Literal(isolate, "networkTestUrl"),
                 value)
             .FromMaybe(false)) {
      return;
    }
  }

  v8::Local<v8::String> source;
  v8::Local<v8::Script> script;
  if (!v8::String::NewFromUtf8(isolate, kBootstrapScript).ToLocal(&source) ||
      !v8::Script::Compile(context, source).ToLocal(&script) ||
      script->Run(context).IsEmpty()) {
    return;
  }
  context_.Reset(isolate, context);
  GetRemote();
  remote_->BindClient(receiver_.BindNewPipeAndPassRemote());
}

void DispatchBindings::WillReleaseScriptContext(v8::Local<v8::Context> context,
                                                int32_t world_id) {
  if (world_id == kMainWorldId && !context_.IsEmpty() &&
      context_.Get(
          render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate()) ==
          context) {
    ResetBridge("ShellAPI document was destroyed", false);
  }
}

void DispatchBindings::OnDestruct() {
  ResetBridge("ShellAPI frame was destroyed");
  delete this;
}

// static
void DispatchBindings::Dispatch(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (DispatchBindings* bindings = FromCallbackData(info)) {
    bindings->DispatchImpl(info);
  }
}

// static
void DispatchBindings::ResetChannelSink(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (DispatchBindings* bindings = FromCallbackData(info)) {
    bindings->ResetChannelSinkImpl(info);
  }
}

void DispatchBindings::DispatchImpl(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  v8::Isolate* isolate = info.GetIsolate();
  v8::Local<v8::Context> context = isolate->GetCurrentContext();
  if (info.Length() != 2 || !info[0]->IsString() || !info[1]->IsUint8Array()) {
    isolate->ThrowException(
        v8::Exception::TypeError(v8::String::NewFromUtf8Literal(
            isolate,
            "invoke(channel, payload) requires a string and Uint8Array")));
    return;
  }
  v8::String::Utf8Value channel_utf8(isolate, info[0]);
  if (!*channel_utf8 || channel_utf8.length() == 0) {
    isolate->ThrowException(v8::Exception::TypeError(
        v8::String::NewFromUtf8Literal(isolate, "channel must not be empty")));
    return;
  }
  v8::Local<v8::Uint8Array> array = info[1].As<v8::Uint8Array>();
  if (array->ByteLength() > kMaxPayloadBytes) {
    isolate->ThrowException(v8::Exception::RangeError(
        v8::String::NewFromUtf8Literal(isolate, "payload is too large")));
    return;
  }
  std::vector<uint8_t> payload(array->ByteLength());
  if (!payload.empty()) {
    array->CopyContents(payload.data(), payload.size());
  }
  v8::Local<v8::Promise::Resolver> resolver;
  if (!v8::Promise::Resolver::New(context).ToLocal(&resolver)) {
    return;
  }
  // Reject up front if the document has more pending dispatches than we are
  // willing to keep alive. Combined with the browser-side dispatch queue cap
  // this bounds the total in-flight work a single document can create.
  if (pending_dispatches_.size() >= kMaxPendingDispatches) {
    isolate->ThrowException(
        v8::Exception::RangeError(v8::String::NewFromUtf8Literal(
            isolate, "too many pending ShellAPI invocations")));
    return;
  }
  const uint64_t request_id = next_request_id_++;
  pending_dispatches_.emplace(
      request_id,
      PendingDispatch{v8::Global<v8::Context>(isolate, context),
                      v8::Global<v8::Promise::Resolver>(isolate, resolver)});
  info.GetReturnValue().Set(resolver->GetPromise());
  GetRemote().DispatchRequest(
      request_id, std::string(*channel_utf8, channel_utf8.length()),
      std::move(payload),
      base::BindOnce(&DispatchBindings::OnDispatched,
                     weak_factory_.GetWeakPtr(), request_id));
}

void DispatchBindings::ResetChannelSinkImpl(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (info.Length() != 1 || !info[0]->IsFunction()) {
    info.GetIsolate()->ThrowException(
        v8::Exception::TypeError(v8::String::NewFromUtf8Literal(
            info.GetIsolate(), "channel sink must be a function")));
    return;
  }
  channel_sink_.Reset(info.GetIsolate(), info[0].As<v8::Function>());
}

void DispatchBindings::OnDispatched(uint64_t request_id,
                                    int32_t status,
                                    const std::vector<uint8_t>& response) {
  auto pending = pending_dispatches_.find(request_id);
  if (pending == pending_dispatches_.end()) {
    return;
  }
  v8::Isolate* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handle_scope(isolate);
  v8::Local<v8::Context> context = pending->second.context.Get(isolate);
  v8::Context::Scope context_scope(context);
  v8::MicrotasksScope microtasks_scope(context,
                                       v8::MicrotasksScope::kRunMicrotasks);
  v8::Local<v8::Promise::Resolver> resolver =
      pending->second.resolver.Get(isolate);
  if (status == 0) {
    resolver->Resolve(context, ToUint8Array(isolate, response)).ToChecked();
  } else {
    const std::string detail(response.begin(), response.end());
    const std::string message = detail.empty()
                                    ? "ShellAPI transport unavailable (" +
                                          base::NumberToString(status) + ")"
                                    : detail;
    const char* code = "internal";
    switch (status) {
      case 1:
        code = "invalid_argument";
        break;
      case 2:
        code = "incompatible_abi";
        break;
      case 3:
        code = "invalid_handle";
        break;
      case 4:
        code = "unavailable";
        break;
      case 5:
        code = "resource_exhausted";
        break;
    }
    v8::Local<v8::Value> exception = v8::Exception::Error(
        v8::String::NewFromUtf8(isolate, message.c_str()).ToLocalChecked());
    v8::Local<v8::Object> error = exception.As<v8::Object>();
    error
        ->Set(context, v8::String::NewFromUtf8Literal(isolate, "code"),
              v8::String::NewFromUtf8(isolate, code).ToLocalChecked())
        .ToChecked();
    error
        ->Set(context, v8::String::NewFromUtf8Literal(isolate, "status"),
              v8::Integer::New(isolate, status))
        .ToChecked();
    resolver->Reject(context, exception).ToChecked();
  }
  pending_dispatches_.erase(pending);
}

void DispatchBindings::DispatchChannelEvent(
    const std::string& channel,
    const std::vector<uint8_t>& payload) {
  if (context_.IsEmpty() || channel_sink_.IsEmpty()) {
    return;
  }
  v8::Isolate* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handle_scope(isolate);
  v8::Local<v8::Context> context = context_.Get(isolate);
  v8::Context::Scope context_scope(context);
  v8::MicrotasksScope microtasks_scope(context,
                                       v8::MicrotasksScope::kRunMicrotasks);
  v8::Local<v8::Value> arguments[] = {
      v8::String::NewFromUtf8(isolate, channel.c_str()).ToLocalChecked(),
      ToUint8Array(isolate, payload),
  };
  channel_sink_.Get(isolate)
      ->Call(context, context->Global(), std::size(arguments), arguments)
      .IsEmpty();
}

mojom::Dispatcher& DispatchBindings::GetRemote() {
  if (!remote_) {
    render_frame()->GetBrowserInterfaceBroker().GetInterface(
        remote_.BindNewPipeAndPassReceiver());
    remote_.set_disconnect_handler(base::BindOnce(
        &DispatchBindings::OnRemoteDisconnected, weak_factory_.GetWeakPtr()));
  }
  return *remote_;
}

void DispatchBindings::OnRemoteDisconnected() {
  ResetBridge("ShellAPI connection closed");
}

void DispatchBindings::ResetBridge(const std::string& reason,
                                   bool reject_pending) {
  // WillReleaseScriptContext runs while Blink forbids entering V8. Rejecting
  // promises with a kRunMicrotasks scope there crashes the renderer. Once a
  // document is being destroyed no script can observe those rejections, so
  // release the persistent handles without executing JavaScript.
  if (reject_pending && !context_.IsEmpty()) {
    v8::Isolate* isolate =
        render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
    v8::HandleScope handle_scope(isolate);
    v8::Local<v8::Context> context = context_.Get(isolate);
    v8::Context::Scope context_scope(context);
    v8::MicrotasksScope microtasks_scope(context,
                                         v8::MicrotasksScope::kRunMicrotasks);
    v8::Local<v8::Value> error = v8::Exception::Error(
        v8::String::NewFromUtf8(isolate, reason.c_str()).ToLocalChecked());
    for (auto& [request_id, pending] : pending_dispatches_) {
      pending.resolver.Get(isolate)->Reject(context, error).FromMaybe(false);
    }
  }
  pending_dispatches_.clear();
  channel_sink_.Reset();
  context_.Reset();
  receiver_.reset();
  remote_.reset();
}

}  // namespace tuffite
