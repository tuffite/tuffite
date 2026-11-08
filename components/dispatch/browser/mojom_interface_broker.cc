// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/dispatch/browser/mojom_interface_broker.h"

#include <algorithm>
#include <limits>
#include <map>
#include <memory>
#include <vector>

#include "base/compiler_specific.h"
#include "base/functional/bind.h"
#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "base/numerics/byte_conversions.h"
#include "base/task/sequenced_task_runner.h"
#include "content/public/browser/document_service.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/weak_document_ptr.h"
#include "mojo/public/cpp/bindings/connector.h"
#include "mojo/public/cpp/bindings/generic_pending_receiver.h"
#include "mojo/public/cpp/bindings/message_header_validator.h"
#include "tuffite/components/dispatch/browser/rpc_handler.h"

namespace tuffite {
namespace {
// The interface identity belongs to the bound pipe. Only its method ordinal
// and typed Mojom parameters cross that pipe; no string channel or RPC header.
class InterfaceReceiver final : public mojo::MessageReceiver {
 public:
  InterfaceReceiver(RpcHandler& handler,
                    content::RenderFrameHost& frame,
                    std::string name,
                    mojo::ScopedMessagePipeHandle pipe,
                    const base::UnguessableToken& connection,
                    uint64_t& next_operation)
      : handler_(handler),
        document_(frame.GetWeakDocumentPtr()),
        name_(std::move(name)),
        connection_(connection),
        next_operation_(next_operation),
        connector_(std::move(pipe), mojo::Connector::SINGLE_THREADED_SEND) {
    connector_.set_incoming_receiver(this);
    connector_.set_connection_error_handler(base::BindOnce(
        &InterfaceReceiver::Disconnected, weak_factory_.GetWeakPtr()));
    connector_.StartReceiving(base::SequencedTaskRunner::GetCurrentDefault());
  }
  ~InterfaceReceiver() override { Disconnected(); }
  bool closed() const { return closed_; }
  bool Accept(mojo::Message* message) override {
    message->SerializeIfNecessary();
    if (!validator_.Accept(message) || message->header()->version < 1 ||
        message->header()->name >= 0xfffffffeu ||
        message->header()->interface_id != 0 ||
        message->header()->flags != mojo::Message::kFlagExpectsResponse ||
        !message->handles()->empty() ||
        message->payload_num_bytes() > 16 * 1024 * 1024 ||
        pending_.size() >= 512 || pending_.contains(message->request_id()) ||
        *next_operation_ == 0) {
      return false;
    }
    auto* frame = document_.AsRenderFrameHostIfValid();
    if (!frame || !frame->IsActive()) {
      return false;
    }
    const auto ordinal = message->header()->name;
    auto method = methods_.find(ordinal);
    if (method == methods_.end()) {
      auto resolved = handler_->ResolveMojomMethod(name_, ordinal);
      if (!resolved) {
        return false;
      }
      method = methods_.emplace(ordinal, std::move(*resolved)).first;
    }
    // SAFETY: Mojo validated and owns this payload until Accept returns.
    auto bytes = UNSAFE_BUFFERS(
        base::span(message->payload(), message->payload_num_bytes()));
    const auto operation = (*next_operation_)++;
    pending_.emplace(message->request_id(), operation);
    handler_->DispatchMojom(
        *frame, method->second,
        std::vector<uint8_t>(bytes.begin(), bytes.end()), connection_,
        operation,
        base::BindOnce(&InterfaceReceiver::Reply, weak_factory_.GetWeakPtr(),
                       message->header()->name, message->request_id()));
    return true;
  }

 private:
  void Disconnected() {
    if (closed_) {
      return;
    }
    closed_ = true;
    weak_factory_.InvalidateWeakPtrs();
    auto pending = std::move(pending_);
    for (const auto& [wire_id, operation] : pending) {
      handler_->CancelMojomRequest(connection_, operation);
    }
  }
  void Reply(uint32_t ordinal,
             uint64_t request_id,
             int32_t status,
             const std::vector<uint8_t>& result) {
    if (!pending_.erase(request_id) || !document_.AsRenderFrameHostIfValid()) {
      return;
    }
    // This is the canonical generated (status, typed value, error) response.
    // The pointed-to value is the method's generated, validated Mojom struct.
    if (result.size() > 16 * 1024 * 1024) {
      status = 6;
    }
    std::vector<uint8_t> bytes(32);
    auto output = base::span(bytes);
    output.first<4>().copy_from(base::U32ToLittleEndian(32));
    output.subspan<8, 4>().copy_from(
        base::U32ToLittleEndian(static_cast<uint32_t>(status)));
    if (status == 0) {
      output.subspan<16, 8>().copy_from(base::U64ToLittleEndian(16));
      bytes.insert(bytes.end(), result.begin(), result.end());
    } else if (!result.empty() && result.size() <= 16 * 1024 * 1024) {
      output.subspan<24, 8>().copy_from(base::U64ToLittleEndian(8));
      const auto offset = bytes.size();
      bytes.resize(offset + 8 + result.size());
      auto text = base::span(bytes).subspan(offset);
      text.first<4>().copy_from(
          base::U32ToLittleEndian(static_cast<uint32_t>(8 + result.size())));
      text.subspan<4, 4>().copy_from(
          base::U32ToLittleEndian(static_cast<uint32_t>(result.size())));
      text.subspan<8>().copy_from(result);
      bytes.resize((bytes.size() + 7) & ~size_t{7});
    }
    mojo::Message reply(ordinal, mojo::Message::kFlagIsResponse, bytes.size(),
                        0, nullptr);
    reply.set_request_id(request_id);
    // Reserve and advance the message buffer, as generated Mojo serializers do.
    auto* payload = static_cast<uint8_t*>(
        reply.payload_buffer()->AllocateAndGet(bytes.size()));
    // SAFETY: AllocateAndGet returns the requested writable payload.
    UNSAFE_BUFFERS(base::span(payload, bytes.size())).copy_from(bytes);
    connector_.Accept(&reply);
  }
  const raw_ref<RpcHandler> handler_;
  const content::WeakDocumentPtr document_;
  const std::string name_;
  const base::UnguessableToken connection_;
  const raw_ref<uint64_t> next_operation_;
  bool closed_ = false;
  std::map<uint64_t, uint64_t> pending_;
  std::map<uint32_t, MojomMethod> methods_;
  mojo::MessageHeaderValidator validator_;
  mojo::Connector connector_;
  base::WeakPtrFactory<InterfaceReceiver> weak_factory_{this};
};

class InterfaceBroker final
    : public content::DocumentService<blink::mojom::BrowserInterfaceBroker> {
 public:
  InterfaceBroker(
      RpcHandler& handler,
      content::RenderFrameHost& frame,
      mojo::PendingReceiver<blink::mojom::BrowserInterfaceBroker> receiver)
      : DocumentService(frame, std::move(receiver)), handler_(handler) {}
  ~InterfaceBroker() override {
    receivers_.clear();
    handler_->CancelDocument(connection_);
  }
  void GetInterface(mojo::GenericPendingReceiver receiver) override {
    std::erase_if(receivers_,
                  [](const auto& entry) { return entry->closed(); });
    if (!handler_->IsAuthorized(render_frame_host()) ||
        receivers_.size() >= 1024 || !receiver.interface_name() ||
        receiver.interface_name()->size() > 4096) {
      return;
    }
    const std::string name = *receiver.interface_name();
    if (!handler_->ResolveMojomMethod(name,
                                      std::numeric_limits<uint32_t>::max())) {
      return;
    }
    receivers_.push_back(std::make_unique<InterfaceReceiver>(
        *handler_, render_frame_host(), name, receiver.PassPipe(), connection_,
        next_operation_));
  }

 private:
  const raw_ref<RpcHandler> handler_;
  const base::UnguessableToken connection_ = base::UnguessableToken::Create();
  uint64_t next_operation_ = 1;
  std::vector<std::unique_ptr<InterfaceReceiver>> receivers_;
};
}  // namespace
void BindMojomInterfaceBroker(
    RpcHandler* handler,
    content::RenderFrameHost* frame,
    mojo::PendingReceiver<blink::mojom::BrowserInterfaceBroker> receiver) {
  if (handler && frame && handler->IsAuthorized(*frame)) {
    new InterfaceBroker(*handler, *frame, std::move(receiver));
  }
}
}  // namespace tuffite
