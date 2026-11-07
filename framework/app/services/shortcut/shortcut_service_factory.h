// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_SHORTCUT_SHORTCUT_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_SHORTCUT_SHORTCUT_SERVICE_FACTORY_H_
#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/shortcut/shortcut_service.h"
namespace tuffite {
class ShortcutServiceFactory final
    : public ApplicationKeyedServiceFactory<ShortcutService> {
 public:
  static ShortcutServiceFactory* Get();

 private:
  friend class base::NoDestructor<ShortcutServiceFactory>;
  ShortcutServiceFactory();
  std::unique_ptr<ShortcutService> BuildServiceInstanceForApplication(
      ApplicationContext&) const override;
};
}  // namespace tuffite
#endif
