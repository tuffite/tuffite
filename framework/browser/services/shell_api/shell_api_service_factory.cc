// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/shell_api_service_factory.h"

#include <algorithm>

#include "base/check.h"
#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "tuffite/framework/app/abi_host_bridge.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service_impl.h"
#include "tuffite/framework/browser/services/window/window_service_factory.h"

namespace tuffite {
ShellApiServiceFactory::~ShellApiServiceFactory() = default;

ShellApiServiceFactory* ShellApiServiceFactory::Get() {
  static base::NoDestructor<ShellApiServiceFactory> instance;
  return instance.get();
}
ShellApiServiceFactory::ShellApiServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteShellApi",
          BrowserContextDependencyManager::GetInstance()) {
  DependsOn(window::WindowServiceFactory::Get());
}
ShellApiService* ShellApiServiceFactory::GetForBrowserContext(
    content::BrowserContext* context) {
  return static_cast<ShellApiService*>(
      Get()->GetServiceForBrowserContext(context, true));
}
std::unique_ptr<KeyedService>
ShellApiServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  auto& application = BrowserContext::GetFrom(*context).application_context();
  auto* manifest = application.GetService<ManifestService>();
  CHECK(manifest);
  auto* utility = application.GetService<UtilityService>();
  CHECK(utility);
  auto service = std::make_unique<ShellApiServiceImpl>(
      *context, application.bridge(), *manifest, *utility);
  services_.insert_or_assign(
      context, RegisteredService{&application, service->GetWeakPtr()});
  return service;
}
void ShellApiServiceFactory::BroadcastEvent(ApplicationContext& application,
                                            std::string name,
                                            std::vector<uint8_t> bytes) {
  auto* factory = Get();
  std::erase_if(factory->services_,
                [](const auto& entry) { return !entry.second.service; });
  const auto services = factory->services_;
  for (const auto& entry : services) {
    const auto& registration = entry.second;
    if (registration.application == &application && registration.service) {
      registration.service->Broadcast(name, bytes);
    }
  }
}
}  // namespace tuffite
