// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/policy/application_policy_service.h"

namespace tuffite {
class ApplicationPolicyService;
class ApplicationPolicyServiceFactory final
    : public ApplicationKeyedServiceFactory<ApplicationPolicyService> {
 public:
  static ApplicationPolicyServiceFactory* Get();

 private:
  friend class base::NoDestructor<ApplicationPolicyServiceFactory>;
  ApplicationPolicyServiceFactory();
  std::unique_ptr<ApplicationPolicyService> BuildServiceInstanceForApplication(
      ApplicationContext& context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_POLICY_APPLICATION_POLICY_SERVICE_FACTORY_H_
