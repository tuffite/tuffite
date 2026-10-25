// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_ISOLATED_COMMON_ISOLATED_CONFIGURATION_H_
#define TUFFITE_COMPONENTS_ISOLATED_COMMON_ISOLATED_CONFIGURATION_H_
#include <string>
#include <vector>
namespace tuffite::isolated {
struct WorldDefinition {
  std::vector<std::string> origins;
  std::string source;
};
}  // namespace tuffite::isolated
#endif
