// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/utility/utility_service_factory.h"

#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/policy/application_policy_service_factory.h"
#include "tuffite/framework/app/services/utility/utility_process_launcher.h"
#include "tuffite/framework/app/services/utility/utility_service.h"

namespace tuffite {
UtilityServiceFactory* UtilityServiceFactory::Get() {
  static base::NoDestructor<UtilityServiceFactory> instance;
  return instance.get();
}
UtilityServiceFactory::UtilityServiceFactory()
    : ApplicationKeyedServiceFactory<UtilityService>("TuffiteUtility") {
  DependsOn(ApplicationPolicyServiceFactory::Get());
}
std::unique_ptr<UtilityService>
UtilityServiceFactory::BuildServiceInstanceForApplication(
    ApplicationContext&) const {
  return CreateUtilityService(CreateUtilityProcessLauncher());
}
}  // namespace tuffite
