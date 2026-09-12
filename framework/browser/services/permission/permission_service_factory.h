// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_PERMISSION_PERMISSION_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_PERMISSION_PERMISSION_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "components/keyed_service/content/browser_context_keyed_service_factory.h"

namespace tuffite {
class PermissionService;
class PermissionServiceFactory final
    : public BrowserContextKeyedServiceFactory {
 public:
  static PermissionServiceFactory* Get();
  static PermissionService* GetForBrowserContext(
      content::BrowserContext* context,
      bool create = true);

 private:
  friend class base::NoDestructor<PermissionServiceFactory>;
  PermissionServiceFactory();
  std::unique_ptr<KeyedService> BuildServiceInstanceForBrowserContext(
      content::BrowserContext* context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_PERMISSION_PERMISSION_SERVICE_FACTORY_H_
