// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_CONFIGURATION_DATA_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_CONFIGURATION_DATA_H_
#include <cstdint>
#include <map>
#include <string>
#include <utility>
#include <vector>

#include "tuffite/framework/app/services/asset/asset_configuration.h"
#include "tuffite/framework/app/services/manifest/app_config.h"

namespace tuffite::app {
// A method grant is compiled from the IDL registry; scopes retain each
// entry's boundary rather than combining fields from unrelated grants.
struct ShellApiGrant {
  std::string method;
  std::string scope_kind;
  bool has_allow = false;
  std::vector<std::string> allow;
  std::vector<std::string> deny;
};

struct OriginCapability {
  std::string identifier;
  std::vector<std::string> origins;
  bool devtools_allowed = true;
  bool create_explorer = false;
  bool execute_javascript = false;
  std::vector<ShellApiGrant> shell_api;
};

// Native configuration data. Only the loader and assembler obtain mutable
// ownership.
struct ManifestData {
  base::FilePath package_root_;
  std::string application_id_;
  std::string product_name_;
  asset::AssetConfiguration assets_{
      .content_security_policy =
          "default-src 'self'; script-src 'self'; style-src 'self' "
          "'unsafe-inline'; font-src 'self' data:; img-src 'self' data: blob: "
          "https: http:; frame-src 'self' https:; object-src 'none'; "
          "base-uri 'none'"};
  std::string app_host_;
  std::string serialized_;
  GURL entry_url_;
  std::vector<app::AppWindow> windows_;
  std::vector<std::string> shell_api_origin_patterns_;
  std::vector<std::string> explorer_origin_patterns_;
  std::vector<OriginCapability> origin_capabilities_;
  std::vector<std::pair<std::string, uint64_t>> shell_api_command_ids_;
  size_t network_max_response_bytes_ = 1024 * 1024;
  int network_timeout_seconds_ = 30;
  bool network_disable_http2_ = false;
  std::map<std::string, std::vector<uint8_t>> utility_configurations_;
};

}  // namespace tuffite::app
#endif
