// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/download/download_service_factory.h"

#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "content/public/browser/browser_context.h"
#include "tuffite/framework/browser/services/download/download_service_impl.h"

namespace tuffite {
DownloadServiceFactory* DownloadServiceFactory::Get() {
  static base::NoDestructor<DownloadServiceFactory> instance;
  return instance.get();
}
DownloadServiceFactory::DownloadServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteDownload",
          BrowserContextDependencyManager::GetInstance()) {}
DownloadService* DownloadServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<DownloadService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
DownloadServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<DownloadServiceImpl>(context->GetDownloadManager());
}
}  // namespace tuffite
