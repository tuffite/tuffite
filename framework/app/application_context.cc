// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/application_context.h"

#include "tuffite/framework/app/services/application_dependency_manager.h"
#include "tuffite/framework/app/services/asset/asset_service_factory.h"
#include "tuffite/framework/app/services/policy/application_policy_service_factory.h"
#include "tuffite/framework/app/services/remoting/remoting_service_factory.h"
#include "tuffite/framework/app/services/utility/utility_service_factory.h"

namespace tuffite {
ApplicationContext::ApplicationContext(const AppManifest& manifest,
                                       AbiHostBridge& bridge)
    : manifest_(manifest), bridge_(bridge) {
  ApplicationPolicyServiceFactory::Get();
  AssetServiceFactory::Get();
  remoting::RemotingServiceFactory::Get();
  UtilityServiceFactory::Get();

  ApplicationDependencyManager::Get()->DisallowKeyedServiceFactoryRegistration(
      "ApplicationContext");
}
ApplicationContext::~ApplicationContext() = default;

KeyedService* ApplicationContext::GetServiceByType(const void* service_type,
                                                   bool create) {
  return ApplicationDependencyManager::Get()->GetForApplication(
      this, service_type, create);
}
}  // namespace tuffite
