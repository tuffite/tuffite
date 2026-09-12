// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/browser/explorer_transfer.h"

#include "base/functional/bind.h"
#include "base/unguessable_token.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/explorer/browser/explorer_router.h"

namespace tuffite {
DOCUMENT_USER_DATA_KEY_IMPL(ExplorerTransferStore);
ExplorerTransferStore::ExplorerTransferStore(content::RenderFrameHost* frame)
    : DocumentUserData(frame),
      WebContentsObserver(content::WebContents::FromRenderFrameHost(frame)) {}
ExplorerTransferStore::~ExplorerTransferStore() = default;
void ExplorerTransferStore::RenderFrameDeleted(
    content::RenderFrameHost* frame) {
  if (frame == &render_frame_host()) {
    records_.clear();
  }
}
void ExplorerTransferStore::PrimaryPageChanged(content::Page&) {
  // DocumentUserData can survive navigation in BFCache, but abandoned live
  // pages must not remain owned by an inactive source document.
  if (!render_frame_host().IsActive()) {
    records_.clear();
  }
}
void ExplorerTransferStore::RenderFrameHostChanged(
    content::RenderFrameHost* old_host,
    content::RenderFrameHost* new_host) {
  // Subframe navigation need not change the WebContents' primary page, and
  // the old frame can remain alive while running unload handlers.
  if (old_host == &render_frame_host()) {
    records_.clear();
  }
}
ExplorerTransferRecord& ExplorerTransferStore::Add(
    std::unique_ptr<content::WebContents> contents,
    ExplorerPolicy policy,
    std::string document_id,
    mojo::PendingRemote<mojom::ExplorerTransferClient> client) {
  auto record = std::make_unique<ExplorerTransferRecord>(
      *this, std::move(contents), std::move(policy), std::move(document_id),
      std::move(client));
  auto& result = *record;
  records_.emplace(result.token(), std::move(record));
  return result;
}
void ExplorerTransferStore::Remove(const std::string& token) {
  records_.erase(token);
}
ExplorerTransferRecord::ExplorerTransferRecord(
    ExplorerTransferStore& store,
    std::unique_ptr<content::WebContents> contents,
    ExplorerPolicy policy,
    std::string document_id,
    mojo::PendingRemote<mojom::ExplorerTransferClient> client)
    : WebContentsObserver(contents.get()),
      store_(store),
      token_(base::UnguessableToken::Create().ToString()),
      owner_origin_(store.render_frame_host().GetLastCommittedOrigin()),
      contents_(std::move(contents)),
      policy_(std::move(policy)),
      document_id_(std::move(document_id)),
      client_(std::move(client)) {
  contents_->SetDelegate(this);
  ExplorerRouter::Get(contents_.get()).AddTransfer(*this);
}
ExplorerTransferRecord::~ExplorerTransferRecord() {
  SetState(state_ == "transferred" ? "transferred" : "closed");
  ExplorerRouter::Get(store_->render_frame_host().GetBrowserContext())
      .RemoveTransfer(*this);
  if (contents_) {
    contents_->SetDelegate(nullptr);
  }
  Observe(nullptr);
}
void ExplorerTransferRecord::SetState(const std::string& state) {
  if (state_ == state) {
    return;
  }
  state_ = state;
  if (client_) {
    client_->StateChanged(state);
  }
}
mojo::PendingRemote<mojom::ExplorerTransferControl>
ExplorerTransferRecord::BindControl() {
  auto remote = receiver_.BindNewPipeAndPassRemote();
  receiver_.set_disconnect_handler(base::BindOnce(
      [](base::WeakPtr<ExplorerTransferRecord> record) {
        if (record && !record->accepted_ && record->pending()) {
          record->Remove();
        }
      },
      weak_factory_.GetWeakPtr()));
  return remote;
}
std::unique_ptr<content::WebContents> ExplorerTransferRecord::TakeContents() {
  Observe(nullptr);
  contents_->SetDelegate(nullptr);
  return std::move(contents_);
}
void ExplorerTransferRecord::Remove() {
  // Copy before erasing this record; the map must not refer to its token
  // storage.
  const std::string token = token_;
  store_->Remove(token);
}
base::WeakPtr<ExplorerTransferRecord> ExplorerTransferRecord::GetWeakPtr() {
  return weak_factory_.GetWeakPtr();
}
void ExplorerTransferRecord::Accept() {
  accepted_ = true;
}
void ExplorerTransferRecord::Close(CloseCallback callback) {
  if (!pending()) {
    std::move(callback).Run("InvalidStateError");
    return;
  }
  SetState("closed");
  std::move(callback).Run("");
  Remove();
}
void ExplorerTransferRecord::DidFinishNavigation(
    content::NavigationHandle* nav) {
  if (nav->HasCommitted() && nav->IsInPrimaryMainFrame() &&
      !nav->IsSameDocument()) {
    document_id_ = base::UnguessableToken::Create().ToString();
  }
}
content::WebContents* ExplorerTransferRecord::OpenURLFromTab(
    content::WebContents*,
    const content::OpenURLParams&,
    base::OnceCallback<void(content::NavigationHandle&)>) {
  return nullptr;
}
bool ExplorerTransferRecord::IsWebContentsCreationOverridden(
    content::RenderFrameHost*,
    content::SiteInstance*,
    content::mojom::WindowContainerType,
    const GURL&,
    const std::string&,
    const GURL&) {
  return true;
}
content::WebContents* ExplorerTransferRecord::CreateCustomWebContents(
    content::RenderFrameHost*,
    content::SiteInstance*,
    bool,
    const GURL&,
    const std::string&,
    const GURL&,
    WindowOpenDisposition,
    const blink::mojom::WindowFeatures&,
    const content::StoragePartitionConfig&,
    content::SessionStorageNamespace*) {
  return nullptr;
}
}  // namespace tuffite
