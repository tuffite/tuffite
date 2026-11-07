// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/shortcut/shortcut_service_factory.h"

#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/shortcut/shortcut_service_impl.h"
#include "tuffite/framework/app/services/window/window_service_factory.h"
namespace tuffite {
ShortcutServiceFactory* ShortcutServiceFactory::Get() {
  static base::NoDestructor<ShortcutServiceFactory> instance;
  return instance.get();
}
ShortcutServiceFactory::ShortcutServiceFactory()
    : ApplicationKeyedServiceFactory<ShortcutService>("TuffiteShortcuts") {
  DependsOn(window::WindowServiceFactory::Get());
}
std::unique_ptr<ShortcutService>
ShortcutServiceFactory::BuildServiceInstanceForApplication(
    ApplicationContext& app) const {
  return std::make_unique<ShortcutServiceImpl>(
      *app.GetService<window::WindowService>());
}
}  // namespace tuffite
