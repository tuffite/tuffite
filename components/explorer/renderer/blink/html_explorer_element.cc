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
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_find_options.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_handle.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_new_window.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_explorer_reload_options.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_window_post_message_options.h"
#include "third_party/blink/renderer/core/dom/abort_controller.h"
#include "third_party/blink/renderer/core/dom/document.h"
#include "third_party/blink/renderer/core/dom/dom_exception.h"
#include "third_party/blink/renderer/core/dom/events/custom_event.h"
#include "third_party/blink/renderer/core/events/message_event.h"
#include "third_party/blink/renderer/core/frame/local_frame.h"
#include "third_party/blink/renderer/core/html_names.h"
#include "third_party/blink/renderer/core/inspector/console_message.h"
#include "third_party/blink/renderer/core/layout/layout_iframe.h"
#include "third_party/blink/renderer/core/messaging/message_port.h"
#include "third_party/blink/renderer/core/permissions_policy/permissions_policy_parser.h"
#include "third_party/blink/renderer/platform/bindings/script_forbidden_scope.h"
#include "third_party/blink/renderer/platform/weborigin/security_origin.h"
#include "third_party/blink/renderer/platform/wtf/functional.h"
#include "tuffite/components/explorer/renderer/blink/explorer_dialog.h"
#include "tuffite/components/explorer/renderer/blink/explorer_handle.h"
#include "tuffite/components/explorer/renderer/blink/explorer_message_serialization.h"
#include "tuffite/components/explorer/renderer/blink/explorer_new_window.h"
#include "v8/include/v8-json.h"

namespace blink {
namespace {

constexpr char kExplorerOriginsSwitch[] = "tuffite-explorer-origins";

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
  auto* event = Event::CreateBubble(type);
  event->SetComposed(true);
  element.DispatchEvent(*event);
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
  visitor->Trace(dialogs_pending_);
  visitor->Trace(windows_pending_);
  visitor->Trace(detach_resolver_);
  visitor->Trace(attach_resolver_);
  visitor->Trace(attaching_handle_);
  visitor->Trace(replaced_handle_);
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
bool HTMLExplorerElement::isolated() const {
  return hasAttribute(AtomicString("isolated"));
}
void HTMLExplorerElement::setIsolated(bool value) {
  if (value) {
    setAttribute(AtomicString("isolated"), AtomicString(""));
  } else {
    removeAttribute(AtomicString("isolated"));
  }
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
}

ScriptPromise<ExplorerHandle> HTMLExplorerElement::detach(ScriptState* state) {
  if (!attached_ || !host_.is_bound() || create_pending_ || detach_resolver_ ||
      attach_resolver_) {
    return ScriptPromise<ExplorerHandle>::RejectWithDOMException(
        state, MakeGarbageCollected<DOMException>(
                   DOMExceptionCode::kInvalidStateError,
                   "Explorer is not available to detach"));
  }
  auto* transfer = MakeGarbageCollected<ExplorerHandle>(GetExecutionContext());
  detach_resolver_ =
      MakeGarbageCollected<ScriptPromiseResolver<ExplorerHandle>>(state);
  auto promise = detach_resolver_->Promise();
  host_->Detach(
      transfer->BindClient(),
      BindOnce(
          [](HTMLExplorerElement* element, ExplorerHandle* transfer,
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

ScriptPromise<IDLUndefined> HTMLExplorerElement::attach(
    ScriptState* state,
    ExplorerHandle* handle) {
  if (!isConnected() || create_pending_ || detach_resolver_ ||
      attach_resolver_ || !handle->BeginAttach()) {
    return ScriptPromise<IDLUndefined>::RejectWithDOMException(
        state, MakeGarbageCollected<DOMException>(
                   DOMExceptionCode::kInvalidStateError,
                   "Explorer or handle is unavailable"));
  }
  attaching_handle_ = handle;
  auto promise = AttachToken(state, handle->token());
  return promise;
}
ScriptPromise<IDLUndefined> HTMLExplorerElement::AttachToken(
    ScriptState* state,
    const String& token) {
  if (!isConnected() || create_pending_ || detach_resolver_ ||
      attach_resolver_) {
    if (attaching_handle_) {
      attaching_handle_->RestorePending();
      attaching_handle_.Clear();
    }
    return ScriptPromise<IDLUndefined>::RejectWithDOMException(
        state,
        MakeGarbageCollected<DOMException>(DOMExceptionCode::kInvalidStateError,
                                           "Explorer target is unavailable"));
  }
  attach_resolver_ =
      MakeGarbageCollected<ScriptPromiseResolver<IDLUndefined>>(state);
  auto promise = attach_resolver_->Promise();
  if (attached_ && host_.is_bound()) {
    auto* previous =
        MakeGarbageCollected<ExplorerHandle>(GetExecutionContext());
    host_->Detach(
        previous->BindClient(),
        BindOnce(
            [](HTMLExplorerElement* element, ExplorerHandle* previous,
               String token, const String& old_token,
               mojo::PendingRemote<
                   tuffite::mojom::blink::ExplorerTransferControl> control,
               const String& error) {
              if (!element->attach_resolver_) {
                return;
              }
              if (!error.empty() || !control.is_valid()) {
                element->FinishAttach("InvalidStateError");
                return;
              }
              previous->Initialize(old_token, std::move(control));
              element->replaced_handle_ = previous;
              element->attached_ = false;
              element->host_.reset();
              element->client_receiver_.reset();
              element->DisconnectContentFrame();
              element->BeginAttach(token);
            },
            WrapPersistent(this), WrapPersistent(previous), token));
  } else {
    BeginAttach(token);
  }
  return promise;
}
void HTMLExplorerElement::BeginAttach(const String& token) {
  if (!PrepareHost()) {
    FinishAttach("AbortError");
    return;
  }
  client_receiver_.reset();
  auto client = client_receiver_.BindNewPipeAndPassRemote(
      GetExecutionContext()->GetTaskRunner(TaskType::kInternalLoading));
  host_->Attach(
      token, CreateParams(), std::move(client),
      BindOnce(&HTMLExplorerElement::FinishAttach, WrapPersistent(this)));
}
void HTMLExplorerElement::FinishAttach(const String& error) {
  if (!attach_resolver_) {
    return;
  }
  auto* script_state = attach_resolver_->GetScriptState();
  if (!script_state->ContextIsValid()) {
    return;
  }
  ScriptState::Scope scope(script_state);
  if (!error.empty()) {
    if (!attached_) {
      host_.reset();
      client_receiver_.reset();
      DisconnectContentFrame();
    }
    if (replaced_handle_ && !restoring_attachment_) {
      attach_error_ = error;
      restoring_attachment_ = true;
      BeginAttach(replaced_handle_->token());
      return;
    }
  }
  auto* resolver = attach_resolver_.Get();
  attach_resolver_.Clear();
  auto* requested = attaching_handle_.Get();
  auto* previous = replaced_handle_.Get();
  String failure = restoring_attachment_ ? attach_error_ : error;
  if (error.empty()) {
    attached_ = true;
    detached_empty_ = false;
    if (!restoring_attachment_) {
      clearing_src_ = true;
      removeAttribute(html_names::kSrcAttr);
      clearing_src_ = false;
      if (previous) {
        previous->close(resolver->GetScriptState());
      }
    }
    host_->SetContextMenuEnabled(context_menu_enabled_);
  }
  attaching_handle_.Clear();
  replaced_handle_.Clear();
  restoring_attachment_ = false;
  attach_error_ = String();
  // Only dispatch handle notifications after completing native/DOM updates.
  if (requested) {
    if (failure.empty()) {
      requested->Consume();
    } else {
      requested->RestorePending();
    }
  }
  if (previous && !failure.empty() && error.empty()) {
    previous->Consume();
  }
  if (previous && !error.empty()) {
    previous->close(resolver->GetScriptState());
  }
  if (failure.empty()) {
    resolver->Resolve();
    DispatchExplorerEvent(*this, AtomicString("attached"));
  } else {
    resolver->RejectWithDOMException(
        failure == "SecurityError" ? DOMExceptionCode::kSecurityError
        : failure == "AbortError"  ? DOMExceptionCode::kAbortError
                                   : DOMExceptionCode::kInvalidStateError,
        "Explorer attach failed: " + failure);
  }
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

ScriptPromise<IDLLong> HTMLExplorerElement::findInPage(
    ScriptState* state,
    const String& text,
    const ExplorerFindOptions* options) {
  auto* resolver = MakeGarbageCollected<ScriptPromiseResolver<IDLLong>>(state);
  auto promise = resolver->Promise();
  if (!host_.is_bound()) {
    resolver->RejectWithTypeError("Explorer is not attached");
    return promise;
  }
  host_->FindInPage(
      text, !options || options->forward(), options && options->matchCase(),
      options && options->findNext(),
      BindOnce(
          [](ScriptPromiseResolver<IDLLong>* resolver, int32_t id) {
            if (id < 0) {
              resolver->RejectWithTypeError("Invalid find request");
            } else {
              resolver->Resolve(id);
            }
          },
          WrapPersistent(resolver)));
  return promise;
}

ScriptPromise<IDLBoolean> HTMLExplorerElement::stopFindInPage(
    ScriptState* state,
    const String& action) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->StopFindInPage(action, std::move(callback));
  });
}

ScriptPromise<IDLBoolean> HTMLExplorerElement::setZoomFactor(ScriptState* state,
                                                             double factor) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->SetZoomFactor(factor, std::move(callback));
  });
}

ScriptPromise<IDLDouble> HTMLExplorerElement::getZoomFactor(
    ScriptState* state) {
  auto* resolver =
      MakeGarbageCollected<ScriptPromiseResolver<IDLDouble>>(state);
  auto promise = resolver->Promise();
  if (!host_.is_bound()) {
    resolver->RejectWithTypeError("Explorer is not attached");
    return promise;
  }
  host_->GetZoomFactor(
      BindOnce([](ScriptPromiseResolver<IDLDouble>* resolver,
                  double factor) { resolver->Resolve(factor); },
               WrapPersistent(resolver)));
  return promise;
}

ScriptPromise<IDLBoolean> HTMLExplorerElement::setAudioMuted(ScriptState* state,
                                                             bool muted) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->SetAudioMuted(muted, std::move(callback));
  });
}

void HTMLExplorerElement::DidChangePageState(const String& type,
                                             const String& json) {
  DispatchJsonEvent(AtomicString(type), json);
}

void HTMLExplorerElement::DidFindResult(const String& json) {
  DispatchJsonEvent(AtomicString("findresult"), json);
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

void HTMLExplorerElement::postMessage(ScriptState* state,
                                      ScriptValue value,
                                      const String& origin,
                                      HeapVector<ScriptObject> transfer,
                                      ExceptionState& exception) {
  auto* options = WindowPostMessageOptions::Create();
  options->setTargetOrigin(origin);
  options->setTransfer(std::move(transfer));
  postMessage(state, value, options, exception);
}
void HTMLExplorerElement::postMessage(ScriptState* state,
                                      ScriptValue value,
                                      const WindowPostMessageOptions* options,
                                      ExceptionState& exception) {
  if (!attached_ || !ContentFrame()) {
    return;
  }
  ContentFrame()->DomWindow()->postMessage(state->GetIsolate(), value, options,
                                           exception);
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
  if (params.name == html_names::kSrcAttr) {
    if (clearing_src_) {
      return;
    }
    UpdateContainerPolicy();
    if (host_.is_bound()) {
      host_->SetFramePolicy(GetFramePolicy());
    }
    detached_empty_ = false;
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
  if (params.name == html_names::kAllowAttr) {
    UpdateContainerPolicy();
    if (host_.is_bound()) {
      host_->SetFramePolicy(GetFramePolicy());
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
  auto* context = GetExecutionContext();
  if (!context) {
    return {};
  }
  PolicyParserMessageBuffer logger;
  auto policy = PermissionsPolicyParser::ParseAttribute(
      FastGetAttribute(html_names::kAllowAttr), *context->GetSecurityOrigin(),
      *MakeOriginForPermissionsPolicy(), logger, context);
  for (const auto& declaration : GetLegacyFramePolicies()) {
    if (std::ranges::none_of(policy, [&](const auto& item) {
          return item.feature == declaration.feature;
        })) {
      policy.push_back(declaration);
    }
  }
  for (const auto& message : logger.GetMessages()) {
    context->AddConsoleMessage(MakeGarbageCollected<ConsoleMessage>(
        mojom::blink::ConsoleMessageSource::kSecurity, message.level,
        message.content));
  }
  return policy;
}
scoped_refptr<const SecurityOrigin>
HTMLExplorerElement::MakeOriginForPermissionsPolicy() const {
  if (!src().empty()) {
    return SecurityOrigin::Create(GetDocument().CompleteURL(src()));
  }
  if (attached_ && ContentFrame()) {
    return ContentFrame()->GetSecurityContext()->GetSecurityOrigin();
  }
  return GetExecutionContext()->GetSecurityOrigin();
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
  params->isolated = isolated();
  params->context_menu_enabled = context_menu_enabled_;
  params->frame_policy = GetFramePolicy();
  return params;
}

void HTMLExplorerElement::EnsureAttached() {
  if (!isConnected() || create_pending_ || attached_ || detached_empty_ ||
      attach_resolver_ || !GetDocument().GetFrame()) {
    return;
  }
  if (!FastHasAttribute(html_names::kSrcAttr)) {
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
  if (auto* frame = GetDocument().GetFrame()) {
    auto* state = ToScriptStateForMainWorld(frame);
    if (state && state->ContextIsValid() &&
        !ScriptForbiddenScope::IsScriptForbidden()) {
      ScriptState::Scope scope(state);
      // Copy before abort dispatch: listeners can synchronously remove this
      // element.
      auto dialogs = std::move(dialogs_pending_);
      auto windows = std::move(windows_pending_);
      for (const auto& item : dialogs) {
        if (item.value) {
          item.value->Finish(state, true);
        }
      }
      for (const auto& item : windows) {
        if (item.value) {
          item.value->Finish(state, true);
        }
      }
    }
  }
  dialogs_pending_.clear();
  windows_pending_.clear();
  if (detach_resolver_) {
    detach_resolver_->RejectWithDOMException(
        DOMExceptionCode::kAbortError, "Explorer disconnected during detach");
    detach_resolver_.Clear();
  }
  if (attaching_handle_) {
    attaching_handle_->RestorePending();
    attaching_handle_.Clear();
  }
  replaced_handle_.Clear();
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
      state, type, true, false,
      ScriptValue(state->GetIsolate(), FromJSONString(state, json)));
  event->SetComposed(true);
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
  event->initCustomEvent(state, AtomicString("loaderror"), true, false,
                         ScriptValue(state->GetIsolate(), detail));
  event->SetComposed(true);
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
  auto parsed = FromJSONString(state, request_json);
  ExplorerNewWindow* request = nullptr;
  if (event_type == "newwindow" && parsed->IsObject()) {
    auto obj = parsed.As<v8::Object>();
    auto field = [&](const char* key) {
      return ToCoreString(
          state->GetIsolate(),
          obj->Get(state->GetContext(), V8String(state->GetIsolate(), key))
              .ToLocalChecked()
              .As<v8::String>());
    };
    auto token = field("token");
    request = MakeGarbageCollected<ExplorerNewWindow>(
        state, *this, token, field("url"), field("disposition"), field("from"),
        obj->Get(state->GetContext(),
                 V8String(state->GetIsolate(), "userGesture"))
            .ToLocalChecked()
            ->IsTrue());
    windows_pending_.Set(token, request);
    parsed = ToV8Traits<ExplorerNewWindow>::ToV8(state, request);
  }
  CustomEvent* event = CustomEvent::Create();
  event->initCustomEvent(state, AtomicString(event_type), true, true,
                         ScriptValue(state->GetIsolate(), parsed));
  event->SetComposed(true);
  event->SetComposed(true);
  DispatchEvent(*event);
  if (host_.is_bound()) {
    host_->CompleteOpen(request_id, !event->defaultPrevented() &&
                                        (!request || !request->responded()));
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
      state, *this, request_id, type, message, default_value, is_reload, from);
  dialogs_pending_.Set(String::Number(request_id), detail);
  CustomEvent* event = CustomEvent::Create();
  event->initCustomEvent(
      state, AtomicString("dialog"), true, true,
      ScriptValue(state->GetIsolate(),
                  ToV8Traits<ExplorerDialog>::ToV8(state, detail)));
  event->SetComposed(true);
  event->SetComposed(true);
  DispatchEvent(*event);
  if (host_.is_bound()) {
    host_->CompleteDialog(request_id,
                          !event->defaultPrevented() && !detail->responded());
  }
}
ScriptPromise<IDLBoolean> HTMLExplorerElement::DiscardWindow(
    ScriptState* state,
    const String& token) {
  return BooleanOperation(state, host_, [&](auto callback) {
    host_->DiscardWindow(token, std::move(callback));
  });
}
void HTMLExplorerElement::DidFinishRequest(const String& kind,
                                           const String& key,
                                           bool canceled) {
  if (!GetDocument().GetFrame()) {
    return;
  }
  auto* state = ToScriptStateForMainWorld(GetDocument().GetFrame());
  if (!state || !state->ContextIsValid()) {
    return;
  }
  ScriptState::Scope scope(state);
  if (kind == "dialog") {
    auto it = dialogs_pending_.find(key);
    if (it != dialogs_pending_.end()) {
      auto* request = it->value.Get();
      dialogs_pending_.erase(it);
      if (request) {
        request->Finish(state, canceled);
      }
    }
  } else {
    auto it = windows_pending_.find(key);
    if (it != windows_pending_.end()) {
      auto* request = it->value.Get();
      windows_pending_.erase(it);
      if (request) {
        request->Finish(state, canceled);
      }
    }
  }
}
}  // namespace blink
