// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/browser_context.h"

#include <algorithm>

#include "base/check.h"
#include "base/no_destructor.h"
#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "content/public/browser/render_frame_host.h"
#include "tuffite/components/dispatch/browser/host_dispatch.h"
#include "tuffite/components/explorer/browser/explorer_router.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/services/client_hints/client_hints_service.h"
#include "tuffite/framework/browser/services/client_hints/client_hints_service_factory.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service_factory.h"
#include "tuffite/framework/browser/services/devtools/devtools_service_factory.h"
#include "tuffite/framework/browser/services/download/download_service.h"
#include "tuffite/framework/browser/services/download/download_service_factory.h"
#include "tuffite/framework/browser/services/explorer/explorer_service_factory.h"
#include "tuffite/framework/browser/services/file_chooser/file_chooser_service_factory.h"
#include "tuffite/framework/browser/services/isolated/isolated_service_factory.h"
#include "tuffite/framework/browser/services/javascript_dialog/javascript_dialog_service_factory.h"
#include "tuffite/framework/browser/services/network_context/network_context_service_factory.h"
#include "tuffite/framework/browser/services/permission/permission_service.h"
#include "tuffite/framework/browser/services/permission/permission_service_factory.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service_factory.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service_factory.h"
#include "tuffite/framework/browser/services/window/window_service.h"
#include "tuffite/framework/browser/services/window/window_service_factory.h"

namespace tuffite {
namespace {
std::vector<BrowserContext*>& AliveBrowserContexts() {
  static base::NoDestructor<std::vector<BrowserContext*>> contexts;
  return *contexts;
}

}  // namespace

std::vector<base::FilePath> BrowserContext::GetNetworkStoragePaths() {
  std::vector<base::FilePath> paths;
  for (auto* context : AliveBrowserContexts()) {
    auto path = context->network_storage_root_.empty()
                    ? context->GetPath()
                    : context->network_storage_root_;
    if (std::ranges::find(paths, path) == paths.end()) {
      paths.push_back(std::move(path));
    }
  }
  return paths;
}

BrowserContext::BrowserContext(base::FilePath path,
                               ApplicationContext& application,
                               HostDispatch& host_dispatch,
                               base::FilePath network_storage_root,
                               UtilityService* utilities)
    : application_(application),
      host_dispatch_(host_dispatch),
      utilities_(utilities),
      path_(std::move(path)),
      network_storage_root_(std::move(network_storage_root)) {
  AliveBrowserContexts().push_back(this);
  ExplorerServiceFactory::Get();
  DownloadServiceFactory::Get();
  DevToolsServiceFactory::Get();
  IsolatedServiceFactory::Get();
  ContextMenuServiceFactory::Get();
  WebAuthenticationServiceFactory::Get();
  FileChooserServiceFactory::Get();
  PermissionServiceFactory::Get();
  ClientHintsServiceFactory::Get();
  NetworkContextServiceFactory::Get();
  JavaScriptDialogServiceFactory::Get();
  ShellApiServiceFactory::Get();
  BrowserContextDependencyManager::GetInstance()->CreateBrowserContextServices(
      this);
  explorer_router_ = std::make_unique<ExplorerRouter>(*this);
}

BrowserContext& BrowserContext::GetFrom(content::BrowserContext& context) {
  return static_cast<BrowserContext&>(context);
}

BrowserContext& BrowserContext::GetFrom(content::RenderFrameHost& frame) {
  return GetFrom(*frame.GetBrowserContext());
}

ManifestService& BrowserContext::manifest_service() const {
  auto* manifest = application_->GetService<ManifestService>();
  CHECK(manifest);
  return *manifest;
}

UtilityService& BrowserContext::utility_service() const {
  return utilities_ ? *utilities_ : *application_->GetService<UtilityService>();
}

BrowserContext::~BrowserContext() {
  weak_factory_.InvalidateWeakPtrs();
  NotifyWillBeDestroyed();
  if (auto* windows =
          window::WindowServiceFactory::GetForBrowserContext(this, false)) {
    windows->CloseContext(*this);
  }
  if (auto* downloads =
          DownloadServiceFactory::GetForBrowserContext(this, false)) {
    downloads->Shutdown();
  }
  explorer_router_.reset();
  // Network encryption services must survive until their partitions shut down.
  ShutdownStoragePartitions();
  std::erase(AliveBrowserContexts(), this);
  BrowserContextDependencyManager::GetInstance()->DestroyBrowserContextServices(
      this);
}

std::unique_ptr<content::ZoomLevelDelegate>
BrowserContext::CreateZoomLevelDelegate(const base::FilePath&) {
  return nullptr;
}

base::FilePath BrowserContext::GetPath() const {
  return path_;
}

bool BrowserContext::IsOffTheRecord() {
  return false;
}

content::DownloadManagerDelegate* BrowserContext::GetDownloadManagerDelegate() {
  return DownloadServiceFactory::GetForBrowserContext(this)->delegate();
}

content::BrowserPluginGuestManager* BrowserContext::GetGuestManager() {
  return nullptr;
}

::storage::SpecialStoragePolicy* BrowserContext::GetSpecialStoragePolicy() {
  return nullptr;
}

content::PlatformNotificationService*
BrowserContext::GetPlatformNotificationService() {
  return nullptr;
}

content::PushMessagingService* BrowserContext::GetPushMessagingService() {
  return nullptr;
}

content::StorageNotificationService*
BrowserContext::GetStorageNotificationService() {
  return nullptr;
}

content::SSLHostStateDelegate* BrowserContext::GetSSLHostStateDelegate() {
  return nullptr;
}

content::PermissionControllerDelegate*
BrowserContext::GetPermissionControllerDelegate() {
  return PermissionServiceFactory::GetForBrowserContext(this);
}

content::ReduceAcceptLanguageControllerDelegate*
BrowserContext::GetReduceAcceptLanguageControllerDelegate() {
  return nullptr;
}

content::ClientHintsControllerDelegate*
BrowserContext::GetClientHintsControllerDelegate() {
  return ClientHintsServiceFactory::GetForBrowserContext(this)->delegate();
}

content::BackgroundFetchDelegate* BrowserContext::GetBackgroundFetchDelegate() {
  return nullptr;
}

content::BackgroundSyncController*
BrowserContext::GetBackgroundSyncController() {
  return nullptr;
}

content::BrowsingDataRemoverDelegate*
BrowserContext::GetBrowsingDataRemoverDelegate() {
  return nullptr;
}

}  // namespace tuffite
