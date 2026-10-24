// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_CONFIGURATION_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_CONFIGURATION_H_
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

#include "base/files/file_path.h"
#include "url/gurl.h"

namespace tuffite {
class ManifestServiceImpl;
}

namespace tuffite::app {
struct ManifestData;

// A validated package declaration, before any host or launch-time adjustments.
class ManifestDefinition {
 public:
  ManifestDefinition(ManifestDefinition&&) noexcept;
  ManifestDefinition& operator=(ManifestDefinition&&) noexcept;
  ~ManifestDefinition();

 private:
  friend class ManifestLoader;
  friend class ConfigurationAssembler;
  explicit ManifestDefinition(std::unique_ptr<ManifestData> data);
  std::unique_ptr<ManifestData> data_;
};

struct StartupEnvironment {
  // The entry origin's grants and denials follow this authenticated dev alias.
  std::optional<GURL> development_url;
};
struct BrowserOptions {
  struct Security {
    struct DevTools {
      std::vector<std::string> denied_origins;
    } devtools;
  } security;
};

// Can only be created by the assembler. Published services cannot mutate it.
class EffectiveConfiguration {
 public:
  EffectiveConfiguration(EffectiveConfiguration&&) noexcept;
  EffectiveConfiguration& operator=(EffectiveConfiguration&&) noexcept;
  ~EffectiveConfiguration();

 private:
  friend class ConfigurationAssembler;
  friend class ::tuffite::ManifestServiceImpl;
  explicit EffectiveConfiguration(std::unique_ptr<ManifestData> data);
  const ManifestData& data() const { return *data_; }
  std::unique_ptr<const ManifestData> data_;
};

class ManifestLoader {
 public:
  static std::optional<ManifestDefinition> Load(
      const base::FilePath& package_root,
      std::string* error);
  // Chromium needs these minimal bootstrap reads before custom schemes exist.
  static std::optional<std::string> ReadApplicationScheme(
      const base::FilePath& package_root,
      std::string* error);
  static bool ReadNetworkDisableHttp2(const base::FilePath& package_root);
};
class ConfigurationAssembler {
 public:
  static std::optional<EffectiveConfiguration> Build(
      ManifestDefinition definition,
      const StartupEnvironment& environment,
      const BrowserOptions& options,
      std::string* error);
};
}  // namespace tuffite::app
#endif
