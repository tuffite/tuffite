// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_DEPENDENCY_MANAGER_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_DEPENDENCY_MANAGER_H_

#include "base/containers/flat_map.h"
#include "base/memory/raw_ptr.h"
#include "base/no_destructor.h"
#include "components/keyed_service/core/dependency_manager.h"
#include "components/keyed_service/core/keyed_service.h"

namespace tuffite {

class ApplicationContext;
class ApplicationDependency;

class ApplicationDependencyManager final : public DependencyManager {
 public:
  static ApplicationDependencyManager* Get();

  void CreateServices(ApplicationContext*);
  void CreateServicesForTest(ApplicationContext*);
  void DestroyServices(ApplicationContext*);

 private:
  friend class ApplicationContext;
  friend class ApplicationDependency;
  friend class base::NoDestructor<ApplicationDependencyManager>;

  ApplicationDependencyManager();
  ~ApplicationDependencyManager() override;

  KeyedService* GetForApplication(ApplicationContext*,
                                  const void* service_type,
                                  bool create);
  void RegisterServiceFactory(const void* service_type, ApplicationDependency*);

  base::flat_map<const void*, raw_ptr<ApplicationDependency>>
      service_factories_;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_DEPENDENCY_MANAGER_H_
