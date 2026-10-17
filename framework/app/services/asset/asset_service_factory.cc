// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/asset/asset_service_factory.h"

#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/asset/asset_service_impl.h"
#include "tuffite/framework/app/services/manifest/manifest_service_factory.h"

namespace tuffite {
AssetServiceFactory* AssetServiceFactory::Get() {
  static base::NoDestructor<AssetServiceFactory> instance;
  return instance.get();
}
AssetServiceFactory::AssetServiceFactory()
    : ApplicationKeyedServiceFactory<AssetService>("TuffiteAssets") {
  DependsOn(ManifestServiceFactory::Get());
}
std::unique_ptr<AssetService>
AssetServiceFactory::BuildServiceInstanceForApplication(
    ApplicationContext& context) const {
  return std::make_unique<AssetServiceImpl>(
      *context.GetService<ManifestService>());
}
}  // namespace tuffite
