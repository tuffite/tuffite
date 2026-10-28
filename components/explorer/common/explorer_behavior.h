// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_EXPLORER_COMMON_EXPLORER_BEHAVIOR_H_
#define TUFFITE_COMPONENTS_EXPLORER_COMMON_EXPLORER_BEHAVIOR_H_

#include <array>
#include <map>
#include <optional>
#include <string_view>

namespace tuffite {
enum class ExplorerAction {
  kNavigation,
  kDialog,
  kExternalProtocol,
  kNewWindow
};
enum class ExplorerMode { kBrowser, kDelegate, kBlock };
struct ExplorerBehaviorEntry {
  ExplorerMode mode = ExplorerMode::kBrowser;
  int timeout_ms = 30000;
  bool operator==(const ExplorerBehaviorEntry&) const = default;
};
using ExplorerBehaviorOverrides =
    std::map<ExplorerAction, ExplorerBehaviorEntry>;
struct ExplorerBehavior {
  std::array<ExplorerBehaviorEntry, 4> entries;
  const ExplorerBehaviorEntry& operator[](ExplorerAction action) const {
    return entries[static_cast<size_t>(action)];
  }
  void Apply(const ExplorerBehaviorOverrides& overrides) {
    for (const auto& [action, entry] : overrides) {
      entries[static_cast<size_t>(action)] = entry;
    }
  }
  bool operator==(const ExplorerBehavior&) const = default;
};
inline std::string_view TrimExplorerBehavior(std::string_view value) {
  const auto first = value.find_first_not_of(" \t\r\n\f");
  if (first == value.npos) {
    return {};
  }
  return value.substr(first, value.find_last_not_of(" \t\r\n\f") - first + 1);
}
inline std::optional<ExplorerAction> ParseExplorerAction(
    std::string_view name) {
  if (name == "navigation") {
    return ExplorerAction::kNavigation;
  }
  if (name == "dialog") {
    return ExplorerAction::kDialog;
  }
  if (name == "external-protocol") {
    return ExplorerAction::kExternalProtocol;
  }
  if (name == "new-window") {
    return ExplorerAction::kNewWindow;
  }
  return std::nullopt;
}
inline std::optional<ExplorerMode> ParseExplorerMode(std::string_view mode) {
  if (mode == "browser") {
    return ExplorerMode::kBrowser;
  }
  if (mode == "delegate") {
    return ExplorerMode::kDelegate;
  }
  if (mode == "block") {
    return ExplorerMode::kBlock;
  }
  return std::nullopt;
}
// A deliberately small declaration grammar, not a CSS stylesheet or cascade.
// Parsing is atomic: unknown operations/values never fall back to browser mode.
inline std::optional<ExplorerBehaviorOverrides> ParseExplorerBehavior(
    std::string_view text) {
  if (text.size() > 4096) {
    return std::nullopt;
  }
  ExplorerBehaviorOverrides result;
  while (!text.empty()) {
    const auto separator = text.find(';');
    auto declaration = TrimExplorerBehavior(text.substr(0, separator));
    text = separator == text.npos ? std::string_view()
                                  : text.substr(separator + 1);
    if (declaration.empty()) {
      continue;
    }
    const auto colon = declaration.find(':');
    if (colon == declaration.npos) {
      return std::nullopt;
    }
    auto action =
        ParseExplorerAction(TrimExplorerBehavior(declaration.substr(0, colon)));
    if (!action) {
      return std::nullopt;
    }
    auto value = TrimExplorerBehavior(declaration.substr(colon + 1));
    const auto space = value.find_first_of(" \t\r\n\f");
    auto mode = ParseExplorerMode(value.substr(0, space));
    if (!mode) {
      return std::nullopt;
    }
    ExplorerBehaviorEntry entry{.mode = *mode};
    if (space != value.npos) {
      auto parameter = TrimExplorerBehavior(value.substr(space));
      if (!parameter.starts_with("timeout(") || !parameter.ends_with(')') ||
          *mode != ExplorerMode::kDelegate ||
          *action == ExplorerAction::kNavigation) {
        return std::nullopt;
      }
      auto duration = parameter.substr(8, parameter.size() - 9);
      int multiplier = 1;
      if (duration.ends_with("ms")) {
        duration.remove_suffix(2);
      } else if (duration.ends_with('s')) {
        duration.remove_suffix(1);
        multiplier = 1000;
      } else {
        return std::nullopt;
      }
      if (duration.empty() || duration.size() > 6) {
        return std::nullopt;
      }
      int amount = 0;
      for (char c : duration) {
        if (c < '0' || c > '9') {
          return std::nullopt;
        }
        amount = amount * 10 + c - '0';
      }
      if (amount < 1 || amount > 300000 / multiplier) {
        return std::nullopt;
      }
      entry.timeout_ms = amount * multiplier;
    }
    result.insert_or_assign(*action, entry);
  }
  return result;
}
}  // namespace tuffite
#endif
