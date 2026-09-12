// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/dispatch_host.h"

#include "base/functional/bind.h"
#include "tuffite/components/dispatch/browser/rpc_handler.h"
namespace tuffite {
void DispatchHost::Create(RpcHandler* handler,
                          content::RenderFrameHost* frame,
                          mojo::PendingReceiver<mojom::Dispatcher> receiver) {
  if (handler && frame) {
    new DispatchHost(*handler, *frame, std::move(receiver));
  }
}
DispatchHost::DispatchHost(RpcHandler& handler,
                           content::RenderFrameHost& frame,
                           mojo::PendingReceiver<mojom::Dispatcher> receiver)
    : DocumentService(frame, std::move(receiver)), handler_(handler) {
  handler_->Subscribe(document_,
                      base::BindRepeating(&DispatchHost::DispatchChannelEvent,
                                          weak_factory_.GetWeakPtr()));
}
DispatchHost::~DispatchHost() {
  weak_factory_.InvalidateWeakPtrs();
  handler_->CancelDocument(document_);
}
bool DispatchHost::IsAuthorized() const {
  return handler_->IsAuthorized(render_frame_host());
}
void DispatchHost::Dispatch(const std::string& channel,
                            const std::vector<uint8_t>& payload,
                            DispatchCallback callback) {
  handler_->Dispatch(render_frame_host(), channel, payload, document_,
                     std::move(callback));
}
void DispatchHost::DispatchRequest(uint64_t id,
                                   const std::string& channel,
                                   const std::vector<uint8_t>& bytes,
                                   DispatchRequestCallback reply) {
  handler_->DispatchRequest(render_frame_host(), id, channel, bytes, document_,
                            std::move(reply));
}
void DispatchHost::CancelRequest(uint64_t id) {
  handler_->CancelRequest(document_, id);
}
void DispatchHost::BindClient(
    mojo::PendingRemote<mojom::DispatchClient> client) {
  if (IsAuthorized()) {
    client_.Bind(std::move(client));
  }
}
void DispatchHost::DispatchChannelEvent(const std::string& channel,
                                        const std::vector<uint8_t>& payload) {
  if (client_ && handler_->IsEventAuthorized(render_frame_host())) {
    client_->DispatchChannelEvent(channel, payload);
  }
}
}  // namespace tuffite
