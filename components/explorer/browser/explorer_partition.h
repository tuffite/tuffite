// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_PARTITION_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_PARTITION_H_

#include <optional>
#include <string>
#include <string_view>

namespace tuffite {

struct ExplorerPartition {
  enum class Mode { kShared, kMemory, kPersistent };
  static std::optional<ExplorerPartition> Parse(std::string_view value);
  bool shared() const { return mode == Mode::kShared; }
  bool in_memory() const { return mode == Mode::kMemory; }
  std::string_view mode_name() const;
  Mode mode = Mode::kShared;
  bool inherit = false;
  std::string name;
};
}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_PARTITION_H_
