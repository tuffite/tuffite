// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_WINDOW_WINDOW_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_WINDOW_WINDOW_SERVICE_FACTORY_H_
#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/window/window_service.h"
namespace content {
class BrowserContext;
}
namespace tuffite::window {
class WindowServiceFactory final
    : public ApplicationKeyedServiceFactory<WindowService> {
 public:
  static WindowServiceFactory* Get();
  static WindowService* GetForApplication(ApplicationContext*,
                                          bool create = true);
  // Compatibility lookup: resolves the owning application, never changes scope.
  static WindowService* GetForBrowserContext(content::BrowserContext*,
                                             bool create = true);

 private:
  friend class base::NoDestructor<WindowServiceFactory>;
  WindowServiceFactory();
  std::unique_ptr<WindowService> BuildServiceInstanceForApplication(
      ApplicationContext&) const override;
};
}  // namespace tuffite::window
#endif
