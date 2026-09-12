// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_NETWORK_CONTEXT_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_NETWORK_CONTEXT_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "components/keyed_service/content/browser_context_keyed_service_factory.h"

namespace tuffite {
class NetworkContextService;
class NetworkContextServiceFactory final
    : public BrowserContextKeyedServiceFactory {
 public:
  static NetworkContextServiceFactory* Get();
  static NetworkContextService* GetForBrowserContext(
      content::BrowserContext* context,
      bool create = true);

 private:
  friend class base::NoDestructor<NetworkContextServiceFactory>;
  NetworkContextServiceFactory();
  // Read platform key preferences during profile initialization, before the
  // main loop disallows synchronous file access.
  bool ServiceIsCreatedWithBrowserContext() const override;
  std::unique_ptr<KeyedService> BuildServiceInstanceForBrowserContext(
      content::BrowserContext* context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_NETWORK_CONTEXT_SERVICE_FACTORY_H_
