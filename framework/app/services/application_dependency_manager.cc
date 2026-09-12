// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/application_dependency_manager.h"

#include "base/check.h"
#include "tuffite/framework/app/services/application_dependency.h"

namespace tuffite {

ApplicationDependencyManager* ApplicationDependencyManager::Get() {
  static base::NoDestructor<ApplicationDependencyManager> instance;
  return instance.get();
}

ApplicationDependencyManager::ApplicationDependencyManager() = default;
ApplicationDependencyManager::~ApplicationDependencyManager() = default;

KeyedService* ApplicationDependencyManager::GetForApplication(
    ApplicationContext* context,
    const void* service_type,
    bool create) {
  const auto factory = service_factories_.find(service_type);
  CHECK(factory != service_factories_.end())
      << "Application service factory was not registered";
  return factory->second->GetForApplication(context, create);
}

void ApplicationDependencyManager::RegisterServiceFactory(
    const void* service_type,
    ApplicationDependency* factory) {
  CHECK(factory);
  CHECK(service_factories_.emplace(service_type, factory).second)
      << "Application service type was registered more than once";
}

void ApplicationDependencyManager::CreateServices(ApplicationContext* context) {
  MarkContextLive(context);
  CreateContextServices(context, false);
}

void ApplicationDependencyManager::CreateServicesForTest(
    ApplicationContext* context) {
  MarkContextLive(context);
  CreateContextServices(context, true);
}

void ApplicationDependencyManager::DestroyServices(
    ApplicationContext* context) {
  DestroyContextServices(context);
}

}  // namespace tuffite
