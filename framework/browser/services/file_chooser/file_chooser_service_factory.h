// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_FILE_CHOOSER_FILE_CHOOSER_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_FILE_CHOOSER_FILE_CHOOSER_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "components/keyed_service/content/browser_context_keyed_service_factory.h"

namespace tuffite {
class FileChooserService;
class FileChooserServiceFactory final
    : public BrowserContextKeyedServiceFactory {
 public:
  static FileChooserServiceFactory* Get();
  static FileChooserService* GetForBrowserContext(
      content::BrowserContext* context);

 private:
  friend class base::NoDestructor<FileChooserServiceFactory>;
  FileChooserServiceFactory();
  std::unique_ptr<KeyedService> BuildServiceInstanceForBrowserContext(
      content::BrowserContext* context) const override;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_FILE_CHOOSER_FILE_CHOOSER_SERVICE_FACTORY_H_
