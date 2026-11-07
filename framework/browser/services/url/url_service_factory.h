// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "components/keyed_service/content/browser_context_keyed_service_factory.h"
#include "tuffite/framework/browser/services/url/url_service.h"

namespace tuffite {
class URLServiceFactory final : public BrowserContextKeyedServiceFactory {
 public:
  static URLServiceFactory* Get();
  static URLService* GetForBrowserContext(content::BrowserContext* context,
                                          bool create = true);

 private:
  friend class base::NoDestructor<URLServiceFactory>;
  URLServiceFactory();
  std::unique_ptr<KeyedService> BuildServiceInstanceForBrowserContext(
      content::BrowserContext* context) const override;
};
}  // namespace tuffite
#endif
