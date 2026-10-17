// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"

namespace tuffite {
class ManifestService;
class ManifestServiceFactory final
    : public ApplicationKeyedServiceFactory<ManifestService> {
 public:
  static ManifestServiceFactory* Get();
  // Parsing completes before the service is published in the application graph.
  static std::unique_ptr<ManifestService> Load(
      const base::FilePath& package_root,
      std::string* error,
      const std::optional<GURL>& development_url = std::nullopt);
  // Chromium needs these before URL schemes and the service graph exist.
  static std::optional<std::string> ReadApplicationScheme(
      const base::FilePath& package_root,
      std::string* error);
  static bool ReadNetworkDisableHttp2(const base::FilePath& package_root);

 private:
  friend class base::NoDestructor<ManifestServiceFactory>;
  ManifestServiceFactory();
  bool ServiceIsCreatedWithContext() const override;
  std::unique_ptr<ManifestService> BuildServiceInstanceForApplication(
      ApplicationContext& context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_FACTORY_H_
