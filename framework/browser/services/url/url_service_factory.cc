// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/url/url_service_factory.h"

#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/isolated/isolated_service_factory.h"
#include "tuffite/framework/browser/services/url/url_service_impl.h"

namespace tuffite {
URLServiceFactory* URLServiceFactory::Get() {
  static base::NoDestructor<URLServiceFactory> instance;
  return instance.get();
}
URLServiceFactory::URLServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteURL",
          BrowserContextDependencyManager::GetInstance()) {
  DependsOn(IsolatedServiceFactory::Get());
}
URLService* URLServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<URLService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
URLServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<URLServiceImpl>(BrowserContext::GetFrom(*context));
}
}  // namespace tuffite
