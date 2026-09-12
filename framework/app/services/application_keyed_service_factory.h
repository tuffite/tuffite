// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_KEYED_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_KEYED_SERVICE_FACTORY_H_

#include <memory>
#include <type_traits>

#include "base/functional/bind.h"
#include "base/functional/callback.h"
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/application_dependency.h"

namespace tuffite {

template <typename Service>
class ApplicationKeyedServiceFactory : public ApplicationDependency {
  static_assert(std::is_base_of_v<KeyedService, Service>);

 public:
  using TestingFactory =
      base::OnceCallback<std::unique_ptr<Service>(ApplicationContext&)>;

  void SetTestingFactoryForApplication(ApplicationContext* context,
                                       TestingFactory factory) {
    KeyedServiceFactory::SetTestingFactory(
        context, base::BindOnce(
                     [](TestingFactory factory,
                        void* context) -> std::unique_ptr<KeyedService> {
                       return std::move(factory).Run(
                           *static_cast<ApplicationContext*>(context));
                     },
                     std::move(factory)));
  }

 protected:
  explicit ApplicationKeyedServiceFactory(const char* name)
      : ApplicationDependency(name,
                              internal::ApplicationServiceTypeKey<Service>()) {}

  virtual std::unique_ptr<Service> BuildServiceInstanceForApplication(
      ApplicationContext&) const = 0;

 private:
  void* GetContextToUse(void* context) const final {
    AssertContextWasntDestroyed(context);
    return context;
  }

  std::unique_ptr<KeyedService> BuildServiceInstanceFor(
      void* context) const final {
    return BuildServiceInstanceForApplication(
        *static_cast<ApplicationContext*>(context));
  }
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_KEYED_SERVICE_FACTORY_H_
