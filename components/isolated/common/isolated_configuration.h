// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_ISOLATED_COMMON_ISOLATED_CONFIGURATION_H_
#define TUFFITE_COMPONENTS_ISOLATED_COMMON_ISOLATED_CONFIGURATION_H_
#include <map>
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
    if (!alpha(ch) && !(ch >= '0' && ch <= '9') && ch != '_' && ch != '-' &&
        ch != '.') {
      return false;
    }
  }
  return true;
}
inline bool IsValidResourcePath(std::string_view path) {
  if (path.empty() || path.size() > 2048 || path.front() == '/' ||
      path.find_first_of("\\:%?#") != std::string_view::npos ||
      path.find('\0') != std::string_view::npos) {
    return false;
  }
  while (!path.empty()) {
    auto end = path.find('/');
    auto part = path.substr(0, end);
    if (part.empty() || part == "." || part == "..") {
      return false;
    }
    if (end == std::string_view::npos) {
      break;
    }
    path.remove_prefix(end + 1);
    if (path.empty()) {
      return false;
    }
  }
  return true;
}
struct WorldDefinition {
  std::vector<std::string> origins;
  std::string source;
  std::string entry = "entry.js";
  std::map<std::string, std::string> resources;
  size_t ByteSize() const {
    size_t size = source.size();
    for (const auto& [path, bytes] : resources) {
      size += bytes.size();
    }
    return size;
  }
};
}  // namespace tuffite::isolated
#endif
