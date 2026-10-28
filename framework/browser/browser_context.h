// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_BROWSER_CONTEXT_H_
#define TUFFITE_FRAMEWORK_BROWSER_BROWSER_CONTEXT_H_

#include "base/files/file_path.h"
#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "content/public/browser/browser_context.h"

namespace content {
class RenderFrameHost;
}

namespace tuffite {

class ExplorerRouter;
class ManifestService;
class UtilityService;
class ApplicationContext;
class HostDispatch;

class BrowserContext final : public content::BrowserContext {
 public:
  BrowserContext(base::FilePath path,
                 ApplicationContext& application,
                 HostDispatch& host_dispatch,
                 base::FilePath network_storage_root = {},
                 UtilityService* utilities = nullptr);

  // Live profile roots granted to the sandboxed network process.
  static std::vector<base::FilePath> GetNetworkStoragePaths();
  static BrowserContext& GetFrom(content::BrowserContext&);
  static BrowserContext& GetFrom(content::RenderFrameHost&);

  ManifestService& manifest_service() const;
  UtilityService& utility_service() const;
  base::WeakPtr<BrowserContext> GetWeakPtr() {
    return weak_factory_.GetWeakPtr();
  }
  ApplicationContext& application_context() const { return *application_; }

  HostDispatch& host_dispatch() const { return *host_dispatch_; }

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
  const raw_ref<HostDispatch> host_dispatch_;
  raw_ptr<UtilityService> utilities_ = nullptr;
  const base::FilePath path_;
  const base::FilePath network_storage_root_;
  std::unique_ptr<ExplorerRouter> explorer_router_;
  base::WeakPtrFactory<BrowserContext> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_BROWSER_CONTEXT_H_
