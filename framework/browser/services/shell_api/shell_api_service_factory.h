// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_FACTORY_H_

#include <map>
#include <string>
#include <vector>

#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "base/no_destructor.h"
#include "components/keyed_service/content/browser_context_keyed_service_factory.h"

namespace tuffite {
class ApplicationContext;
class ShellApiService;
class ShellApiServiceFactory final : public BrowserContextKeyedServiceFactory {
 public:
  static ShellApiServiceFactory* Get();
  static ShellApiService* GetForBrowserContext(
      content::BrowserContext* context);
  static void BroadcastEvent(ApplicationContext& application,
                             std::string name,
                             std::vector<uint8_t> bytes);

 private:
  friend class ShellApiServiceFactoryTestPeer;
  friend class base::NoDestructor<ShellApiServiceFactory>;
  ShellApiServiceFactory();
  ~ShellApiServiceFactory() override;
  std::unique_ptr<KeyedService> BuildServiceInstanceForBrowserContext(
      content::BrowserContext* context) const override;
  struct RegisteredService {
    raw_ptr<ApplicationContext> application;
    base::WeakPtr<ShellApiService> service;
  };
  mutable std::map<content::BrowserContext*, RegisteredService> services_;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_FACTORY_H_
