// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_DISPATCH_HOST_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_DISPATCH_HOST_H_

#include <string>
#include <vector>

#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "base/unguessable_token.h"
#include "content/public/browser/document_service.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/dispatch/common/dispatcher.mojom.h"

namespace content {
class RenderFrameHost;
}

namespace tuffite {
class RpcHandler;

class DispatchHost final : public content::DocumentService<mojom::Dispatcher> {
 public:
  static void Create(RpcHandler* handler,
                     content::RenderFrameHost* frame,
                     mojo::PendingReceiver<mojom::Dispatcher> receiver);

  void Dispatch(const std::string& channel,
                const std::vector<uint8_t>& payload,
                DispatchCallback callback) override;
  void DispatchRequest(uint64_t request_id,
                       const std::string& channel,
                       const std::vector<uint8_t>& payload,
                       DispatchRequestCallback callback) override;
  void CancelRequest(uint64_t request_id) override;
  void BindClient(mojo::PendingRemote<mojom::DispatchClient> client) override;

 private:
  DispatchHost(RpcHandler& handler,
               content::RenderFrameHost& frame,
               mojo::PendingReceiver<mojom::Dispatcher> receiver);
  ~DispatchHost() override;

  bool IsAuthorized() const;
  void DispatchChannelEvent(const std::string& channel,
                            const std::vector<uint8_t>& payload);

  const raw_ref<RpcHandler> handler_;
  mojo::Remote<mojom::DispatchClient> client_;
  const base::UnguessableToken document_ = base::UnguessableToken::Create();
  base::WeakPtrFactory<DispatchHost> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_DISPATCH_HOST_H_
