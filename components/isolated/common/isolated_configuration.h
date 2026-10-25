// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_ISOLATED_COMMON_ISOLATED_CONFIGURATION_H_
#define TUFFITE_COMPONENTS_ISOLATED_COMMON_ISOLATED_CONFIGURATION_H_
#include <string>
#include <string_view>
#include <vector>

namespace tuffite::isolated {
inline bool IsValidWorldName(std::string_view name) {
  const auto alpha = [](char ch) {
    return (ch >= 'A' && ch <= 'Z') || (ch >= 'a' && ch <= 'z');
  };
  if (name.empty() || name.size() > 128 || !alpha(name.front())) {
    return false;
  }
  for (char ch : name) {
    if (!alpha(ch) && !(ch >= '0' && ch <= '9') && ch != '_' &&
        ch != '-' && ch != '.') {
      return false;
    }
  }
  return true;
}
struct WorldDefinition {
  std::vector<std::string> origins;
  std::string source;
};
}  // namespace tuffite::isolated
#endif
