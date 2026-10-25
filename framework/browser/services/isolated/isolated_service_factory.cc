// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/isolated/isolated_service_factory.h"

#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/isolated/isolated_service_impl.h"

namespace tuffite {
IsolatedServiceFactory* IsolatedServiceFactory::Get() {
  static base::NoDestructor<IsolatedServiceFactory> instance;
  return instance.get();
}
IsolatedServiceFactory::IsolatedServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteIsolated",
          BrowserContextDependencyManager::GetInstance()) {}
IsolatedService* IsolatedServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<IsolatedService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
IsolatedServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<IsolatedServiceImpl>(
      BrowserContext::GetFrom(*context).manifest_service());
}
}  // namespace tuffite
