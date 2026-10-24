// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/rpc.h"

#include <algorithm>
#include <utility>

#include "base/check.h"

namespace tuffite::rpc {
Router::~Router() = default;

Router::Router(std::vector<Service> services) {
  services_.reserve(services.size());
  for (auto& service : services) {
    CHECK(service.dispatch);
    services_.push_back(
        {std::string(service.name), std::move(service.dispatch)});
  }
  std::ranges::sort(services_, {}, &Entry::name);
  CHECK(std::ranges::adjacent_find(services_, {}, &Entry::name) ==
        services_.end());
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
  const auto separator = method.rfind('.');
  if (separator == std::string_view::npos) {
    return false;
  }
  const auto interface = method.substr(0, separator);
  const auto it =
      std::ranges::lower_bound(services_, interface, {}, &Entry::name);
  if (it == services_.end() || it->name != interface) {
    return false;
  }
  return it->dispatch.Run(context, method.substr(separator + 1), payload,
                          reply);
}

}  // namespace tuffite::rpc
