// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_REQUEST_REGISTRY_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_REQUEST_REGISTRY_H_
#include <cstdint>
#include <map>
#include <optional>
#include <tuple>

#include "base/memory/weak_ptr.h"
#include "tuffite/components/dispatch/browser/rpc_handler.h"
namespace tuffite::rpc {
struct RequestKey {
  base::UnguessableToken connection;
  uint64_t id;
  bool operator<(const RequestKey& other) const {
    return std::tie(connection, id) < std::tie(other.connection, other.id);
  }
};
class RequestRegistry final {
 public:
  using Reply = RpcHandler::Reply;
  RequestRegistry();
  ~RequestRegistry();
  std::optional<Reply> Admit(RequestKey, Reply, base::OnceClosure cancel);
  bool Cancel(RequestKey);
  void CancelConnection(const base::UnguessableToken&);
  void Shutdown();
  size_t size() const { return pending_.size(); }

 private:
  struct Pending {
    uint64_t generation;
    Reply reply;
    base::OnceClosure cancel;
  };
  void Complete(RequestKey,
                uint64_t generation,
                int32_t,
                const std::vector<uint8_t>&);
  std::map<RequestKey, Pending> pending_;
  bool stopped_ = false;
  uint64_t next_generation_ = 1;
  base::WeakPtrFactory<RequestRegistry> weak_factory_{this};
};
}  // namespace tuffite::rpc

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_REQUEST_REGISTRY_H_
