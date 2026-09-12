// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_DOWNLOAD_DOWNLOAD_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_DOWNLOAD_DOWNLOAD_SERVICE_IMPL_H_
#include <set>

#include "base/memory/raw_ptr.h"
#include "content/public/browser/download_manager.h"
#include "content/public/browser/download_manager_delegate.h"
#include "tuffite/framework/browser/services/download/download_service.h"
namespace tuffite {
class DownloadServiceImpl final : public DownloadService,
                                  public content::DownloadManagerDelegate,
                                  public content::DownloadManager::Observer,
                                  public download::DownloadItem::Observer {
 public:
  explicit DownloadServiceImpl(content::DownloadManager* manager);
  ~DownloadServiceImpl() override;
  content::DownloadManagerDelegate* delegate() override;
  void Shutdown() override;
  void GetNextId(content::DownloadIdCallback callback) override;
  bool DetermineDownloadTarget(
      download::DownloadItem* item,
      download::DownloadTargetCallback* callback) override;
  void OnDownloadCreated(content::DownloadManager*,
                         download::DownloadItem*) override;
  void OnDownloadUpdated(download::DownloadItem*) override;
  void OnDownloadDestroyed(download::DownloadItem*) override;

 private:
  raw_ptr<content::DownloadManager> manager_;
  std::set<download::DownloadItem*> items_;
  uint32_t next_id_ = 1;
};
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_DOWNLOAD_DOWNLOAD_SERVICE_IMPL_H_
