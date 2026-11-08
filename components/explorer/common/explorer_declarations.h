// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_EXPLORER_COMMON_EXPLORER_DECLARATIONS_H_
#define TUFFITE_COMPONENTS_EXPLORER_COMMON_EXPLORER_DECLARATIONS_H_

#include <optional>
#include <string>
#include <string_view>
#include <vector>

namespace tuffite {
struct ExplorerDeclarationError {
  size_t offset = 0;
  std::string message;
};
struct ExplorerValue {
  enum class Kind {
    kIdentifier,
    kNumber,
    kDimension,
    kString,
    kFunction,
    kComma
  };
  Kind kind;
  std::string text;
  size_t offset;
  std::vector<ExplorerValue> arguments;
};
struct ExplorerDeclaration {
  std::string name;
  size_t offset;
  std::vector<ExplorerValue> values;
};
// Syntax is independent of property names and their semantic value types.
// Offsets are UTF-8 byte offsets into the supplied declaration text.
std::optional<std::vector<ExplorerDeclaration>> ParseExplorerDeclarations(
    std::string_view text,
    ExplorerDeclarationError* error = nullptr);
}  // namespace tuffite
#endif
