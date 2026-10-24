// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_APP_CONFIG_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_APP_CONFIG_H_

#include <cstddef>
#include <optional>
#include <string>

#include "url/gurl.h"

namespace tuffite::app {

struct AppWindow {
  std::string label;
  GURL url;
  std::string title;
  std::optional<int> x;
  std::optional<int> y;
  int width = 0;
  int height = 0;
  int min_width = 1;
  int min_height = 1;
  int max_width = 16384;
  int max_height = 16384;
  bool focused = true;
  bool decorations = true;
};

struct AppInfo {
  std::string identifier;
  std::string product_name;
  GURL entry_url;
  std::string scheme;
};

struct NetworkLimits {
  size_t max_response_bytes;
  int timeout_seconds;
};

}  // namespace tuffite::app
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_APP_CONFIG_H_
