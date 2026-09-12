// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/request_registry.h"

#include "base/functional/bind.h"
#include "tuffite/components/dispatch/common/status.h"
namespace tuffite::rpc {
RequestRegistry::RequestRegistry() = default;
RequestRegistry::~RequestRegistry() {
  Shutdown();
}

std::optional<RequestRegistry::Reply>
RequestRegistry::Admit(RequestKey key, Reply reply, base::OnceClosure cancel) {
  size_t count = 0;
  for (const auto& [k, p] : pending_) {
    if (k.connection == key.connection) {
      ++count;
    }
  }
  if (stopped_ || key.id == 0 || pending_.contains(key) || count >= 512 ||
      next_generation_ == 0) {
    const auto status = stopped_ ? Status::kUnavailable
                        : (key.id == 0 || pending_.contains(key))
                            ? Status::kInvalidArgument
                            : Status::kResourceExhausted;
    std::move(reply).Run(static_cast<int32_t>(status), {});
    return std::nullopt;
  }
  const uint64_t generation = next_generation_++;
  pending_.emplace(key,
                   Pending{generation, std::move(reply), std::move(cancel)});
  return Reply(base::BindOnce(&RequestRegistry::Complete,
                              weak_factory_.GetWeakPtr(), key, generation));
}
void RequestRegistry::Complete(RequestKey key,
                               uint64_t generation,
                               int32_t status,
                               const std::vector<uint8_t>& bytes) {
  auto found = pending_.find(key);
  if (found == pending_.end() || found->second.generation != generation) {
    return;
  }
  auto node = pending_.extract(found);
  std::move(node.mapped().reply).Run(status, bytes);
}
bool RequestRegistry::Cancel(RequestKey key) {
  auto node = pending_.extract(key);
  if (node.empty()) {
    return false;
  }
  if (node.mapped().cancel) {
    std::move(node.mapped().cancel).Run();
  }
  std::move(node.mapped().reply)
      .Run(static_cast<int32_t>(Status::kCancelled), {});
  return true;
}
void RequestRegistry::CancelConnection(
    const base::UnguessableToken& connection) {
  std::vector<RequestKey> keys;
  for (const auto& [key, p] : pending_) {
    if (key.connection == connection) {
      keys.push_back(key);
    }
  }
  auto weak = weak_factory_.GetWeakPtr();
  for (auto key : keys) {
    if (!weak) {
      break;
    }
    weak->Cancel(key);
  }
}
void RequestRegistry::Shutdown() {
  if (stopped_) {
    return;
  }
  stopped_ = true;
  auto requests = std::move(pending_);
  weak_factory_.InvalidateWeakPtrs();
  for (auto& [key, p] : requests) {
    if (p.cancel) {
      std::move(p.cancel).Run();
    }
    std::move(p.reply).Run(static_cast<int32_t>(Status::kUnavailable), {});
  }
}
}  // namespace tuffite::rpc
