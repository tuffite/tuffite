// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/renderer/explorer_surface_bindings.h"

#include <string>
#include <utility>

#include "base/functional/bind.h"
#include "base/task/single_thread_task_runner.h"
#include "base/time/time.h"
#include "content/public/renderer/render_frame.h"
#include "third_party/blink/public/platform/browser_interface_broker_proxy.h"
#include "third_party/blink/public/platform/scheduler/web_agent_group_scheduler.h"
#include "third_party/blink/public/web/web_local_frame.h"
#include "tuffite/components/explorer/renderer/blink/explorer_handle_bridge.h"
#include "v8/include/v8-context.h"
#include "v8/include/v8-external.h"
#include "v8/include/v8-function.h"
#include "v8/include/v8-isolate.h"
#include "v8/include/v8-microtask-queue.h"
#include "v8/include/v8-object.h"
#include "v8/include/v8-promise.h"
#include "v8/include/v8-script.h"

namespace tuffite {
namespace {

constexpr int32_t kMainWorldId = 0;
constexpr v8::ExternalPointerTypeTag kBindingsPointerTag =
    v8::kExternalPointerTypeTagDefault;

constexpr char kBootstrapScript[] = R"JS(
(() => {
  'use strict';
  const native = globalThis.__tuffite_surface__;
  if (!native) return;
  const root = globalThis.tuffite || new EventTarget();
  if (typeof root.addEventListener !== 'function' ||
      typeof root.removeEventListener !== 'function' ||
      typeof root.dispatchEvent !== 'function') return;
  const respondToDialog = native.respondToDialog;
  const requestAction = native.requestAction;
  const attachWindow = native.attachWindow;
  const pending = new Map();
  native.resetRequestSink((kind, key, canceled) => {
    const item = pending.get(`${kind}:${key}`);
    if (!item) return;
    item.finished = true;
    pending.delete(`${kind}:${key}`);
    if (canceled) item.controller.abort();
  });
  native.resetOpenSink((type, json) => {
    let value;
    try { value = JSON.parse(json); } catch { return; }
    const token = value.token;
    delete value.token;
    let detail = value;
    let item;
    if (type === "newwindow") {
      item = {finished: false, controller: new AbortController()};
      pending.set(`newwindow:${token}`, item);
      detail = {...value, signal: item.controller.signal,
        attach: target => {
          if (item.finished) return Promise.resolve(false);
          if (!(target instanceof HTMLExplorerElement)) return Promise.reject(new TypeError("Expected an explorer target"));
          item.finished = true;
          return attachWindow(token, target).then(() => true).catch(error => { requestAction("discard", token, false); throw error; });
        },
        discard: () => {
          if (item.finished) return Promise.resolve(false);
          item.finished = true;
          return requestAction("discard", token, false);
        },
      };
    }
    const event = new CustomEvent(type, {detail: Object.freeze(detail), cancelable: true, bubbles: true, composed: true});
    root.dispatchEvent(event);
    if (token) requestAction("open", token, !event.defaultPrevented && !item.finished);
  });
  native.resetDialogSink((requestId, type, message, defaultValue, isReload, from) => {
    const item = {finished: false, controller: new AbortController()};
    pending.set(`dialog:${requestId}`, item);
    const respond = (accept, promptText = '') => {
      if (item.finished) return Promise.resolve(false);
      item.finished = true;
      return respondToDialog(requestId, accept, String(promptText));
    };
    const detail = Object.freeze({
      type, message, defaultValue, isReload, from, signal: item.controller.signal,
      accept: (promptText = '') => respond(true, promptText),
      dismiss: () => respond(false),
    });
    const event = new CustomEvent("dialog", {detail, cancelable: true, bubbles: true, composed: true});
    root.dispatchEvent(event);
    requestAction("dialog", requestId, !event.defaultPrevented && !item.finished);
  });
  const addEventListener = root.addEventListener.bind(root);
  const removeEventListener = root.removeEventListener.bind(root);
  Object.defineProperties(root, {
    addEventListener: {value: (type, ...args) => type === 'message' || type === 'messageerror'
      ? window.addEventListener(type, ...args) : addEventListener(type, ...args)},
    removeEventListener: {value: (type, ...args) => type === 'message' || type === 'messageerror'
      ? window.removeEventListener(type, ...args) : removeEventListener(type, ...args)},
  });
  const postMessage = native.postMessage;
  Object.defineProperties(root, {
    postWindowMessage: { value: native.postWindowMessage, enumerable: false },
    postMessage: {
      value: postMessage, enumerable: true, configurable: false, writable: false,
    },
  });
  for (const key of [
    'attachWindow', 'postWindowMessage', 'resetOpenSink', 'resetDialogSink', 'respondToDialog', 'requestAction', 'resetRequestSink'
  ]) delete native[key];
  if (!globalThis.tuffite) {
    Object.defineProperty(globalThis, 'tuffite', {
      configurable: false, enumerable: true, writable: false, value: root,
    });
  }
  delete globalThis.__tuffite_surface__;
})();
)JS";

ExplorerSurfaceBindings* FromCallbackData(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  return info.Data()->IsExternal()
             ? static_cast<ExplorerSurfaceBindings*>(
                   info.Data().As<v8::External>()->Value(kBindingsPointerTag))
             : nullptr;
}

}  // namespace

// static
void ExplorerSurfaceBindings::Create(content::RenderFrame* frame) {
  new ExplorerSurfaceBindings(frame);
}

ExplorerSurfaceBindings::ExplorerSurfaceBindings(content::RenderFrame* frame)
    : content::RenderFrameObserver(frame) {}
ExplorerSurfaceBindings::~ExplorerSurfaceBindings() = default;

void ExplorerSurfaceBindings::DidCreateScriptContext(
    v8::Local<v8::Context> context,
    int32_t world_id) {
  if (world_id != kMainWorldId || !render_frame()->IsMainFrame()) {
    return;
  }
  v8::Isolate* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handles(isolate);
  v8::Context::Scope context_scope(context);
  v8::Local<v8::External> data =
      v8::External::New(isolate, this, kBindingsPointerTag);
  v8::Local<v8::Object> native = v8::Object::New(isolate);
  v8::Local<v8::Function> transfer_handle, attach_window;
  if (!v8::Function::New(context, &blink::PostExplorerWindowMessage)
           .ToLocal(&transfer_handle) ||
      !v8::Function::New(context, &blink::AttachExplorerWindow)
           .ToLocal(&attach_window) ||
      !native
           ->CreateDataProperty(
               context,
               v8::String::NewFromUtf8Literal(isolate, "postWindowMessage"),
               transfer_handle)
           .FromMaybe(false) ||
      !native
           ->CreateDataProperty(
               context, v8::String::NewFromUtf8Literal(isolate, "attachWindow"),
               attach_window)
           .FromMaybe(false)) {
    return;
  }
  v8::Local<v8::Function> request_action, reset_request_sink;
  v8::Local<v8::Function> post_message;
  v8::Local<v8::Function> reset_open_sink;
  v8::Local<v8::Function> reset_dialog_sink;
  v8::Local<v8::Function> respond_to_dialog;
  if (!v8::Function::New(context, &ExplorerSurfaceBindings::RequestAction, data)
           .ToLocal(&request_action) ||
      !v8::Function::New(context, &ExplorerSurfaceBindings::ResetRequestSink,
                         data)
           .ToLocal(&reset_request_sink) ||
      !native
           ->CreateDataProperty(
               context,
               v8::String::NewFromUtf8Literal(isolate, "requestAction"),
               request_action)
           .FromMaybe(false) ||
      !native
           ->CreateDataProperty(
               context,
               v8::String::NewFromUtf8Literal(isolate, "resetRequestSink"),
               reset_request_sink)
           .FromMaybe(false) ||
      !v8::Function::New(context, &blink::PostExplorerSurfaceMessage)
           .ToLocal(&post_message) ||
      !v8::Function::New(context, &ExplorerSurfaceBindings::ResetOpenSink, data)
           .ToLocal(&reset_open_sink) ||
      !v8::Function::New(context, &ExplorerSurfaceBindings::ResetDialogSink,
                         data)
           .ToLocal(&reset_dialog_sink) ||
      !v8::Function::New(context, &ExplorerSurfaceBindings::RespondToDialog,
                         data)
           .ToLocal(&respond_to_dialog) ||
      !native
           ->CreateDataProperty(
               context, v8::String::NewFromUtf8Literal(isolate, "postMessage"),
               post_message)
           .FromMaybe(false) ||
      !native
           ->CreateDataProperty(
               context,
               v8::String::NewFromUtf8Literal(isolate, "resetOpenSink"),
               reset_open_sink)
           .FromMaybe(false) ||
      !native
           ->CreateDataProperty(
               context,
               v8::String::NewFromUtf8Literal(isolate, "resetDialogSink"),
               reset_dialog_sink)
           .FromMaybe(false) ||
      !native
           ->CreateDataProperty(
               context,
               v8::String::NewFromUtf8Literal(isolate, "respondToDialog"),
               respond_to_dialog)
           .FromMaybe(false) ||
      !context->Global()
           ->CreateDataProperty(
               context,
               v8::String::NewFromUtf8Literal(isolate, "__tuffite_surface__"),
               native)
           .FromMaybe(false)) {
    return;
  }
  v8::Local<v8::String> source;
  v8::Local<v8::Script> script;
  if (!v8::String::NewFromUtf8(isolate, kBootstrapScript).ToLocal(&source) ||
      !v8::Script::Compile(context, source).ToLocal(&script) ||
      script->Run(context).IsEmpty()) {
    return;
  }
  context_.Reset(isolate, context);
  GetRemote().BindClient(receiver_.BindNewPipeAndPassRemote());
  std::vector<blink::FrameToken> ancestors;
  std::optional<blink::FramePolicy> policy;
  if (GetRemote().GetFrameAncestors(&ancestors, &policy)) {
    blink::SetExplorerFrameAncestors(render_frame()->GetWebFrame(), ancestors,
                                     policy);
  }
}

void ExplorerSurfaceBindings::WillReleaseScriptContext(
    v8::Local<v8::Context> context,
    int32_t world_id) {
  if (world_id == kMainWorldId) {
    Reset();
  }
}

void ExplorerSurfaceBindings::OnDestruct() {
  Reset();
  delete this;
}

// static
// static
void ExplorerSurfaceBindings::ResetOpenSink(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (ExplorerSurfaceBindings* bindings = FromCallbackData(info)) {
    bindings->ResetOpenSinkImpl(info);
  }
}

// static
void ExplorerSurfaceBindings::ResetDialogSink(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (ExplorerSurfaceBindings* bindings = FromCallbackData(info)) {
    bindings->ResetDialogSinkImpl(info);
  }
}

// static
void ExplorerSurfaceBindings::RespondToDialog(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (ExplorerSurfaceBindings* bindings = FromCallbackData(info)) {
    bindings->RespondToDialogImpl(info);
  }
}

void ExplorerSurfaceBindings::ResetRequestSink(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (auto* bindings = FromCallbackData(info);
      bindings && info.Length() == 1 && info[0]->IsFunction()) {
    bindings->request_sink_.Reset(info.GetIsolate(),
                                  info[0].As<v8::Function>());
  }
}
void ExplorerSurfaceBindings::RequestAction(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  auto* bindings = FromCallbackData(info);
  if (!bindings || info.Length() != 3 || !info[0]->IsString() ||
      !info[2]->IsBoolean()) {
    return;
  }
  v8::String::Utf8Value action(info.GetIsolate(), info[0]);
  bool propagate = info[2].As<v8::Boolean>()->Value();
  if (std::string_view(*action) == "dialog" && info[1]->IsBigInt()) {
    bindings->GetRemote().CompleteDialog(
        info[1].As<v8::BigInt>()->Uint64Value(), propagate);
    return;
  }
  if (!info[1]->IsString()) {
    return;
  }
  v8::String::Utf8Value key(info.GetIsolate(), info[1]);
  if (std::string_view(*action) == "open") {
    bindings->GetRemote().CompleteOpen(*key, propagate);
    return;
  }
  if (std::string_view(*action) != "discard") {
    return;
  }
  auto* isolate = info.GetIsolate();
  auto context = isolate->GetCurrentContext();
  auto resolver = v8::Promise::Resolver::New(context).ToLocalChecked();
  auto id = bindings->next_request_id_++;
  bindings->pending_posts_.emplace(
      id, PendingPost{v8::Global<v8::Context>(isolate, context),
                      v8::Global<v8::Promise::Resolver>(isolate, resolver)});
  info.GetReturnValue().Set(resolver->GetPromise());
  bindings->GetRemote().DiscardWindow(
      *key, base::BindOnce(&ExplorerSurfaceBindings::OnPosted,
                           bindings->weak_factory_.GetWeakPtr(), id));
}
void ExplorerSurfaceBindings::ResetOpenSinkImpl(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (info.Length() == 1 && info[0]->IsFunction()) {
    open_sink_.Reset(info.GetIsolate(), info[0].As<v8::Function>());
  }
}

void ExplorerSurfaceBindings::ResetDialogSinkImpl(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  if (info.Length() == 1 && info[0]->IsFunction()) {
    dialog_sink_.Reset(info.GetIsolate(), info[0].As<v8::Function>());
  }
}

void ExplorerSurfaceBindings::RespondToDialogImpl(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  v8::Isolate* isolate = info.GetIsolate();
  v8::Local<v8::Context> context = isolate->GetCurrentContext();
  if (info.Length() != 3 || !info[0]->IsBigInt() || !info[1]->IsBoolean() ||
      !info[2]->IsString()) {
    isolate->ThrowException(v8::Exception::TypeError(
        v8::String::NewFromUtf8Literal(isolate, "invalid dialog response")));
    return;
  }
  bool lossless = false;
  const uint64_t request_id = info[0].As<v8::BigInt>()->Uint64Value(&lossless);
  v8::String::Utf8Value prompt(isolate, info[2]);
  v8::Local<v8::Promise::Resolver> resolver;
  if (!lossless || !*prompt ||
      !v8::Promise::Resolver::New(context).ToLocal(&resolver)) {
    return;
  }
  if (pending_dialog_responses_.contains(request_id)) {
    resolver->Resolve(context, v8::Boolean::New(isolate, false))
        .FromMaybe(false);
    info.GetReturnValue().Set(resolver->GetPromise());
    return;
  }
  pending_dialog_responses_.emplace(
      request_id,
      PendingPost{v8::Global<v8::Context>(isolate, context),
                  v8::Global<v8::Promise::Resolver>(isolate, resolver)});
  info.GetReturnValue().Set(resolver->GetPromise());
  GetRemote().RespondToDialog(
      request_id, info[1].As<v8::Boolean>()->Value(),
      std::string(*prompt, prompt.length()),
      base::BindOnce(&ExplorerSurfaceBindings::OnDialogResponded,
                     weak_factory_.GetWeakPtr(), request_id));
}

void ExplorerSurfaceBindings::OnPosted(uint64_t request_id, bool accepted) {
  auto pending = pending_posts_.find(request_id);
  if (pending == pending_posts_.end()) {
    return;
  }
  v8::Isolate* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handles(isolate);
  v8::Local<v8::Context> context = pending->second.context.Get(isolate);
  v8::Context::Scope context_scope(context);
  v8::MicrotasksScope microtasks(context, v8::MicrotasksScope::kRunMicrotasks);
  pending->second.resolver.Get(isolate)
      ->Resolve(context, v8::Boolean::New(isolate, accepted))
      .ToChecked();
  pending_posts_.erase(pending);
}

void ExplorerSurfaceBindings::OnDialogResponded(uint64_t request_id,
                                                bool accepted) {
  auto pending = pending_dialog_responses_.find(request_id);
  if (pending == pending_dialog_responses_.end()) {
    return;
  }
  v8::Isolate* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handles(isolate);
  v8::Local<v8::Context> context = pending->second.context.Get(isolate);
  v8::Context::Scope context_scope(context);
  pending->second.resolver.Get(isolate)
      ->Resolve(context, v8::Boolean::New(isolate, accepted))
      .FromMaybe(false);
  pending_dialog_responses_.erase(pending);
}

void ExplorerSurfaceBindings::DidFinishRequest(const std::string& kind,
                                               const std::string& key,
                                               bool canceled) {
  if (context_.IsEmpty() || request_sink_.IsEmpty()) {
    return;
  }
  auto* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handles(isolate);
  auto context = context_.Get(isolate);
  v8::Context::Scope context_scope(context);
  v8::Local<v8::Value> args[] = {
      v8::String::NewFromUtf8(isolate, kind.c_str()).ToLocalChecked(),
      v8::String::NewFromUtf8(isolate, key.c_str()).ToLocalChecked(),
      v8::Boolean::New(isolate, canceled)};
  request_sink_.Get(isolate)
      ->Call(context, context->Global(), 3, args)
      .IsEmpty();
}
void ExplorerSurfaceBindings::DidChangeAncestors(
    const std::vector<blink::FrameToken>& ancestors,
    const std::optional<blink::FramePolicy>& policy,
    DidChangeAncestorsCallback callback) {
  ApplyAncestors(ancestors, policy, std::move(callback), 0);
}
void ExplorerSurfaceBindings::ApplyAncestors(
    std::vector<blink::FrameToken> ancestors,
    std::optional<blink::FramePolicy> policy,
    DidChangeAncestorsCallback callback,
    unsigned attempt) {
  if (blink::SetExplorerFrameAncestors(render_frame()->GetWebFrame(), ancestors,
                                       policy)) {
    std::move(callback).Run(true);
    return;
  }
  if (attempt >= 64) {
    std::move(callback).Run(false);
    return;
  }
  base::SingleThreadTaskRunner::GetCurrentDefault()->PostDelayedTask(
      FROM_HERE,
      base::BindOnce(&ExplorerSurfaceBindings::ApplyAncestors,
                     weak_factory_.GetWeakPtr(), std::move(ancestors),
                     std::move(policy), std::move(callback), attempt + 1),
      base::Milliseconds(1));
}
void ExplorerSurfaceBindings::DidReceiveOpen(const std::string& event_type,
                                             const std::string& request_json) {
  if (context_.IsEmpty() || open_sink_.IsEmpty()) {
    return;
  }
  v8::Isolate* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handles(isolate);
  v8::Local<v8::Context> context = context_.Get(isolate);
  v8::Context::Scope context_scope(context);
  v8::Local<v8::Value> args[] = {
      v8::String::NewFromUtf8(isolate, event_type.c_str()).ToLocalChecked(),
      v8::String::NewFromUtf8(isolate, request_json.c_str()).ToLocalChecked(),
  };
  open_sink_.Get(isolate)->Call(context, context->Global(), 2, args).IsEmpty();
}

void ExplorerSurfaceBindings::DidReceiveDialog(uint64_t request_id,
                                               const std::string& type,
                                               const std::string& message,
                                               const std::string& default_value,
                                               bool is_reload,
                                               const std::string& from) {
  if (context_.IsEmpty() || dialog_sink_.IsEmpty()) {
    return;
  }
  v8::Isolate* isolate =
      render_frame()->GetWebFrame()->GetAgentGroupScheduler()->Isolate();
  v8::HandleScope handles(isolate);
  v8::Local<v8::Context> context = context_.Get(isolate);
  v8::Context::Scope context_scope(context);
  v8::Local<v8::Value> args[] = {
      v8::BigInt::NewFromUnsigned(isolate, request_id),
      v8::String::NewFromUtf8(isolate, type.c_str()).ToLocalChecked(),
      v8::String::NewFromUtf8(isolate, message.c_str()).ToLocalChecked(),
      v8::String::NewFromUtf8(isolate, default_value.c_str()).ToLocalChecked(),
      v8::Boolean::New(isolate, is_reload),
      v8::String::NewFromUtf8(isolate, from.c_str()).ToLocalChecked(),
  };
  dialog_sink_.Get(isolate)
      ->Call(context, context->Global(), std::size(args), args)
      .IsEmpty();
}

mojom::ExplorerSurfaceHost& ExplorerSurfaceBindings::GetRemote() {
  if (!remote_) {
    render_frame()->GetBrowserInterfaceBroker().GetInterface(
        remote_.BindNewPipeAndPassReceiver());
  }
  return *remote_;
}

void ExplorerSurfaceBindings::Reset() {
  pending_posts_.clear();
  pending_dialog_responses_.clear();
  request_sink_.Reset();
  open_sink_.Reset();
  dialog_sink_.Reset();
  context_.Reset();
  receiver_.reset();
  remote_.reset();
}

}  // namespace tuffite
