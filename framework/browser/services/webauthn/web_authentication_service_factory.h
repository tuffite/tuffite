// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "components/keyed_service/content/browser_context_keyed_service_factory.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service.h"

namespace tuffite {
class WebAuthenticationServiceFactory final
    : public BrowserContextKeyedServiceFactory {
 public:
  static WebAuthenticationServiceFactory* Get();
  static WebAuthenticationService* GetForBrowserContext(
      content::BrowserContext* context);
  static content::WebAuthenticationDelegate* GetDelegate();

 private:
  friend class base::NoDestructor<WebAuthenticationServiceFactory>;
  WebAuthenticationServiceFactory();
  std::unique_ptr<KeyedService> BuildServiceInstanceForBrowserContext(
      content::BrowserContext* context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_FACTORY_H_
