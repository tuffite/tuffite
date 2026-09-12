// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_CLIENT_HINTS_CLIENT_HINTS_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_CLIENT_HINTS_CLIENT_HINTS_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "components/keyed_service/content/browser_context_keyed_service_factory.h"

namespace tuffite {
class ClientHintsService;
class ClientHintsServiceFactory final
    : public BrowserContextKeyedServiceFactory {
 public:
  static ClientHintsServiceFactory* Get();
  static ClientHintsService* GetForBrowserContext(
      content::BrowserContext* context,
      bool create = true);

 private:
  friend class base::NoDestructor<ClientHintsServiceFactory>;
  ClientHintsServiceFactory();
  std::unique_ptr<KeyedService> BuildServiceInstanceForBrowserContext(
      content::BrowserContext* context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_CLIENT_HINTS_CLIENT_HINTS_SERVICE_FACTORY_H_
