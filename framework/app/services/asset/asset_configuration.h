// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_CONFIGURATION_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_CONFIGURATION_H_
#include <optional>
#include <string>
#include <vector>

#include "base/files/file_path.h"
#include "url/gurl.h"
namespace tuffite::asset {
struct AssetRoute {
  std::vector<std::string> origins;
  std::string path;
  std::string source;
  bool directory = false;
};

// Copyable resource-only snapshot, safe to retain after application shutdown.
struct AssetConfiguration {
  std::string scheme;
  base::FilePath root;
  std::vector<AssetRoute> routes;
  std::string content_security_policy;
  bool IsAppUrl(const GURL&) const;
  std::string ResolveAssetRoute(const GURL&) const;
  std::optional<base::FilePath> ResolveAssetFile(const GURL&) const;
};
}  // namespace tuffite::asset
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_CONFIGURATION_H_
