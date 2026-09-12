// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/utility/utility_service.h"

namespace tuffite {
class UtilityService;
class UtilityServiceFactory final
    : public ApplicationKeyedServiceFactory<UtilityService> {
 public:
  static UtilityServiceFactory* Get();

 private:
  friend class base::NoDestructor<UtilityServiceFactory>;
  UtilityServiceFactory();
  std::unique_ptr<UtilityService> BuildServiceInstanceForApplication(
      ApplicationContext& context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_SERVICE_FACTORY_H_
