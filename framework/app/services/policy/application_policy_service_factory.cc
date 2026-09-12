// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/policy/application_policy_service_factory.h"

#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/policy/application_policy_service_impl.h"

namespace tuffite {
ApplicationPolicyServiceFactory* ApplicationPolicyServiceFactory::Get() {
  static base::NoDestructor<ApplicationPolicyServiceFactory> instance;
  return instance.get();
}
ApplicationPolicyServiceFactory::ApplicationPolicyServiceFactory()
    : ApplicationKeyedServiceFactory<ApplicationPolicyService>(
          "TuffitePolicy") {}
std::unique_ptr<ApplicationPolicyService>
ApplicationPolicyServiceFactory::BuildServiceInstanceForApplication(
    ApplicationContext& context) const {
  return std::make_unique<ApplicationPolicyServiceImpl>(context.manifest());
}
}  // namespace tuffite
