// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_BROWSER_CONTEXT_H_
#define TUFFITE_FRAMEWORK_BROWSER_BROWSER_CONTEXT_H_

#include "base/files/file_path.h"
#include "base/memory/raw_ref.h"
#include "content/public/browser/browser_context.h"

namespace content {
class RenderFrameHost;
}

namespace tuffite {

class ExplorerRouter;
class AppManifest;
class ApplicationPolicyService;
class ApplicationContext;

class BrowserContext final : public content::BrowserContext {
 public:
  BrowserContext(base::FilePath path, ApplicationContext& application);

  // Live profile roots granted to the sandboxed network process.
  static std::vector<base::FilePath> GetNetworkStoragePaths();
  static BrowserContext& GetFrom(content::BrowserContext&);
  static BrowserContext& GetFrom(content::RenderFrameHost&);

  ApplicationPolicyService& application_policy() const;
  ApplicationContext& application_context() const { return *application_; }
  const AppManifest& manifest() const;

  BrowserContext(const BrowserContext&) = delete;
  BrowserContext& operator=(const BrowserContext&) = delete;
  ~BrowserContext() override;

 private:
  std::unique_ptr<content::ZoomLevelDelegate> CreateZoomLevelDelegate(
      const base::FilePath& partition_path) override;
  base::FilePath GetPath() const override;
  bool IsOffTheRecord() override;
  content::DownloadManagerDelegate* GetDownloadManagerDelegate() override;
  content::BrowserPluginGuestManager* GetGuestManager() override;
  ::storage::SpecialStoragePolicy* GetSpecialStoragePolicy() override;
  content::PlatformNotificationService* GetPlatformNotificationService()
      override;
  content::PushMessagingService* GetPushMessagingService() override;
  content::StorageNotificationService* GetStorageNotificationService() override;
  content::SSLHostStateDelegate* GetSSLHostStateDelegate() override;
  content::PermissionControllerDelegate* GetPermissionControllerDelegate()
      override;
  content::ReduceAcceptLanguageControllerDelegate*
  GetReduceAcceptLanguageControllerDelegate() override;
  content::ClientHintsControllerDelegate* GetClientHintsControllerDelegate()
      override;
  content::BackgroundFetchDelegate* GetBackgroundFetchDelegate() override;
  content::BackgroundSyncController* GetBackgroundSyncController() override;
  content::BrowsingDataRemoverDelegate* GetBrowsingDataRemoverDelegate()
      override;

  const raw_ref<ApplicationContext> application_;
  const base::FilePath path_;
  std::unique_ptr<ExplorerRouter> explorer_router_;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_BROWSER_CONTEXT_H_
