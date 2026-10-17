// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/context_menu/context_menu_service_factory.h"

#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service_impl.h"
#include "tuffite/framework/browser/services/devtools/devtools_service_factory.h"

namespace tuffite {
ContextMenuServiceFactory* ContextMenuServiceFactory::Get() {
  static base::NoDestructor<ContextMenuServiceFactory> instance;
  return instance.get();
}
ContextMenuServiceFactory::ContextMenuServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteContextMenu",
          BrowserContextDependencyManager::GetInstance()) {
  DependsOn(DevToolsServiceFactory::Get());
}
ContextMenuService* ContextMenuServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<ContextMenuService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
ContextMenuServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<ContextMenuServiceImpl>(
      BrowserContext::GetFrom(*context).manifest_service(),
      *DevToolsServiceFactory::GetForBrowserContext(context));
}
}  // namespace tuffite
