// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#include "tuffite/components/explorer/renderer/blink/html_explorer_element.h"

#include <algorithm>
#include <string>
#include <utility>
#include <vector>

#include "base/command_line.h"
#include "base/functional/callback_helpers.h"
#include "base/strings/pattern.h"
#include "base/strings/string_split.h"
#include "third_party/blink/renderer/bindings/core/v8/script_promise_resolver.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_binding_for_core.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_dialog.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_execute_java_script_options.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_reload_options.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_transfer.h"
#include "third_party/blink/renderer/core/dom/document.h"
#include "third_party/blink/renderer/core/dom/dom_exception.h"
#include "third_party/blink/renderer/core/dom/events/custom_event.h"
#include "third_party/blink/renderer/core/frame/local_frame.h"
#include "third_party/blink/renderer/core/html_names.h"
#include "third_party/blink/renderer/core/layout/layout_iframe.h"
#include "third_party/blink/renderer/platform/bindings/script_forbidden_scope.h"
#include "third_party/blink/renderer/platform/weborigin/security_origin.h"
#include "third_party/blink/renderer/platform/wtf/functional.h"
#include "tuffite/components/explorer/renderer/blink/explorer_dialog.h"
#include "tuffite/components/explorer/renderer/blink/explorer_event_propagation_monitor.h"
#include "tuffite/components/explorer/renderer/blink/explorer_transfer.h"
#include "v8/include/v8-json.h"

namespace blink {
namespace {

constexpr char kExplorerOriginsSwitch[] = "tuffite-explorer-origins";
constexpr size_t kMaxMessageBytes = 1024 * 1024;

const QualifiedName& ExplorerTagName() {
  DEFINE_STATIC_LOCAL(
      QualifiedName, name,
      (g_null_atom, AtomicString("explorer"), html_names::xhtmlNamespaceURI));
  return name;
}

void DispatchExplorerEvent(HTMLExplorerElement& element,
                           const AtomicString& type) {
  if (!element.isConnected() || !element.GetDocument().GetFrame() ||
      ScriptForbiddenScope::IsScriptForbidden()) {
    return;
  }
  element.DispatchEvent(*Event::Create(type));
}

template <typename Callback>
ScriptPromise<IDLBoolean> BooleanOperation(
    ScriptState* script_state,
    HeapMojoRemote<tuffite::mojom::blink::ExplorerHost>& host,
    Callback invoke) {
  auto* resolver =
      MakeGarbageCollected<ScriptPromiseResolver<IDLBoolean>>(script_state);
  auto promise = resolver->Promise();
  if (!host.is_bound()) {
    resolver->Resolve(false);
    return promise;
  }
  invoke(BindOnce([](ScriptPromiseResolver<IDLBoolean>* resolver,
                     bool accepted) { resolver->Resolve(accepted); },
                  WrapPersistent(resolver)));
  return promise;
}

}  // namespace

HTMLExplorerElement::HTMLExplorerElement(Document& document)
    : HTMLFrameOwnerElement(ExplorerTagName(), document),
      host_(document.GetExecutionContext()),
      client_receiver_(this, document.GetExecutionContext()) {}

HTMLExplorerElement::~HTMLExplorerElement() = default;

// static
bool HTMLExplorerElement::IsEnabled(const Document& document) {
  const ExecutionContext* execution_context = document.GetExecutionContext();
  const SecurityOrigin* security_origin =
      execution_context ? execution_context->GetSecurityOrigin() : nullptr;
  if (!security_origin || security_origin->IsOpaque()) {
    return false;
  }
  const std::string origin = security_origin->ToUrlOrigin().Serialize();
  const std::string allowlist =
      base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
          kExplorerOriginsSwitch);
  const std::vector<std::string> allowed = base::SplitString(
      allowlist, ",", base::TRIM_WHITESPACE, base::SPLIT_WANT_NONEMPTY);
  return std::ranges::any_of(allowed, [&](const std::string& pattern) {
    return base::MatchPattern(origin, pattern);
  });
}

void HTMLExplorerElement::Trace(Visitor* visitor) const {
  visitor->Trace(detach_resolver_);
  visitor->Trace(attach_resolver_);
  visitor->Trace(host_);
  visitor->Trace(client_receiver_);
  HTMLFrameOwnerElement::Trace(visitor);
}

String HTMLExplorerElement::src() const {
  return FastGetAttribute(html_names::kSrcAttr);
}

void HTMLExplorerElement::setSrc(const String& value) {
  setAttribute(html_names::kSrcAttr, AtomicString(value));
}

String HTMLExplorerElement::partition() const {
  return FrozenAttribute("partition", "");
}
void HTMLExplorerElement::setPartition(const String& value) {
  setAttribute(AtomicString("partition"), AtomicString(value));
}
String HTMLExplorerElement::navigation() const {
  return FrozenAttribute("navigation", "emit");
}
void HTMLExplorerElement::setNavigation(const String& value) {
  setAttribute(AtomicString("navigation"), AtomicString(value));
}
String HTMLExplorerElement::dialogs() const {
  return FrozenAttribute("dialogs", "suppress");
}
void HTMLExplorerElement::setDialogs(const String& value) {
  setAttribute(AtomicString("dialogs"), AtomicString(value));
}

String HTMLExplorerElement::FrozenAttribute(const char* name,
                                            const char* default_value) const {
  const AtomicString value = getAttribute(AtomicString(name));
  return value.empty() ? String(default_value) : String(value);
}

bool HTMLExplorerElement::contextMenuEnabled() const {
  return context_menu_enabled_;
}

void HTMLExplorerElement::setContextMenuEnabled(bool enabled) {
  context_menu_enabled_ = enabled;
  if (host_.is_bound()) {
    host_->SetContextMenuEnabled(enabled);
  }
}

void HTMLExplorerElement::load(const String& value) {
  if (detach_resolver_ || attach_resolver_) {
    return;
  }
  detached_empty_ = false;
  setSrc(value);
  setDeferred(false);
}

bool HTMLExplorerElement::deferred() const {
  return FastHasAttribute(QualifiedName(AtomicString("deferred")));
}
void HTMLExplorerElement::setDeferred(bool value) {
  SetBooleanAttribute(QualifiedName(AtomicString("deferred")), value);
}

ScriptPromise<ExplorerTransfer> HTMLExplorerElement::detach(
    ScriptState* state) {
  if (!attached_ || !host_.is_bound() || create_pending_ || detach_resolver_ ||
      attach_resolver_) {
    return ScriptPromise<ExplorerTransfer>::RejectWithDOMException(
        state, MakeGarbageCollected<DOMException>(
                   DOMExceptionCode::kInvalidStateError,
                   "Explorer is not available to detach"));
  }
  auto* transfer =
      MakeGarbageCollected<ExplorerTransfer>(GetExecutionContext());
  detach_resolver_ =
      MakeGarbageCollected<ScriptPromiseResolver<ExplorerTransfer>>(state);
  auto promise = detach_resolver_->Promise();
  host_->Detach(
      transfer->BindClient(),
      BindOnce(
          [](HTMLExplorerElement* element, ExplorerTransfer* transfer,
             const String& token,
             mojo::PendingRemote<tuffite::mojom::blink::ExplorerTransferControl>
                 control,
             const String& error) {
            if (!element->detach_resolver_) {
              return;
            }
            auto* resolver = element->detach_resolver_.Get();
            element->detach_resolver_.Clear();
            if (!error.empty() || !control.is_valid()) {
              resolver->RejectWithDOMException(
                  DOMExceptionCode::kInvalidStateError,
                  "Explorer detach failed");
              return;
            }
            transfer->Initialize(token, std::move(control));
            element->attached_ = false;
            element->detached_empty_ = true;
            element->host_.reset();
            element->client_receiver_.reset();
            element->DisconnectContentFrame();
            DispatchExplorerEvent(*element, AtomicString("detached"));
            resolver->Resolve(transfer);
          },
          WrapPersistent(this), WrapPersistent(transfer)));
  return promise;
}

ScriptPromise<IDLUndefined> HTMLExplorerElement::attach(ScriptState* state,
                                                        const String& token) {
  if (attached_ || create_pending_ || detach_resolver_ || attach_resolver_ ||
      (!detached_empty_ && FastHasAttribute(html_names::kSrcAttr)) ||
      !PrepareHost()) {
    return ScriptPromise<IDLUndefined>::RejectWithDOMException(
        state, MakeGarbageCollected<DOMException>(
                   DOMExceptionCode::kInvalidStateError,
                   "Explorer target must be connected and empty"));
  }
  attach_resolver_ =
      MakeGarbageCollected<ScriptPromiseResolver<IDLUndefined>>(state);
  auto promise = attach_resolver_->Promise();
  client_receiver_.reset();
  auto client = client_receiver_.BindNewPipeAndPassRemote(
      GetExecutionContext()->GetTaskRunner(TaskType::kInternalLoading));
  host_->Attach(token, CreateParams(), std::move(client),
                BindOnce(
                    [](HTMLExplorerElement* element, const String& error) {
                      if (!element->attach_resolver_) {
                        return;
                      }
                      auto* resolver = element->attach_resolver_.Get();
                      element->attach_resolver_.Clear();
                      if (!error.empty()) {
                        element->client_receiver_.reset();
                        element->host_.reset();
                        // Preparation reserves the placeholder frame for inner
                        // contents. Remove it so a retry uses a fresh frame.
                        element->DisconnectContentFrame();
                        auto code = error == "SecurityError"
                                        ? DOMExceptionCode::kSecurityError
                                    : error == "AbortError"
                                        ? DOMExceptionCode::kAbortError
                                        : DOMExceptionCode::kInvalidStateError;
                        resolver->RejectWithDOMException(
                            code, "Explorer attach failed: " + error);
                        return;
                      }
                      element->attached_ = true;
                      element->detached_empty_ = false;
                      element->setDeferred(false);
                      element->host_->SetContextMenuEnabled(
                          element->context_menu_enabled_);
                      DispatchExplorerEvent(*element, AtomicString("attached"));
                      resolver->Resolve();
                    },
                    WrapPersistent(this)));
  return promise;
}

ScriptPromise<IDLBoolean> HTMLExplorerElement::back(ScriptState* state) {
  return go(state, -1);
}
ScriptPromise<IDLBoolean> HTMLExplorerElement::forward(ScriptState* state) {
  return go(state, 1);
}
ScriptPromise<IDLBoolean> HTMLExplorerElement::go(ScriptState* state,
                                                  int32_t offset) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->Go(offset, std::move(callback));
  });
}
ScriptPromise<IDLBoolean> HTMLExplorerElement::reload(
    ScriptState* state,
    const ExplorerReloadOptions* options) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->Reload(options && options->bypassCache(), std::move(callback));
  });
}
ScriptPromise<IDLBoolean> HTMLExplorerElement::stop(ScriptState* state) {
  return BooleanOperation(
      state, host_, [&](auto callback) { host_->Stop(std::move(callback)); });
}

ScriptPromise<IDLBoolean> HTMLExplorerElement::download(ScriptState* state,
                                                        const String& url) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->Download(url, std::move(callback));
  });
}
ScriptPromise<IDLBoolean> HTMLExplorerElement::cancelDownload(
    ScriptState* state,
    uint32_t id) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->CancelDownload(id, std::move(callback));
  });
}
ScriptPromise<IDLString> HTMLExplorerElement::readDownload(ScriptState* state,
                                                           uint32_t id) {
  auto* resolver =
      MakeGarbageCollected<ScriptPromiseResolver<IDLString>>(state);
  auto promise = resolver->Promise();
  if (!host_.is_bound()) {
    resolver->RejectWithTypeError("Explorer is not attached");
    return promise;
  }
  host_->ReadDownload(id, BindOnce(
                              [](ScriptPromiseResolver<IDLString>* resolver,
                                 const String& data, const String& error) {
                                if (!error.empty()) {
                                  resolver->RejectWithTypeError(error);
                                } else {
                                  resolver->Resolve(data);
                                }
                              },
                              WrapPersistent(resolver)));
  return promise;
}
ScriptPromise<IDLString> HTMLExplorerElement::capturePreview(
    ScriptState* state) {
  auto* resolver =
      MakeGarbageCollected<ScriptPromiseResolver<IDLString>>(state);
  auto promise = resolver->Promise();
  if (!host_.is_bound()) {
    resolver->Resolve(g_empty_string);
    return promise;
  }
  host_->CapturePreview(
      BindOnce([](ScriptPromiseResolver<IDLString>* resolver,
                  const String& data) { resolver->Resolve(data); },
               WrapPersistent(resolver)));
  return promise;
}
void HTMLExplorerElement::DidDownload(const String& json) {
  DispatchJsonEvent(AtomicString("download"), json);
}

ScriptPromise<IDLAny> HTMLExplorerElement::getDownloads(ScriptState* state) {
  auto* resolver = MakeGarbageCollected<ScriptPromiseResolver<IDLAny>>(state);
  auto promise = resolver->Promise();
  if (!host_.is_bound()) {
    resolver->RejectWithTypeError("Explorer is not attached");
    return promise;
  }
  host_->GetDownloads(BindOnce(
      [](ScriptPromiseResolver<IDLAny>* resolver, const String& json) {
        ScriptState* state = resolver->GetScriptState();
        ScriptState::Scope scope(state);
        resolver->Resolve(
            ScriptValue(state->GetIsolate(), FromJSONString(state, json)));
      },
      WrapPersistent(resolver)));
  return promise;
}

ScriptPromise<IDLAny> HTMLExplorerElement::getNavigationState(
    ScriptState* state) {
  auto* resolver = MakeGarbageCollected<ScriptPromiseResolver<IDLAny>>(state);
  auto promise = resolver->Promise();
  if (!host_.is_bound()) {
    resolver->RejectWithTypeError("Explorer is not attached");
    return promise;
  }
  host_->GetNavigationState(BindOnce(
      [](ScriptPromiseResolver<IDLAny>* resolver, const String& json) {
        ScriptState* state = resolver->GetScriptState();
        ScriptState::Scope scope(state);
        resolver->Resolve(
            ScriptValue(state->GetIsolate(), FromJSONString(state, json)));
      },
      WrapPersistent(resolver)));
  return promise;
}

ScriptPromise<IDLAny> HTMLExplorerElement::executeJavaScript(
    ScriptState* state,
    const String& source,
    const ExplorerExecuteJavaScriptOptions* options) {
  auto* resolver = MakeGarbageCollected<ScriptPromiseResolver<IDLAny>>(state);
  auto promise = resolver->Promise();
  if (!host_.is_bound()) {
    resolver->RejectWithTypeError("Explorer is not attached");
    return promise;
  }
  const bool main_world = options && options->world() == "main";
  if (options && options->world() != "isolated" && !main_world) {
    resolver->RejectWithTypeError("world must be isolated or main");
    return promise;
  }
  const String expected_document_id =
      options ? options->expectedDocumentId() : g_empty_string;
  host_->ExecuteJavaScript(
      source, main_world,
      expected_document_id.IsNull() ? g_empty_string : expected_document_id,
      BindOnce(
          [](ScriptPromiseResolver<IDLAny>* resolver, bool success,
             const String& json, const String& error) {
            if (!success) {
              resolver->RejectWithTypeError(error);
              return;
            }
            ScriptState* state = resolver->GetScriptState();
            ScriptState::Scope scope(state);
            resolver->Resolve(
                ScriptValue(state->GetIsolate(), FromJSONString(state, json)));
          },
          WrapPersistent(resolver)));
  return promise;
}

ScriptPromise<IDLBoolean> HTMLExplorerElement::RespondToDialog(
    ScriptState* state,
    uint64_t request_id,
    bool accept,
    const String& prompt_text) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->RespondToDialog(request_id, accept,
                           prompt_text.IsNull() ? g_empty_string : prompt_text,
                           std::move(callback));
  });
}

ScriptPromise<IDLBoolean> HTMLExplorerElement::postMessage(ScriptState* state,
                                                           ScriptValue data) {
  v8::Local<v8::String> json;
  if (!v8::JSON::Stringify(state->GetContext(), data.V8Value())
           .ToLocal(&json)) {
    return ToResolvedPromise<IDLBoolean>(state, false);
  }
  const String serialized =
      ToBlinkString<String>(state->GetIsolate(), json, kDoNotExternalize);
  if (serialized == "undefined" ||
      serialized.Utf8().size() > kMaxMessageBytes) {
    return ToResolvedPromise<IDLBoolean>(state, false);
  }
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->PostMessage(serialized, std::move(callback));
  });
}

FrameOwnerElementType HTMLExplorerElement::OwnerType() const {
  return FrameOwnerElementType::kIframe;
}

Node::InsertionNotificationRequest HTMLExplorerElement::InsertedInto(
    ContainerNode& insertion_point) {
  HTMLFrameOwnerElement::InsertedInto(insertion_point);
  return kInsertionShouldCallDidNotifySubtreeInsertions;
}
void HTMLExplorerElement::DidNotifySubtreeInsertionsToDocument() {
  EnsureAttached();
}
void HTMLExplorerElement::RemovedFrom(ContainerNode& insertion_point) {
  if (GetDocument().StatePreservingAtomicMoveInProgress()) {
    HTMLFrameOwnerElement::RemovedFrom(insertion_point);
    return;
  }
  OnConnectionError();
  client_receiver_.reset();
  host_.reset();
  create_pending_ = false;
  attached_ = false;
  create_initial_url_ = String();
  HTMLFrameOwnerElement::RemovedFrom(insertion_point);
}
void HTMLExplorerElement::ParseAttribute(
    const AttributeModificationParams& params) {
  if (params.name == QualifiedName(AtomicString("deferred"))) {
    if (params.new_value.IsNull()) {
      EnsureAttached();
    }
    return;
  }
  if (params.name == html_names::kSrcAttr) {
    if (attached_ && host_.is_bound()) {
      const String target =
          params.new_value.empty()
              ? BlankUrl().GetString()
              : GetDocument().CompleteURL(params.new_value).GetString();
      host_->Navigate(target, base::DoNothing());
    } else {
      EnsureAttached();
    }
    return;
  }
  HTMLFrameOwnerElement::ParseAttribute(params);
}
bool HTMLExplorerElement::LayoutObjectIsNeeded(
    const DisplayStyle& style) const {
  return ContentFrame() && HTMLElement::LayoutObjectIsNeeded(style);
}
LayoutObject* HTMLExplorerElement::CreateLayoutObject(const ComputedStyle&) {
  return MakeGarbageCollected<LayoutIFrame>(this);
}
bool HTMLExplorerElement::IsInteractiveContent() const {
  return true;
}
network::ParsedPermissionsPolicy HTMLExplorerElement::ConstructContainerPolicy()
    const {
  return GetLegacyFramePolicies();
}

bool HTMLExplorerElement::PrepareHost() {
  if (!isConnected() || !GetDocument().GetFrame()) {
    return false;
  }
  if (!ContentFrame()) {
    LoadOrRedirectSubframe(BlankUrl(), g_null_atom, true);
  }
  if (!ContentFrame() ||
      !ContentFrame()->GetFrameToken().Is<LocalFrameToken>()) {
    return false;
  }
  if (!host_.is_bound()) {
    GetDocument().GetFrame()->GetBrowserInterfaceBroker().GetInterface(
        host_.BindNewPipeAndPassReceiver(
            GetExecutionContext()->GetTaskRunner(TaskType::kInternalLoading)));
    host_.set_disconnect_handler(BindOnce(
        &HTMLExplorerElement::OnConnectionError, WrapWeakPersistent(this)));
  }
  return true;
}

tuffite::mojom::blink::ExplorerCreateParamsPtr
HTMLExplorerElement::CreateParams() const {
  auto params = tuffite::mojom::blink::ExplorerCreateParams::New();
  params->frame_to_replace =
      ContentFrame()->GetFrameToken().GetAs<LocalFrameToken>();
  params->initial_url = src().empty()
                            ? BlankUrl().GetString()
                            : GetDocument().CompleteURL(src()).GetString();
  params->partition = partition();
  params->navigation = navigation();
  params->dialogs = dialogs();
  params->context_menu_enabled = context_menu_enabled_;
  return params;
}

void HTMLExplorerElement::EnsureAttached() {
  if (!isConnected() || create_pending_ || attached_ || deferred() ||
      detached_empty_ || attach_resolver_ || !GetDocument().GetFrame()) {
    return;
  }
  if (!PrepareHost()) {
    return;
  }
  client_receiver_.reset();
  auto client = client_receiver_.BindNewPipeAndPassRemote(
      GetExecutionContext()->GetTaskRunner(TaskType::kInternalLoading));
  create_pending_ = true;
  const String initial_url = src().empty()
                                 ? BlankUrl().GetString()
                                 : GetDocument().CompleteURL(src()).GetString();
  create_initial_url_ = initial_url;
  auto params = CreateParams();
  host_->Create(std::move(params), std::move(client),
                BindOnce(&HTMLExplorerElement::OnCreateComplete,
                         WrapWeakPersistent(this)));
}

void HTMLExplorerElement::OnCreateComplete(bool accepted) {
  create_pending_ = false;
  attached_ = accepted;
  if (!accepted) {
    create_initial_url_ = String();
    client_receiver_.reset();
    DispatchExplorerEvent(*this, AtomicString("error"));
    return;
  }
  const String current_url = src().empty()
                                 ? BlankUrl().GetString()
                                 : GetDocument().CompleteURL(src()).GetString();
  if (current_url != create_initial_url_ && host_.is_bound()) {
    host_->Navigate(current_url, base::DoNothing());
  }
  create_initial_url_ = String();
}

void HTMLExplorerElement::OnConnectionError() {
  if (detach_resolver_) {
    detach_resolver_->RejectWithDOMException(
        DOMExceptionCode::kAbortError, "Explorer disconnected during detach");
    detach_resolver_.Clear();
  }
  if (attach_resolver_) {
    attach_resolver_->RejectWithDOMException(
        DOMExceptionCode::kAbortError, "Explorer disconnected during attach");
    attach_resolver_.Clear();
  }
  host_.reset();
  client_receiver_.reset();
  create_pending_ = false;
  attached_ = false;
  create_initial_url_ = String();
}

void HTMLExplorerElement::DispatchJsonEvent(const AtomicString& type,
                                            const String& json) {
  if (!isConnected() || !GetDocument().GetFrame() ||
      ScriptForbiddenScope::IsScriptForbidden()) {
    return;
  }
  ScriptState* state = ToScriptStateForMainWorld(GetDocument().GetFrame());
  if (!state || !state->ContextIsValid()) {
    return;
  }
  ScriptState::Scope scope(state);
  CustomEvent* event = CustomEvent::Create();
  event->initCustomEvent(
      state, type, false, false,
      ScriptValue(state->GetIsolate(), FromJSONString(state, json)));
  DispatchEvent(*event);
}

void HTMLExplorerElement::DidStartLoading(const String& state_json) {
  DispatchJsonEvent(AtomicString("loadstart"), state_json);
}
void HTMLExplorerElement::DidStopLoading(const String& state_json) {
  DispatchJsonEvent(AtomicString("loadstop"), state_json);
}
void HTMLExplorerElement::DidFailLoad(int32_t error_code,
                                      const String& error_description,
                                      const String& state_json) {
  if (!isConnected() || !GetDocument().GetFrame() ||
      ScriptForbiddenScope::IsScriptForbidden()) {
    return;
  }
  ScriptState* state = ToScriptStateForMainWorld(GetDocument().GetFrame());
  if (!state) {
    return;
  }
  ScriptState::Scope scope(state);
  v8::Local<v8::Value> parsed = FromJSONString(state, state_json);
  if (!parsed->IsObject()) {
    parsed = v8::Object::New(state->GetIsolate());
  }
  v8::Local<v8::Context> context = state->GetContext();
  v8::Local<v8::Object> detail = parsed.As<v8::Object>();
  detail
      ->Set(context, V8String(state->GetIsolate(), "errorCode"),
            v8::Integer::New(state->GetIsolate(), error_code))
      .Check();
  detail
      ->Set(context, V8String(state->GetIsolate(), "errorDescription"),
            V8String(state->GetIsolate(), error_description))
      .Check();
  CustomEvent* event = CustomEvent::Create();
  event->initCustomEvent(state, AtomicString("loaderror"), false, false,
                         ScriptValue(state->GetIsolate(), detail));
  DispatchEvent(*event);
}
void HTMLExplorerElement::DidCommitNavigation(const String& state_json) {
  DispatchJsonEvent(AtomicString("loadcommit"), state_json);
}
void HTMLExplorerElement::DidRequestOpen(uint64_t request_id,
                                         const String& event_type,
                                         const String& request_json) {
  if (!isConnected() || !GetDocument().GetFrame() ||
      ScriptForbiddenScope::IsScriptForbidden()) {
    if (host_.is_bound()) {
      host_->CompleteOpen(request_id, true);
    }
    return;
  }
  ScriptState* state = ToScriptStateForMainWorld(GetDocument().GetFrame());
  if (!state || !state->ContextIsValid()) {
    if (host_.is_bound()) {
      host_->CompleteOpen(request_id, true);
    }
    return;
  }
  ScriptState::Scope scope(state);
  CustomEvent* event = CustomEvent::Create();
  event->initCustomEvent(
      state, AtomicString(event_type), true, false,
      ScriptValue(state->GetIsolate(), FromJSONString(state, request_json)));
  event->SetComposed(true);
  TuffiteEventPropagationMonitor propagation_monitor(*event);
  DispatchEvent(*event);
  if (host_.is_bound()) {
    host_->CompleteOpen(request_id, !propagation_monitor.WasStopped());
  }
}
void HTMLExplorerElement::DidRequestDialog(uint64_t request_id,
                                           const String& type,
                                           const String& message,
                                           const String& default_value,
                                           bool is_reload,
                                           const String& from) {
  if (!isConnected() || !GetDocument().GetFrame() ||
      ScriptForbiddenScope::IsScriptForbidden()) {
    if (host_.is_bound()) {
      host_->CompleteDialog(request_id, true);
    }
    return;
  }
  ScriptState* state = ToScriptStateForMainWorld(GetDocument().GetFrame());
  if (!state || !state->ContextIsValid()) {
    if (host_.is_bound()) {
      host_->CompleteDialog(request_id, true);
    }
    return;
  }
  ScriptState::Scope scope(state);
  auto* detail = MakeGarbageCollected<ExplorerDialog>(
      *this, request_id, type, message, default_value, is_reload, from);
  CustomEvent* event = CustomEvent::Create();
  event->initCustomEvent(
      state, AtomicString("dialog"), true, false,
      ScriptValue(state->GetIsolate(),
                  ToV8Traits<ExplorerDialog>::ToV8(state, detail)));
  event->SetComposed(true);
  TuffiteEventPropagationMonitor propagation_monitor(*event);
  DispatchEvent(*event);
  if (host_.is_bound()) {
    host_->CompleteDialog(request_id, !propagation_monitor.WasStopped());
  }
}
void HTMLExplorerElement::DidReceiveMessage(const String& json,
                                            const String& from) {
  (void)from;
  DispatchJsonEvent(AtomicString("message"), json);
}

}  // namespace blink
