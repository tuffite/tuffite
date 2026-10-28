// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_PROFILE_PROFILE_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_PROFILE_PROFILE_SERVICE_FACTORY_H_
#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/profile/profile_manager.h"
namespace tuffite {
class ProfileServiceFactory final
    : public ApplicationKeyedServiceFactory<ProfileManager> {
 public:
  static ProfileServiceFactory* Get();

 private:
  friend class base::NoDestructor<ProfileServiceFactory>;
  ProfileServiceFactory();
  std::unique_ptr<ProfileManager> BuildServiceInstanceForApplication(
      ApplicationContext&) const override;
};
}  // namespace tuffite
#endif
