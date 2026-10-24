// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_IMPL_H_
#include <memory>
#include <utility>

#include "tuffite/framework/app/services/manifest/manifest_configuration.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"

namespace tuffite::app {
struct OriginCapability;
}

namespace tuffite {
class ManifestServiceImpl final : public ManifestService {
 public:
  explicit ManifestServiceImpl(app::EffectiveConfiguration configuration);
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
  bool IsDevToolsAllowed(const url::Origin& origin) const override;
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
  using OriginCapability = app::OriginCapability;
  const app::ManifestData& data() const { return configuration_.data(); }
  OriginCapability GrantsFor(const url::Origin& origin) const;
  bool IsFileSystemPathAllowed(const url::Origin&,
                               std::string_view base,
                               std::string_view relative,
                               bool write,
                               std::string_view method) const;
  const app::EffectiveConfiguration configuration_;
  mutable base::WeakPtrFactory<ManifestServiceImpl> weak_factory_{this};
};
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_IMPL_H_
