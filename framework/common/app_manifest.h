// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_APP_MANIFEST_H_
#define TUFFITE_FRAMEWORK_COMMON_APP_MANIFEST_H_

#include <stddef.h>
#include <stdint.h>

#include <map>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "base/files/file_path.h"
#include "url/gurl.h"
#include "url/origin.h"

namespace tuffite {

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

struct AssetRoute {
  std::vector<std::string> origins;
  std::string path;
  std::string source;
  bool directory = false;
};

// A method grant is compiled from the IDL registry; scopes retain each entry's
// boundary rather than combining fields from unrelated grants.
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
  bool create_explorer = false;
  bool execute_javascript = false;
  std::vector<ShellApiGrant> shell_api;
};

// Immutable, validated configuration loaded from <package>/tuffite.json.
// Keeping this type in common lets startup, the browser client, and resource
// loading consume the same policy without application-specific constants.
class AppManifest {
 public:
  static std::optional<AppManifest> Load(const base::FilePath& package_root,
                                         std::string* error);
  static std::optional<std::string> ReadApplicationScheme(
      const base::FilePath& package_root,
      std::string* error);
  static bool ReadNetworkDisableHttp2(const base::FilePath& package_root);

  // Applies an explicit development-only main-window URL. Only loopback
  // HTTP URLs are accepted, and the exact origin is granted to the App
  // renderer without mutating the packaged manifest text.
  bool ApplyDevelopmentUrl(const GURL& url, std::string* error);

  AppManifest(const AppManifest&);
  AppManifest& operator=(const AppManifest&);
  AppManifest(AppManifest&&);
  AppManifest& operator=(AppManifest&&);
  ~AppManifest();

  const std::string& application_id() const { return application_id_; }
  const std::string& product_name() const { return product_name_; }
  const GURL& entry_url() const { return entry_url_; }
  const std::vector<AppWindow>& windows() const { return windows_; }
  const std::string& app_host() const { return app_host_; }
  const std::string& app_scheme() const { return app_scheme_; }
  const base::FilePath& package_root() const { return package_root_; }
  const base::FilePath& asset_root() const { return asset_root_; }
  const std::string& content_security_policy() const {
    return content_security_policy_;
  }
  std::string ResolveAssetRoute(const GURL& url) const;
  std::optional<base::FilePath> ResolveAssetFile(const GURL& url) const;
  OriginCapability GrantsFor(const url::Origin& origin) const;
  const std::string& shell_api_namespace() const {
    return shell_api_namespace_;
  }
  const std::string& serialized() const { return serialized_; }
  std::map<std::string, std::vector<uint8_t>> UtilityConfigurations() const;
  size_t network_max_response_bytes() const {
    return network_max_response_bytes_;
  }
  int network_timeout_seconds() const { return network_timeout_seconds_; }
  bool network_disable_http2() const { return network_disable_http2_; }
  bool IsAppUrl(const GURL& url) const;
  bool IsNetworkUrlAllowed(const url::Origin& caller, const GURL& url) const;
  bool IsShellApiOriginAllowed(const url::Origin& origin) const;
  bool IsExplorerOriginAllowed(const url::Origin& origin) const;
  bool IsExplorerNavigationAllowed(const url::Origin& caller,
                                   const GURL& url) const;
  bool IsAllShellApiAllowed(const url::Origin&) const;
  bool IsShellApiCommandAllowed(const url::Origin&, std::string_view) const;
  bool IsShellApiScopeAllowed(const url::Origin& origin,
                              std::string_view method,
                              std::string_view resource) const;
  bool IsApplicationCommandRequest(std::string_view channel,
                                   uint64_t command_id) const;
  std::vector<std::string> ExecuteJavaScriptOrigins(const url::Origin&) const;
  bool IsWindowUrlAllowed(const url::Origin& caller, const GURL& url) const;
  bool IsFileSystemPathAllowed(
      const url::Origin& caller,
      std::string_view base,
      std::string_view relative,
      bool write,
      std::string_view method = "tuffite.fs.readFile") const;
  std::optional<base::FilePath> ResolveBaseDirectory(
      std::string_view base) const;
  std::optional<base::FilePath> ResolveFileSystemPath(
      const url::Origin& caller,
      std::string_view base,
      std::string_view relative,
      bool write,
      std::string_view method = "tuffite.fs.readFile") const;
  static bool HasNoSymlinkComponents(const base::FilePath& root,
                                     const base::FilePath& path);
  std::string SerializeShellApiOrigins() const;
  std::string SerializeExplorerOrigins() const;

 private:
  AppManifest();

  base::FilePath package_root_;
  base::FilePath asset_root_;
  std::vector<AssetRoute> asset_routes_;
  std::string application_id_;
  std::string product_name_;
  std::string content_security_policy_ =
      "default-src 'self'; script-src 'self'; style-src 'self' "
      "'unsafe-inline'; font-src 'self' data:; img-src 'self' data: blob: "
      "https: http:; frame-src 'self' https:; object-src 'none'; "
      "base-uri 'none'";
  std::string app_scheme_;
  std::string app_host_;
  std::string shell_api_namespace_;
  std::string serialized_;
  GURL entry_url_;
  std::vector<AppWindow> windows_;
  std::vector<std::string> shell_api_origin_patterns_;
  std::vector<std::string> explorer_origin_patterns_;
  std::vector<OriginCapability> origin_capabilities_;
  std::vector<std::pair<std::string, uint64_t>> shell_api_command_ids_;
  size_t network_max_response_bytes_ = 1024 * 1024;
  int network_timeout_seconds_ = 30;
  bool network_disable_http2_ = false;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_COMMON_APP_MANIFEST_H_
