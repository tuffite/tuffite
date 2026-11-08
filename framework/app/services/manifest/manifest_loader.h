// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_LOADER_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_LOADER_H_

#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "base/files/file_path.h"
#include "tuffite/framework/app/services/manifest/effective_manifest.h"
#include "url/gurl.h"

namespace tuffite {

struct ManifestEnvironment {
  // The entry origin's grants and denials follow this authenticated dev alias.
  std::optional<GURL> development_url;
};
struct ManifestOptions {
  struct Security {
    struct DevTools {
      std::vector<std::string> denied_origins;
    } devtools;
  } security;
};

class ManifestLoader {
 public:
  static std::optional<EffectiveManifest> Load(
      const base::FilePath& package_root,
      std::string* error,
      std::string_view configuration = {},
      const ManifestEnvironment& environment = {},
      const ManifestOptions& options = {});
  // Chromium needs these minimal bootstrap reads before custom schemes exist.
  static std::optional<std::vector<std::string>> ReadApplicationSchemes(
      const base::FilePath& package_root,
      std::string* error,
      std::string_view configuration = {});
  static bool ReadNetworkDisableHttp2(const base::FilePath& package_root,
                                      std::string_view configuration = {});
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_LOADER_H_
