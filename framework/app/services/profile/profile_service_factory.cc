// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/services/profile/profile_service_factory.h"

#include "tuffite/framework/app/services/manifest/manifest_service_factory.h"
#include "tuffite/framework/app/services/window/window_service_factory.h"
namespace tuffite {
ProfileServiceFactory* ProfileServiceFactory::Get() {
  static base::NoDestructor<ProfileServiceFactory> instance;
  return instance.get();
}
ProfileServiceFactory::ProfileServiceFactory()
    : ApplicationKeyedServiceFactory<ProfileManager>("TuffiteProfiles") {
  DependsOn(ManifestServiceFactory::Get());
  DependsOn(window::WindowServiceFactory::Get());
}
std::unique_ptr<ProfileManager>
ProfileServiceFactory::BuildServiceInstanceForApplication(
    ApplicationContext& app) const {
  return std::make_unique<ProfileManager>(app);
}
}  // namespace tuffite
