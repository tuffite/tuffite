// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/rpc_router.h"

#include <algorithm>
#include <utility>

#include "base/check.h"
#include "tuffite/components/dispatch/common/status.h"

namespace tuffite::rpc {
Router::~Router() = default;

Router::Router(std::vector<Route> routes) {
  routes_.reserve(routes.size());
  for (auto& route : routes) {
    routes_.push_back({std::string(route.path), std::move(route.handler)});
  }
  std::ranges::sort(routes_, {}, &Entry::path);
  CHECK(std::ranges::adjacent_find(routes_, {}, &Entry::path) == routes_.end());
}

bool Router::Dispatch(content::RenderFrameHost& frame,
                      std::string_view method,
                      base::span<const uint8_t> payload,
                      const base::UnguessableToken& document,
                      Reply* reply) const {
  return Dispatch(dispatch::Context(frame, document), method, payload, reply);
}

bool Router::Dispatch(const dispatch::Context& context,
                      std::string_view method,
                      base::span<const uint8_t> payload,
                      Reply* reply) const {
  CHECK(reply);
  const auto it = std::ranges::lower_bound(routes_, method, {}, &Entry::path);
  if (it == routes_.end() || it->path != method) {
    return false;
  }
  it->handler.Run(context, method, payload, std::move(*reply));
  return true;
}

}  // namespace tuffite::rpc
