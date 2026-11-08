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
    std::string wire_name(service.name);
    const auto split = wire_name.rfind('.');
    if (split != std::string::npos) {
      wire_name.insert(split, ".wire");
    }
    services_.push_back({std::string(service.name), std::move(wire_name),
                         std::move(service.dispatch),
                         std::move(service.methods)});
  }
  std::ranges::sort(services_, {}, &Entry::name);
  CHECK(std::ranges::adjacent_find(services_, {}, &Entry::name) ==
        services_.end());
}

std::optional<std::string> Router::ResolveMethod(std::string_view name,
                                                 uint32_t ordinal) const {
  const auto entry = std::ranges::find(services_, name, &Entry::wire_name);
  if (entry == services_.end()) {
    return std::nullopt;
  }
  for (const auto& method : entry->methods) {
    if (ordinal == UINT32_MAX || method.ordinal == ordinal) {
      return entry->name + "." + std::string(method.name);
    }
  }
  return std::nullopt;
}

bool Router::Dispatch(const dispatch::RendererRequestContext& context,
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
