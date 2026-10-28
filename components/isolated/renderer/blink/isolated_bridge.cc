// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/isolated/renderer/blink/isolated_bridge.h"

#include <array>
#include <atomic>

#include "third_party/blink/public/platform/web_isolated_world_info.h"
#include "third_party/blink/public/platform/web_security_origin.h"
#include "third_party/blink/renderer/bindings/core/v8/local_window_proxy.h"
#include "third_party/blink/renderer/bindings/core/v8/to_v8_traits.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_binding_for_core.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_html_element.h"
#include "third_party/blink/renderer/core/dom/document.h"
#include "third_party/blink/renderer/core/frame/local_dom_window.h"
#include "third_party/blink/renderer/core/frame/local_frame.h"
#include "third_party/blink/renderer/core/html/custom/custom_element_registry.h"
#include "third_party/blink/renderer/core/html/html_element.h"
#include "third_party/blink/renderer/platform/bindings/dom_wrapper_world.h"
#include "third_party/blink/renderer/platform/bindings/trace_wrapper_v8_reference.h"
#include "third_party/blink/renderer/platform/heap/collection_support/heap_hash_map.h"
#include "third_party/blink/renderer/platform/supplementable.h"
#include "tuffite/components/isolated/renderer/blink/isolated_scripts.h"
#include "v8/include/v8.h"

namespace blink {
namespace {
v8::Local<v8::String> Text(v8::Isolate* isolate, const char* text) {
  return v8::String::NewFromUtf8(isolate, text).ToLocalChecked();
}

class World final : public GarbageCollected<World> {
 public:
  Member<ScriptState> state;
  TraceWrapperV8Reference<v8::Value> operations;
  void Trace(Visitor* visitor) const {
    visitor->Trace(state);
    visitor->Trace(operations);
  }
};

class Worlds final : public GarbageCollected<Worlds>,
                     public Supplement<LocalDOMWindow> {
 public:
  static const char kSupplementName[];
  explicit Worlds(LocalDOMWindow& window) : Supplement(window) {}
  static Worlds& From(LocalDOMWindow& window) {
    auto* value = Supplement<LocalDOMWindow>::From<Worlds>(window);
    if (!value) {
      value = MakeGarbageCollected<Worlds>(window);
      ProvideTo(window, value);
    }
    return *value;
  }
  void Trace(Visitor* visitor) const override {
    visitor->Trace(worlds);
    visitor->Trace(host);
    visitor->Trace(sink);
    Supplement<LocalDOMWindow>::Trace(visitor);
  }
  HeapHashMap<int, Member<World>> worlds;
  Member<ScriptState> host;
  TraceWrapperV8Reference<v8::Value> sink;
};
const char Worlds::kSupplementName[] = "TuffiteIsolatedWorlds";

v8::MaybeLocal<v8::Value> Evaluate(ScriptState* state, const char* source) {
  v8::Local<v8::Script> script;
  if (!v8::Script::Compile(state->GetContext(),
                           Text(state->GetIsolate(), source))
           .ToLocal(&script)) {
    return {};
  }
  return script->Run(state->GetContext());
}

// Entry completion crosses realms as a primitive ID or a message-only error.
void ModuleCompleted(const v8::FunctionCallbackInfo<v8::Value>& info) {
  auto* isolate = info.GetIsolate();
  auto* state = ScriptState::ForCurrentRealm(isolate);
  auto* window = LocalDOMWindow::From(state);
  if (!window || !window->GetFrame()) {
    return;
  }
  auto& manager = Worlds::From(*window);
  auto data = info.Data().As<v8::Array>();
  auto context = state->GetContext();
  const int id =
      data->Get(context, 1).ToLocalChecked().As<v8::Int32>()->Value();
  const bool success = data->Get(context, 2).ToLocalChecked()->IsTrue();
  auto resolver =
      data->Get(context, 0).ToLocalChecked().As<v8::Promise::Resolver>();
  if (!manager.host || !manager.host->ContextIsValid() ||
      !manager.worlds.Contains(id)) {
    return;
  }
  std::string message = "Isolated module evaluation failed";
  if (!success && info.Length()) {
    v8::String::Utf8Value detail(isolate, info[0]);
    if (*detail) {
      message.assign(*detail, detail.length());
    }
  }
  ScriptState::Scope scope(manager.host);
  if (success) {
    std::ignore = resolver->Resolve(manager.host->GetContext(),
                                    v8::Integer::New(isolate, id));
  } else {
    auto* world = manager.worlds.find(id)->value.Get();
    manager.worlds.erase(id);
    window->GetFrame()->WindowProxy(world->state->World())->ClearForClose();
    SetIsolatedWorldInfo(id, WebIsolatedWorldInfo());
    std::ignore =
        resolver->Reject(manager.host->GetContext(),
                         v8::Exception::Error(Text(isolate, message.c_str())));
  }
}

// No callback carries a raw C++ owner pointer or a caller-supplied principal.
// Every entry resolves its document and world from the executing context.
void Operate(const v8::FunctionCallbackInfo<v8::Value>& info) {
  auto* isolate = info.GetIsolate();
  auto* caller = ScriptState::ForCurrentRealm(isolate);
  auto* window = LocalDOMWindow::From(caller);
  if (!caller->ContextIsValid() || !window || !window->GetFrame()) {
    isolate->ThrowException(
        v8::Exception::Error(Text(isolate, "Document is gone")));
    return;
  }
  auto& manager = Worlds::From(*window);
  const int caller_id = caller->World().GetWorldId();
  if (info.Length() < 1 || !info[0]->IsString()) {
    return;
  }
  v8::String::Utf8Value action_text(isolate, info[0]);
  std::string action(*action_text, action_text.length());
  std::string failure = "Isolated operation failed or world is disposed";
  auto fail = [&] {
    isolate->ThrowException(
        v8::Exception::Error(Text(isolate, failure.c_str())));
  };
  if (caller_id != 0) {
    if (!manager.worlds.Contains(caller_id) || manager.sink.IsEmpty() ||
        !manager.host || !manager.host->ContextIsValid() ||
        (action != "request" && action != "cancel" && action != "error")) {
      fail();
      return;
    }
    // Only primitive protocol fields cross this boundary. DOM and JS objects
    // are never forwarded to the main world.
    std::array<v8::Local<v8::Value>, 7> args;
    if (info.Length() > 6) {
      fail();
      return;
    }
    args[0] = v8::Integer::New(isolate, caller_id);
    for (int i = 0; i < info.Length(); ++i) {
      if (!info[i]->IsString() && !info[i]->IsInt32()) {
        fail();
        return;
      }
      args[i + 1] = info[i];
    }
    ScriptState::Scope scope(manager.host);
    v8::TryCatch caught(isolate);
    v8::Local<v8::Value> ignored;
    bool called = manager.sink.Get(isolate)
                      .As<v8::Function>()
                      ->Call(manager.host->GetContext(), v8::Undefined(isolate),
                             info.Length() + 1, args.data())
                      .ToLocal(&ignored);
    if (!called) {
      caught.Reset();
    }
    return;
  }
  if (action == "sink") {
    if (info.Length() == 2 && info[1]->IsFunction()) {
      manager.host = caller;
      manager.sink.Reset(isolate, info[1]);
    }
    return;
  }
  if (action == "create") {
    if (info.Length() != 3 || !info[1]->IsString() || !info[2]->IsString() ||
        manager.worlds.size() >= 32) {
      fail();
      return;
    }
    static std::atomic<int> next_id{1 << 20};
    const int id = next_id.fetch_add(1);
    if (id >= kEmbedderWorldIdLimit) {
      fail();
      return;
    }
    WebIsolatedWorldInfo policy;
    policy.security_origin = WebSecurityOrigin(window->GetSecurityOrigin());
    v8::String::Utf8Value entry_text(isolate, info[2]);
    std::string entry(*entry_text, entry_text.length());
    constexpr std::string_view marker = "/.tuffite-isolated/";
    const auto start = entry.find(marker);
    if (start == std::string::npos) {
      fail();
      return;
    }
    const auto end = entry.find('/', start + marker.size());
    if (end == std::string::npos) {
      fail();
      return;
    }
    const auto root = entry.substr(0, end + 1);
    policy.content_security_policy = WebString::FromUtf8(
        "default-src 'none'; script-src " + root +
        "; style-src 'unsafe-inline' " + root +
        "; img-src data: blob: " + root + "; font-src data: " + root +
        "; connect-src " + root + "; base-uri 'none'; form-action 'none'");
    v8::String::Utf8Value name(isolate, info[1]);
    policy.human_readable_name =
        WebString::FromUtf8(std::string(*name, name.length()));
    SetIsolatedWorldInfo(id, policy);
    auto* wrapper_world = DOMWrapperWorld::EnsureIsolatedWorld(isolate, id);
    window->GetFrame()->WindowProxy(*wrapper_world)->InitializeIfNeeded();
    auto* state = ToScriptState(window->GetFrame(), *wrapper_world);
    if (!state || !state->ContextIsValid()) {
      fail();
      return;
    }
    auto* world = MakeGarbageCollected<World>();
    world->state = state;
    manager.worlds.Set(id, world);
    auto ready =
        v8::Promise::Resolver::New(caller->GetContext()).ToLocalChecked();
    bool success = false;
    {
      ScriptState::Scope scope(state);
      v8::TryCatch caught(isolate);
      v8::Local<v8::Value> factory;
      v8::Local<v8::Value> operations;
      v8::Local<v8::Value> ignored;
      auto native =
          v8::Function::New(state->GetContext(), Operate).ToLocalChecked();
      auto* registry = CustomElementRegistry::Create(state);
      v8::Local<v8::Value> args[] = {
          info[1], ToV8Traits<CustomElementRegistry>::ToV8(state, registry),
          native};
      success = Evaluate(state, kIsolatedWorldScript).ToLocal(&factory) &&
                factory.As<v8::Function>()
                    ->Call(state->GetContext(), v8::Undefined(isolate), 3, args)
                    .ToLocal(&operations);
      if (success) {
        world->operations.Reset(isolate, operations);
        v8::Local<v8::Value> importer;
        success = Evaluate(state, "(url => import(url))").ToLocal(&importer);
        if (success) {
          v8::Local<v8::Value> url = info[2];
          success =
              importer.As<v8::Function>()
                  ->Call(state->GetContext(), v8::Undefined(isolate), 1, &url)
                  .ToLocal(&ignored) &&
              ignored->IsPromise();
        }
        if (success) {
          std::array<v8::Local<v8::Function>, 2> callbacks;
          for (int i = 0; i < 2; ++i) {
            auto data = v8::Array::New(isolate, 3);
            data->Set(state->GetContext(), 0, ready).Check();
            data->Set(state->GetContext(), 1, v8::Integer::New(isolate, id))
                .Check();
            data->Set(state->GetContext(), 2, v8::Boolean::New(isolate, i == 0))
                .Check();
            callbacks[i] =
                v8::Function::New(state->GetContext(), ModuleCompleted, data)
                    .ToLocalChecked();
          }
          success = !ignored.As<v8::Promise>()
                         ->Then(state->GetContext(), callbacks[0], callbacks[1])
                         .IsEmpty();
        }
      }
      if (caught.HasCaught()) {
        v8::String::Utf8Value message(isolate, caught.Exception());
        if (*message) {
          failure.assign(*message, message.length());
        }
        caught.Reset();
      }
    }
    if (!success) {
      manager.worlds.erase(id);
      window->GetFrame()->WindowProxy(*wrapper_world)->ClearForClose();
      SetIsolatedWorldInfo(id, WebIsolatedWorldInfo());
      fail();
      return;
    }
    info.GetReturnValue().Set(ready->GetPromise());
    return;
  }
  if (info.Length() < 2 || !info[1]->IsInt32()) {
    fail();
    return;
  }
  const int id = info[1].As<v8::Int32>()->Value();
  auto it = manager.worlds.find(id);
  if (it == manager.worlds.end()) {
    fail();
    return;
  }
  World* world = it->value;
  if (!world->state->ContextIsValid()) {
    fail();
    return;
  }
  HTMLElement* container = nullptr;
  if (action == "mount") {
    if (info.Length() != 6 || !info[2]->IsObject()) {
      fail();
      return;
    }
    container = V8HTMLElement::ToWrappable(isolate, info[2]);
    if (!container || container->GetDocument().domWindow() != window) {
      fail();
      return;
    }
  }
  bool success = false;
  {
    ScriptState::Scope scope(world->state);
    v8::TryCatch caught(isolate);
    v8::Local<v8::Value> operation;
    auto object = world->operations.Get(isolate).As<v8::Object>();
    if (object->Get(world->state->GetContext(), Text(isolate, action.c_str()))
            .ToLocal(&operation) &&
        operation->IsFunction()) {
      std::array<v8::Local<v8::Value>, 5> args;
      if (info.Length() > 7) {
        fail();
        return;
      }
      for (int i = 2; i < info.Length(); ++i) {
        if (i == 2 && container) {
          args[i - 2] = ToV8Traits<HTMLElement>::ToV8(world->state, container);
        } else {
          if (!info[i]->IsString() && !info[i]->IsInt32() &&
              !info[i]->IsBoolean()) {
            fail();
            return;
          }
          args[i - 2] = info[i];
        }
      }
      v8::Local<v8::Value> ignored;
      success = operation.As<v8::Function>()
                    ->Call(world->state->GetContext(), object,
                           info.Length() - 2, args.data())
                    .ToLocal(&ignored);
    }
    if (caught.HasCaught()) {
      v8::String::Utf8Value message(isolate, caught.Exception());
      if (*message) {
        failure.assign(*message, message.length());
      }
      caught.Reset();
    }
  }
  if (action == "disposeWorld") {
    manager.worlds.erase(id);
    window->GetFrame()->WindowProxy(world->state->World())->ClearForClose();
    SetIsolatedWorldInfo(id, WebIsolatedWorldInfo());
  }
  if (!success) {
    fail();
  }
}
}  // namespace

bool IsTuffiteIsolatedWorld(ScriptState* state) {
  if (!state || state->World().IsMainWorld()) {
    return false;
  }
  auto* window = LocalDOMWindow::From(state);
  auto* manager =
      window ? Supplement<LocalDOMWindow>::From<Worlds>(*window) : nullptr;
  return manager && manager->worlds.Contains(state->World().GetWorldId());
}

void InstallIsolatedHost(v8::Local<v8::Context> context,
                         v8::Local<v8::Function> load) {
  auto* isolate = v8::Isolate::GetCurrent();
  auto* state = ScriptState::From(isolate, context);
  if (!state || !state->World().IsMainWorld()) {
    return;
  }
  ScriptState::Scope scope(state);
  v8::TryCatch caught(isolate);
  v8::Local<v8::Value> factory;
  if (!Evaluate(state, kIsolatedHostScript).ToLocal(&factory)) {
    return;
  }
  v8::Local<v8::Value> args[] = {
      v8::Function::New(context, Operate).ToLocalChecked(), load};
  v8::Local<v8::Value> ignored;
  bool installed = factory.As<v8::Function>()
                       ->Call(context, v8::Undefined(isolate), 2, args)
                       .ToLocal(&ignored);
  if (!installed) {
    caught.Reset();
  }
}
}  // namespace blink
