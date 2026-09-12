// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/application_dependency.h"

#include "tuffite/framework/app/services/application_dependency_manager.h"

namespace tuffite {
ApplicationDependency::ApplicationDependency(const char* name,
                                             const void* service_type)
    : KeyedServiceFactory(name,
                          ApplicationDependencyManager::Get(),
                          KeyedServiceBaseFactory::SIMPLE) {
  ApplicationDependencyManager::Get()->RegisterServiceFactory(service_type,
                                                              this);
}
ApplicationDependency::~ApplicationDependency() = default;

KeyedService* ApplicationDependency::GetForApplication(
    ApplicationContext* context,
    bool create) {
  return GetServiceForContext(context, create);
}
}  // namespace tuffite
