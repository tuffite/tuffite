// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_ISOLATED_ISOLATED_REGISTRY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_ISOLATED_ISOLATED_REGISTRY_H_
#include <cstdint>
#include <map>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "base/values.h"
#include "tuffite/components/isolated/common/isolated_configuration.h"
#include "url/origin.h"

namespace tuffite {
// Browser-sequence registry. Entries own immutable script snapshots. Replacing
// or removing an entry affects future loads, never already evaluated worlds.
class IsolatedRegistry {
 public:
  IsolatedRegistry();
  ~IsolatedRegistry();
  IsolatedRegistry(const IsolatedRegistry&) = delete;
  IsolatedRegistry& operator=(const IsolatedRegistry&) = delete;
  using Definition = isolated::WorldDefinition;
  void SetDevelopmentAlias(const url::Origin& packaged,
                           const url::Origin& development);
  void AddBundled(const std::string& name, const Definition& definition);
  std::optional<std::string> Source(std::string_view name,
                                    const url::Origin& origin) const;
  // Native application control plane only; never dispatched by renderer RPC.
  // Returns a generation as decimal text, or nullopt for invalid/stale
  // requests.
  std::optional<std::string> Apply(const base::DictValue& request);

 private:
  struct Entry {
    Definition definition;
    uint64_t generation;
    bool bundled;
  };
  std::map<std::string, Entry> entries_;
  uint64_t next_generation_ = 1;
  std::optional<url::Origin> packaged_origin_;
  std::optional<url::Origin> development_origin_;
};
}  // namespace tuffite
#endif
