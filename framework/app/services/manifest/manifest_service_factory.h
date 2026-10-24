// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/manifest/manifest_configuration.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"

namespace tuffite {
class ManifestService;
class ManifestServiceFactory final
    : public ApplicationKeyedServiceFactory<ManifestService> {
 public:
  static ManifestServiceFactory* Get();
  static std::unique_ptr<ManifestService> Create(
      app::EffectiveConfiguration configuration);

 private:
  friend class base::NoDestructor<ManifestServiceFactory>;
  ManifestServiceFactory();
  bool ServiceIsCreatedWithContext() const override;
  std::unique_ptr<ManifestService> BuildServiceInstanceForApplication(
      ApplicationContext& context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_MANIFEST_SERVICE_FACTORY_H_
