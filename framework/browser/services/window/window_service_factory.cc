// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/window/window_service_factory.h"

#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/window/window_service_impl.h"
namespace tuffite::window {
WindowServiceFactory* WindowServiceFactory::Get() {
  static base::NoDestructor<WindowServiceFactory> instance;
  return instance.get();
}
WindowServiceFactory::WindowServiceFactory()
    : ApplicationKeyedServiceFactory<WindowService>("TuffiteWindow") {}
WindowService* WindowServiceFactory::GetForApplication(ApplicationContext* app,
                                                       bool create) {
  return app ? app->GetService<WindowService>(create) : nullptr;
}
WindowService* WindowServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return context ? GetForApplication(
                       &BrowserContext::GetFrom(*context).application_context(),
                       create)
                 : nullptr;
}
std::unique_ptr<WindowService>
WindowServiceFactory::BuildServiceInstanceForApplication(
    ApplicationContext& app) const {
  return std::make_unique<WindowServiceImpl>(app);
}
}  // namespace tuffite::window
