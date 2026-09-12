// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/download/download_service_impl.h"

#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/task/thread_pool.h"
#include "components/download/public/common/download_source.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/download_item_utils.h"
#include "net/base/filename_util.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/browser/explorer_router.h"
#include "tuffite/components/explorer/browser/explorer_surface_data.h"

namespace tuffite {
DownloadServiceImpl::DownloadServiceImpl(content::DownloadManager* manager)
    : manager_(manager) {
  manager_->AddObserver(this);
}
DownloadServiceImpl::~DownloadServiceImpl() {
  Shutdown();
}
content::DownloadManagerDelegate* DownloadServiceImpl::delegate() {
  return this;
}
void DownloadServiceImpl::Shutdown() {
  for (auto* item : items_) {
    item->RemoveObserver(this);
  }
  items_.clear();
  if (manager_) {
    manager_->SetDelegate(nullptr);
    manager_->RemoveObserver(this);
  }
  manager_ = nullptr;
}
void DownloadServiceImpl::GetNextId(content::DownloadIdCallback callback) {
  std::move(callback).Run(next_id_++);
}
bool DownloadServiceImpl::DetermineDownloadTarget(
    download::DownloadItem* item,
    download::DownloadTargetCallback* callback) {
  const auto name = net::GenerateFileName(
      item->GetURL(), item->GetContentDisposition(), "",
      item->GetSuggestedFilename(), item->GetMimeType(), "download");
  // Each item has its own destination, including simultaneous equal filenames.
  // No existing user file is overwritten and intermediate files remain private.
  const auto directory = manager_->GetBrowserContext()
                             ->GetPath()
                             .AppendASCII("Downloads")
                             .AppendASCII(item->GetGuid());
  base::ThreadPool::PostTaskAndReplyWithResult(
      FROM_HERE, {base::MayBlock()},
      base::BindOnce(
          [](base::FilePath directory, base::FilePath name) {
            download::DownloadTargetInfo info;
            if (!base::CreateDirectory(directory)) {
              info.interrupt_reason =
                  download::DOWNLOAD_INTERRUPT_REASON_FILE_FAILED;
              return info;
            }
            info.target_path = directory.Append(name);
            info.intermediate_path =
                info.target_path.AddExtensionASCII("crdownload");
            return info;
          },
          directory, name),
      std::move(*callback));
  return true;
}
void DownloadServiceImpl::OnDownloadCreated(content::DownloadManager*,
                                            download::DownloadItem* item) {
  if (auto* contents = content::DownloadItemUtils::GetWebContents(item);
      contents &&
      item->GetDownloadSource() == download::DownloadSource::NAVIGATION &&
      item->GetMimeType() == "application/pdf") {
    ExplorerSurfaceData::SetNavigationDownload(contents, item->GetId());
  }
  items_.insert(item);
  item->AddObserver(this);
  OnDownloadUpdated(item);
}
void DownloadServiceImpl::OnDownloadUpdated(download::DownloadItem* item) {
  auto* contents = content::DownloadItemUtils::GetWebContents(item);
  if (!contents) {
    return;
  }
  auto* host = ExplorerRouter::Get(contents).FindHostForContents(contents);
  if (!host) {
    return;
  }
  host->NotifyDownload(ExplorerHost::DownloadDetail(*item));
}

void DownloadServiceImpl::OnDownloadDestroyed(download::DownloadItem* item) {
  items_.erase(item);
}
}  // namespace tuffite
