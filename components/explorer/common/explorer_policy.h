// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_EXPLORER_COMMON_EXPLORER_POLICY_H_
#define TUFFITE_COMPONENTS_EXPLORER_COMMON_EXPLORER_POLICY_H_

#include <array>
#include <cstdint>
#include <map>
#include <optional>
#include <string_view>

#include "tuffite/components/explorer/common/explorer_declarations.h"

namespace tuffite {
enum class ExplorerAction {
  kNavigation,
  kDialog,
  kExternalProtocol,
  kNewWindow,
  kContextMenu
};
enum class ExplorerMode { kBrowser, kDelegate, kBlock };
struct ExplorerActionPolicy {
  ExplorerMode mode = ExplorerMode::kBrowser;
  int timeout_ms = 30000;
  bool operator==(const ExplorerActionPolicy&) const = default;
};
struct ExplorerDelay {
  enum class Mode { kDefault, kNever, kAfter };
  Mode mode = Mode::kDefault;
  int64_t milliseconds = 0;
  std::optional<int64_t> Resolve(int64_t default_ms) const;
  bool operator==(const ExplorerDelay&) const = default;
};
inline constexpr int64_t kExplorerDefaultFreezeMs = 30000;
inline constexpr int64_t kExplorerDefaultDiscardMs = 900000;
inline constexpr int64_t kExplorerMaximumDelayMs = 86400000;

struct ExplorerPolicyOverrides {
  bool operator==(const ExplorerPolicyOverrides&) const = default;
  std::map<ExplorerAction, ExplorerActionPolicy> actions;
  std::optional<ExplorerDelay> freeze;
  std::optional<ExplorerDelay> discard;
};
struct ExplorerPolicySettings {
  ExplorerPolicySettings();
  std::array<ExplorerActionPolicy, 5> actions;
  ExplorerDelay freeze;
  ExplorerDelay discard;
  const ExplorerActionPolicy& operator[](ExplorerAction action) const;
  void Apply(const ExplorerPolicyOverrides& overrides);
  bool operator==(const ExplorerPolicySettings&) const = default;
};
std::optional<ExplorerAction> ParseExplorerAction(std::string_view name);
std::optional<ExplorerMode> ParseExplorerMode(std::string_view mode);
std::optional<ExplorerPolicyOverrides> ParseExplorerPolicy(
    std::string_view text,
    ExplorerDeclarationError* error = nullptr);
}  // namespace tuffite
#endif
