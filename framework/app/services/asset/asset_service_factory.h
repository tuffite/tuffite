// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/asset/asset_service.h"

namespace tuffite {
class AssetServiceFactory final
    : public ApplicationKeyedServiceFactory<AssetService> {
 public:
  static AssetServiceFactory* Get();

 private:
  friend class base::NoDestructor<AssetServiceFactory>;
  AssetServiceFactory();
  std::unique_ptr<AssetService> BuildServiceInstanceForApplication(
      ApplicationContext& context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_FACTORY_H_
