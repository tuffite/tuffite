// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_H_
#include <cstdint>
#include <map>
#include <string_view>

#include "base/files/file_path.h"
#include "base/memory/weak_ptr.h"
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/framework/app/services/asset/asset_configuration.h"
#include "tuffite/framework/app/services/manifest/app_config.h"
#include "url/origin.h"
namespace tuffite {
// Immutable application configuration and origin-based grants. Browser roles
// and frame state are deliberately evaluated by browser-side consumers.
class ManifestService : public KeyedService {
 public:
  virtual app::AppInfo app_info() const = 0;
  virtual const std::vector<app::AppWindow>& startup_windows() const = 0;
  virtual app::NetworkLimits network_limits() const = 0;
  virtual asset::AssetConfiguration assets() const = 0;
  virtual const std::string& SerializeRuntimeConfiguration() const = 0;
  virtual std::map<std::string, std::vector<uint8_t>> UtilityConfigurations()
      const = 0;
  virtual std::string SerializeShellApiOrigins() const = 0;
  virtual std::string SerializeExplorerOrigins() const = 0;
  virtual std::optional<base::FilePath> ResolveBaseDirectory(
      std::string_view base) const = 0;
  virtual std::optional<base::FilePath> ResolveFileSystemPath(
      const url::Origin& caller,
      std::string_view base,
      std::string_view relative,
      bool write,
      std::string_view method) const = 0;
  virtual std::vector<std::string> ExecuteJavaScriptOrigins(
      const url::Origin& origin) const = 0;
  virtual bool IsApplicationCommandRequest(std::string_view channel,
                                           uint64_t command_id) const = 0;
  virtual bool IsShellApiOriginAllowed(const url::Origin& origin) const = 0;
  virtual bool IsAllShellApiAllowed(const url::Origin& origin) const = 0;
  virtual bool IsExplorerOriginAllowed(const url::Origin& origin) const = 0;
  virtual bool IsShellApiCommandAllowed(const url::Origin& origin,
                                        std::string_view channel) const = 0;
  virtual bool IsShellApiScopeAllowed(const url::Origin& origin,
                                      std::string_view method,
                                      std::string_view resource) const = 0;
  virtual bool IsExplorerNavigationAllowed(const url::Origin& caller,
                                           const GURL& url) const = 0;
  virtual bool IsWindowUrlAllowed(const url::Origin& caller,
                                  const GURL& url) const = 0;
  virtual bool IsNetworkUrlAllowed(const url::Origin& caller,
                                   const GURL& url) const = 0;
  virtual base::WeakPtr<const ManifestService> GetWeakPtr() const = 0;
};
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_H_
