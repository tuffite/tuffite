// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_partition.h"

#include <algorithm>

#include "base/strings/string_util.h"

namespace tuffite {
std::optional<ExplorerPartition> ExplorerPartition::Parse(
    std::string_view value) {
  if (value == "shared") {
    return ExplorerPartition{Mode::kShared, false, {}};
  }
  struct Prefix {
    std::string_view value;
    Mode mode;
    bool inherit;
  };
  static constexpr Prefix kPrefixes[] = {
      {"memory:", Mode::kMemory, false},
      {"persist:", Mode::kPersistent, false},
      {"inherit+memory:", Mode::kMemory, true},
      {"inherit+persist:", Mode::kPersistent, true},
  };
  for (const auto& prefix : kPrefixes) {
    if (!value.starts_with(prefix.value)) {
      continue;
    }
    const std::string_view name = value.substr(prefix.value.size());
    if (name.empty() || name.size() > 128 ||
        !std::ranges::all_of(name, [](char character) {
          return base::IsAsciiAlphaNumeric(character) || character == '-' ||
                 character == '_' || character == '.';
        })) {
      return std::nullopt;
    }
    return ExplorerPartition{prefix.mode, prefix.inherit, std::string(name)};
  }
  return std::nullopt;
}
std::string_view ExplorerPartition::mode_name() const {
  if (shared()) {
    return "shared";
  }
  if (inherit) {
    return in_memory() ? "inherit+memory" : "inherit+persist";
  }
  return in_memory() ? "memory" : "persist";
}
}  // namespace tuffite
