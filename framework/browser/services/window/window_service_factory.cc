// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/window_service_factory.h"

#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "tuffite/framework/browser/services/file_chooser/file_chooser_service_factory.h"
#include "tuffite/framework/browser/services/javascript_dialog/javascript_dialog_service_factory.h"
#include "tuffite/framework/browser/services/window/window_service_impl.h"

namespace tuffite::window {
WindowServiceFactory* WindowServiceFactory::Get() {
  static base::NoDestructor<WindowServiceFactory> instance;
  return instance.get();
}
WindowServiceFactory::WindowServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteWindow",
          BrowserContextDependencyManager::GetInstance()) {
  DependsOn(JavaScriptDialogServiceFactory::Get());
  DependsOn(FileChooserServiceFactory::Get());
}
WindowService* WindowServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<WindowService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
WindowServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<WindowServiceImpl>(*context);
}
}  // namespace tuffite::window
