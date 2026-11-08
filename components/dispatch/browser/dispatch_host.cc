// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/dispatch_host.h"

#include "base/functional/bind.h"
#include "tuffite/components/dispatch/browser/rpc_handler.h"
namespace tuffite {
void DispatchHost::Create(RpcHandler* handler,
                          content::RenderFrameHost* frame,
                          mojo::PendingReceiver<mojom::EventSource> receiver) {
  if (handler && frame) {
    new DispatchHost(*handler, *frame, std::move(receiver));
  }
}
DispatchHost::DispatchHost(RpcHandler& handler,
                           content::RenderFrameHost& frame,
                           mojo::PendingReceiver<mojom::EventSource> receiver)
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
