// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_HANDLER_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_HANDLER_H_
#include <string>
#include <vector>

#include "base/functional/callback.h"
#include "base/unguessable_token.h"

namespace content {
class RenderFrameHost;
}

namespace tuffite {

class RpcHandler {
 public:
  using Reply = base::OnceCallback<void(int32_t, const std::vector<uint8_t>&)>;
  using Event = base::RepeatingCallback<void(const std::string&,
                                             const std::vector<uint8_t>&)>;
  virtual ~RpcHandler() = default;
  virtual bool IsAuthorized(content::RenderFrameHost&) const = 0;
  virtual bool IsEventAuthorized(content::RenderFrameHost&) const = 0;
  virtual void Dispatch(content::RenderFrameHost&,
                        const std::string&,
                        const std::vector<uint8_t>&,
                        const base::UnguessableToken&,
                        Reply) = 0;
  virtual void DispatchRequest(content::RenderFrameHost&,
                               uint64_t,
                               const std::string&,
                               const std::vector<uint8_t>&,
                               const base::UnguessableToken&,
                               Reply) = 0;
  virtual void CancelRequest(const base::UnguessableToken&, uint64_t) = 0;
  virtual void CancelDocument(const base::UnguessableToken&) = 0;
  virtual void Subscribe(const base::UnguessableToken&, Event) = 0;
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_HANDLER_H_
