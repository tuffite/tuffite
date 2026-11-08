// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/common/explorer_policy.h"

#include <utility>

#include "base/strings/string_number_conversions.h"

namespace tuffite {
namespace {
using Kind = ExplorerValue::Kind;
std::optional<int64_t> Duration(const ExplorerValue& value, int64_t maximum) {
  if (value.kind != Kind::kDimension) {
    return std::nullopt;
  }
  std::string_view number = value.text;
  int64_t multiplier;
  if (number.ends_with("ms")) {
    number.remove_suffix(2);
    multiplier = 1;
  } else if (number.ends_with('s')) {
    number.remove_suffix(1);
    multiplier = 1000;
  } else if (number.ends_with('m')) {
    number.remove_suffix(1);
    multiplier = 60000;
  } else if (number.ends_with('h')) {
    number.remove_suffix(1);
    multiplier = 3600000;
  } else {
    return std::nullopt;
  }
  // Parse decimal durations exactly: floating-point scaling can turn 1.001s
  // into 1000.9999999999999ms and reject an otherwise integral duration.
  if (number.starts_with('+')) {
    number.remove_prefix(1);
  }
  if (number.starts_with('-')) {
    return std::nullopt;
  }
  const auto point = number.find('.');
  const auto whole_text = number.substr(0, point);
  int64_t whole = 0;
  if ((!whole_text.empty() && !base::StringToInt64(whole_text, &whole)) ||
      whole > maximum / multiplier) {
    return std::nullopt;
  }
  int64_t remainder = 0;
  if (point != number.npos) {
    auto fraction = number.substr(point + 1);
    while (fraction.ends_with('0')) {
      fraction.remove_suffix(1);
    }
    // All supported units have fewer than 10 decimal factors. Bound the
    // numerator before multiplication, even for arbitrarily long input.
    if (fraction.size() > 9) {
      return std::nullopt;
    }
    if (!fraction.empty()) {
      int64_t numerator = 0;
      if (!base::StringToInt64(fraction, &numerator)) {
        return std::nullopt;
      }
      int64_t denominator = 1;
      for (size_t i = 0; i < fraction.size(); ++i) {
        denominator *= 10;
      }
      const int64_t scaled = numerator * multiplier;
      if (scaled % denominator != 0) {
        return std::nullopt;
      }
      remainder = scaled / denominator;
    }
  }
  const int64_t milliseconds = whole * multiplier + remainder;
  return milliseconds <= maximum ? std::optional(milliseconds) : std::nullopt;
}
std::optional<ExplorerDelay> Delay(const ExplorerValue& value) {
  if (value.kind == Kind::kIdentifier) {
    if (value.text == "default") {
      return ExplorerDelay{};
    }
    if (value.text == "never") {
      return ExplorerDelay{ExplorerDelay::Mode::kNever};
    }
  }
  if (auto milliseconds = Duration(value, kExplorerMaximumDelayMs)) {
    return ExplorerDelay{ExplorerDelay::Mode::kAfter, *milliseconds};
  }
  return std::nullopt;
}
}  // namespace

std::optional<int64_t> ExplorerDelay::Resolve(int64_t default_ms) const {
  if (mode == Mode::kNever) {
    return std::nullopt;
  }
  return mode == Mode::kDefault ? default_ms : milliseconds;
}
ExplorerPolicySettings::ExplorerPolicySettings() {
  actions[static_cast<size_t>(ExplorerAction::kContextMenu)].mode =
      ExplorerMode::kBlock;
}
const ExplorerActionPolicy& ExplorerPolicySettings::operator[](
    ExplorerAction action) const {
  return actions[static_cast<size_t>(action)];
}
void ExplorerPolicySettings::Apply(const ExplorerPolicyOverrides& overrides) {
  for (const auto& [action, entry] : overrides.actions) {
    actions[static_cast<size_t>(action)] = entry;
  }
  if (overrides.freeze) {
    freeze = *overrides.freeze;
  }
  if (overrides.discard) {
    discard = *overrides.discard;
  }
}
std::optional<ExplorerAction> ParseExplorerAction(std::string_view name) {
  constexpr std::array<std::string_view, 5> names = {
      "navigation", "dialog", "external-protocol", "new-window",
      "context-menu"};
  for (size_t i = 0; i < names.size(); ++i) {
    if (names[i] == name) {
      return static_cast<ExplorerAction>(i);
    }
  }
  return std::nullopt;
}
std::optional<ExplorerMode> ParseExplorerMode(std::string_view mode) {
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
std::optional<ExplorerPolicyOverrides> ParseExplorerPolicy(
    std::string_view text,
    ExplorerDeclarationError* error) {
  auto declarations = ParseExplorerDeclarations(text, error);
  if (!declarations) {
    return std::nullopt;
  }
  ExplorerPolicyOverrides result;
  for (const auto& declaration : *declarations) {
    auto fail = [&](std::string_view message)
        -> std::optional<ExplorerPolicyOverrides> {
      if (error) {
        *error = {declaration.offset, std::string(message)};
      }
      return std::nullopt;
    };
    if (declaration.name == "freeze" || declaration.name == "discard") {
      if (declaration.values.size() != 1) {
        return fail("Expected never, default or one duration");
      }
      auto delay = Delay(declaration.values.front());
      if (!delay) {
        return fail("Expected never, default or a duration from 0ms to 24h");
      }
      (declaration.name == "freeze" ? result.freeze : result.discard) = *delay;
      continue;
    }
    const auto action = ParseExplorerAction(declaration.name);
    if (!action) {
      return fail("Unknown policy property");
    }
    const auto& values = declaration.values;
    const auto mode = values.front().kind == Kind::kIdentifier
                          ? ParseExplorerMode(values.front().text)
                          : std::nullopt;
    if (!mode || values.size() > 2 ||
        (*action == ExplorerAction::kContextMenu &&
         *mode == ExplorerMode::kDelegate)) {
      return fail("Invalid action policy mode");
    }
    ExplorerActionPolicy entry{.mode = *mode};
    if (values.size() == 2) {
      const auto& function = values[1];
      if (*mode != ExplorerMode::kDelegate ||
          *action == ExplorerAction::kNavigation ||
          function.kind != Kind::kFunction || function.text != "timeout" ||
          function.arguments.size() != 1) {
        return fail("Expected timeout(duration) for a delegated controller");
      }
      const auto timeout = Duration(function.arguments.front(), 300000);
      if (!timeout || *timeout == 0) {
        return fail("Controller timeout must be positive and at most 5m");
      }
      entry.timeout_ms = static_cast<int>(*timeout);
    }
    result.actions.insert_or_assign(*action, entry);
  }
  return result;
}
}  // namespace tuffite
