// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_IMPL_H_
#include <memory>
#include <utility>

#include "tuffite/framework/app/services/manifest/manifest_service.h"

namespace tuffite {
class ManifestServiceImpl final : public ManifestService {
 public:
  static std::unique_ptr<ManifestServiceImpl> Load(
      const base::FilePath& package_root,
      std::string* error,
      const std::optional<GURL>& development_url);
  static std::optional<std::string> ReadApplicationScheme(
      const base::FilePath& package_root,
      std::string* error);
  static bool ReadNetworkDisableHttp2(const base::FilePath& package_root);
  ~ManifestServiceImpl() override;
  app::AppInfo app_info() const override;
  const std::vector<app::AppWindow>& startup_windows() const override;
  app::NetworkLimits network_limits() const override;
  asset::AssetConfiguration assets() const override;
  const std::string& SerializeRuntimeConfiguration() const override;
  std::map<std::string, std::vector<uint8_t>> UtilityConfigurations()
      const override;
  std::string SerializeShellApiOrigins() const override;
  std::string SerializeExplorerOrigins() const override;
  std::optional<base::FilePath> ResolveBaseDirectory(
      std::string_view base) const override;
  std::optional<base::FilePath> ResolveFileSystemPath(
      const url::Origin& caller,
      std::string_view base,
      std::string_view relative,
      bool write,
      std::string_view method) const override;
  std::vector<std::string> ExecuteJavaScriptOrigins(
      const url::Origin& origin) const override;
  std::string ApplicationInterface(std::string_view channel) const override;
  bool IsApplicationInterface(std::string_view service) const override;
  bool IsApplicationCommandRequest(std::string_view channel,
                                   uint64_t command_id) const override;
  bool IsShellApiOriginAllowed(const url::Origin& origin) const override;
  bool IsAllShellApiAllowed(const url::Origin& origin) const override;
  bool IsExplorerOriginAllowed(const url::Origin& origin) const override;
  bool IsShellApiCommandAllowed(const url::Origin& origin,
                                std::string_view channel) const override;
  bool IsShellApiScopeAllowed(const url::Origin& origin,
                              std::string_view method,
                              std::string_view resource,
                              std::string_view kind,
                              bool write = false) const override;
  bool IsExplorerNavigationAllowed(const url::Origin& caller,
                                   const GURL& url) const override;
  bool IsWindowUrlAllowed(const url::Origin& caller,
                          const GURL& url) const override;
  bool IsNetworkUrlAllowed(const url::Origin& caller,
                           const GURL& url) const override;
  base::WeakPtr<const ManifestService> GetWeakPtr() const override;
  void Shutdown() override;

 private:
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
    bool create_explorer = false;
    bool execute_javascript = false;
    std::vector<ShellApiGrant> shell_api;
  };

  ManifestServiceImpl();
  bool ApplyDevelopmentUrl(const GURL& url, std::string* error);
  OriginCapability GrantsFor(const url::Origin& origin) const;
  bool IsFileSystemPathAllowed(const url::Origin&,
                               std::string_view base,
                               std::string_view relative,
                               bool write,
                               std::string_view method) const;
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
  mutable base::WeakPtrFactory<ManifestServiceImpl> weak_factory_{this};
};
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_IMPL_H_
