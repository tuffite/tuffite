// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_CONFIGURATION_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_CONFIGURATION_H_
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "base/files/file_path.h"
#include "base/functional/callback.h"
#include "url/gurl.h"
namespace tuffite::asset {
// Optional in-memory resources supplied by a document-scoped owner.
struct MemoryAsset {
  std::string mime_type;
  std::string contents;
  std::string allow_origin;
};
using MemoryAssetResolver =
    base::RepeatingCallback<std::optional<MemoryAsset>(const GURL&)>;
struct AssetRoute {
  std::vector<std::string> origins;
  std::string path;
  std::string source;
  bool directory = false;
};

// Copyable resource-only snapshot, safe to retain after application shutdown.
struct AssetConfiguration {
  std::vector<std::string> schemes;
  bool HandlesScheme(std::string_view scheme) const;
  base::FilePath root;
  std::vector<AssetRoute> routes;
  std::string content_security_policy;
  std::vector<std::pair<std::vector<std::string>, std::string>> csp_rules;
  std::string ContentSecurityPolicy(const GURL& url,
                                    bool fallback = true) const;
  bool IsAppUrl(const GURL&) const;
  std::string ResolveAssetRoute(const GURL&) const;
  std::optional<base::FilePath> ResolveAssetFile(const GURL&) const;
};
}  // namespace tuffite::asset
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_CONFIGURATION_H_
