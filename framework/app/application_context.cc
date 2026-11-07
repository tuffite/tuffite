// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/application_context.h"

#include "tuffite/framework/app/services/application_dependency_manager.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/manifest/manifest_service_factory.h"
#include "tuffite/framework/app/services/profile/profile_service_factory.h"
#include "tuffite/framework/app/services/remoting/remoting_service_factory.h"
#include "tuffite/framework/app/services/shortcut/shortcut_service_factory.h"
#include "tuffite/framework/app/services/utility/utility_service_factory.h"
#include "tuffite/framework/app/services/window/window_service_factory.h"

namespace tuffite {
ApplicationContext::ApplicationContext(
    std::unique_ptr<ManifestService> manifest)
    : pending_manifest_(std::move(manifest)) {
  ManifestServiceFactory::Get();
  remoting::RemotingServiceFactory::Get();
  UtilityServiceFactory::Get();
  window::WindowServiceFactory::Get();
  ProfileServiceFactory::Get();
  ShortcutServiceFactory::Get();

  ApplicationDependencyManager::Get()->DisallowKeyedServiceFactoryRegistration(
      "ApplicationContext");
  ApplicationDependencyManager::Get()->CreateServices(this);
}

ApplicationContext::~ApplicationContext() {
  weak_factory_.InvalidateWeakPtrs();
  ApplicationDependencyManager::Get()->DestroyServices(this);
}

KeyedService* ApplicationContext::GetServiceByType(const void* service_type,
                                                   bool create) {
  return ApplicationDependencyManager::Get()->GetForApplication(
      this, service_type, create);
}

}  // namespace tuffite
