// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/explorer/explorer_service_factory.h"

#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/explorer/explorer_service_impl.h"
namespace tuffite {
ExplorerServiceFactory* ExplorerServiceFactory::Get() {
  static base::NoDestructor<ExplorerServiceFactory> instance;
  return instance.get();
}
ExplorerServiceFactory::ExplorerServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteExplorer",
          BrowserContextDependencyManager::GetInstance()) {}
bool ExplorerServiceFactory::ServiceIsCreatedWithBrowserContext() const {
  return true;
}
ExplorerService* ExplorerServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<ExplorerService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
ExplorerServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<ExplorerServiceImpl>(
      BrowserContext::GetFrom(*context).manifest_service());
}
}  // namespace tuffite
