// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_HANDLER_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_HANDLER_H_
#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "base/functional/callback.h"
#include "base/unguessable_token.h"
#include "tuffite/components/dispatch/browser/host_dispatch.h"

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
  virtual std::optional<MojomMethod> ResolveMojomMethod(std::string_view,
                                                        uint32_t) const = 0;
  virtual void DispatchMojom(content::RenderFrameHost&,
                             const MojomMethod&,
                             const std::vector<uint8_t>&,
                             const base::UnguessableToken&,
                             uint64_t,
                             Reply) = 0;
  virtual void CancelMojomRequest(const base::UnguessableToken&, uint64_t) = 0;
  virtual bool IsAuthorized(content::RenderFrameHost&) const = 0;
  virtual bool IsEventAuthorized(content::RenderFrameHost&) const = 0;
  virtual void CancelDocument(const base::UnguessableToken&) = 0;
  virtual void Subscribe(const base::UnguessableToken&, Event) = 0;
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_HANDLER_H_
