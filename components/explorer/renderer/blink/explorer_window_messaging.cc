// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "mojo/public/cpp/bindings/remote.h"
#include "third_party/blink/public/platform/browser_interface_broker_proxy.h"
#include "third_party/blink/renderer/bindings/core/v8/native_value_traits_impl.h"
#include "third_party/blink/renderer/bindings/core/v8/serialization/post_message_helper.h"
#include "third_party/blink/renderer/bindings/core/v8/serialization/transferables.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_binding_for_core.h"
#include "third_party/blink/renderer/bindings/core/v8/v8_window_post_message_options.h"
#include "third_party/blink/renderer/core/frame/local_dom_window.h"
#include "third_party/blink/renderer/core/frame/local_frame.h"
#include "third_party/blink/renderer/core/messaging/blink_transferable_message.h"
#include "third_party/blink/renderer/core/messaging/message_port.h"
#include "tuffite/components/explorer/common/explorer.mojom-blink.h"
#include "tuffite/components/explorer/renderer/blink/explorer_handle_bridge.h"
#include "tuffite/components/explorer/renderer/blink/explorer_message_serialization.h"

namespace blink {
BlinkTransferableMessage SerializeExplorerMessage(
    ScriptState* state,
    const ScriptValue& value,
    const WindowPostMessageOptions* options,
    scoped_refptr<const SecurityOrigin>& origin,
    ExceptionState& exception) {
  auto* context = ExecutionContext::From(state);
  BlinkTransferableMessage message;
  origin = PostMessageHelper::GetTargetOrigin(options, *context, exception);
  if (exception.HadException()) {
    return message;
  }
  Transferables transferables;
  message.message = PostMessageHelper::SerializeMessageByMove(
      state->GetIsolate(), value, options, transferables, exception);
  if (exception.HadException()) {
    return message;
  }
  message.ports = MessagePort::DisentanglePorts(
      context, transferables.message_ports, exception);
  if (exception.HadException()) {
    return message;
  }
  message.sender_origin = context->GetSecurityOrigin()->IsolatedCopy();
  message.sender_agent_cluster_id = context->GetAgentClusterID();
  message.locked_to_sender_agent_cluster =
      message.message->IsLockedToAgentCluster();
  return message;
}
namespace {
void Post(const v8::FunctionCallbackInfo<v8::Value>& info, bool window_target) {
  auto* isolate = info.GetIsolate();
  auto* state = ScriptState::From(isolate, isolate->GetCurrentContext());
  ExceptionState exception(isolate);
  const int offset = window_target ? 1 : 0;
  if (info.Length() < offset + 1 || (window_target && !info[0]->IsBigInt())) {
    exception.ThrowTypeError("Expected a message");
    return;
  }
  bool lossless = true;
  int64_t id =
      window_target ? info[0].As<v8::BigInt>()->Int64Value(&lossless) : 0;
  if (!lossless) {
    exception.ThrowTypeError("Invalid window id");
    return;
  }
  auto* context = ExecutionContext::From(state);
  auto* window = DynamicTo<LocalDOMWindow>(context);
  if (!window || !window->GetFrame()) {
    exception.ThrowDOMException(DOMExceptionCode::kInvalidStateError,
                                "Document is inactive");
    return;
  }
  auto* options = WindowPostMessageOptions::Create();
  // Match Window.postMessage's overload selection: a third public argument
  // selects the positional overload; otherwise null/undefined/objects select
  // the dictionary overload, and primitives are converted to USVString.
  const int option_index = offset + 1;
  const bool positional =
      info.Length() > option_index + 1 ||
      (info.Length() > option_index && !info[option_index]->IsObject() &&
       !info[option_index]->IsNullOrUndefined());
  if (positional) {
    options->setTargetOrigin(NativeValueTraits<IDLUSVString>::NativeValue(
        isolate, info[option_index], exception));
    if (!exception.HadException() && info.Length() > option_index + 1 &&
        !info[option_index + 1]->IsUndefined()) {
      options->setTransfer(
          NativeValueTraits<IDLSequence<IDLObject>>::NativeValue(
              isolate, info[option_index + 1], exception));
    }
  } else if (info.Length() > option_index) {
    options = NativeValueTraits<WindowPostMessageOptions>::NativeValue(
        isolate, info[option_index], exception);
  }
  if (exception.HadException()) {
    return;
  }
  if (!window_target) {
    window->top()->postMessage(isolate, ScriptValue(isolate, info[offset]),
                               options, exception);
    return;
  }
  scoped_refptr<const SecurityOrigin> origin;
  auto message = SerializeExplorerMessage(
      state, ScriptValue(isolate, info[offset]), options, origin, exception);
  if (exception.HadException()) {
    return;
  }
  mojo::Remote<tuffite::mojom::blink::ExplorerSurfaceHost> host;
  window->GetFrame()->GetBrowserInterfaceBroker().GetInterface(
      host.BindNewPipeAndPassReceiver());
  if (window_target) {
    host->PostWindowMessage(id, std::move(message),
                            origin ? origin->ToString() : "*");
  }
}
}  // namespace
void PostExplorerWindowMessage(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  Post(info, true);
}
void PostExplorerSurfaceMessage(
    const v8::FunctionCallbackInfo<v8::Value>& info) {
  Post(info, false);
}
}  // namespace blink
