// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/devtools/devtools_service_factory.h"

#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/devtools/devtools_service_impl.h"

namespace tuffite {
DevToolsServiceFactory* DevToolsServiceFactory::Get() {
  static base::NoDestructor<DevToolsServiceFactory> instance;
  return instance.get();
}
DevToolsServiceFactory::DevToolsServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteDevTools",
          BrowserContextDependencyManager::GetInstance()) {}
DevToolsService* DevToolsServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<DevToolsService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
DevToolsServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<DevToolsServiceImpl>(*context);
}
}  // namespace tuffite
