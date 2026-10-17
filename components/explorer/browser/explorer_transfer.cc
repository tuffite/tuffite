// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/browser/explorer_transfer.h"

#include "base/functional/bind.h"
#include "base/unguessable_token.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/explorer/browser/explorer_policy_provider.h"
#include "tuffite/components/explorer/browser/explorer_router.h"
#include "tuffite/components/explorer/browser/explorer_surface_host.h"

namespace tuffite {
DOCUMENT_USER_DATA_KEY_IMPL(ExplorerTransferStore);
ExplorerTransferStore::ExplorerTransferStore(content::RenderFrameHost* frame)
    : DocumentUserData(frame),
      WebContentsObserver(content::WebContents::FromRenderFrameHost(frame)) {}
ExplorerTransferStore::~ExplorerTransferStore() {
  Clear();
}
void ExplorerTransferStore::Clear() {
  auto records = std::move(records_);
  for (auto& [token, record] : records) {
    if (record->pipe_owned()) {
      auto* context = render_frame_host().GetBrowserContext();
      record->ReleaseDocument();
      ExplorerRouter::Get(context).HoldTransfer(std::move(record));
    }
  }
}
void ExplorerTransferStore::RenderFrameDeleted(
    content::RenderFrameHost* frame) {
  if (frame == &render_frame_host()) {
    Clear();
  }
}
void ExplorerTransferStore::PrimaryPageChanged(content::Page&) {
  // DocumentUserData can survive navigation in BFCache, but abandoned live
  // pages must not remain owned by an inactive source document.
  if (!render_frame_host().IsActive()) {
    Clear();
  }
}
void ExplorerTransferStore::RenderFrameHostChanged(
    content::RenderFrameHost* old_host,
    content::RenderFrameHost* new_host) {
  // Subframe navigation need not change the WebContents' primary page, and
  // the old frame can remain alive while running unload handlers.
  if (old_host == &render_frame_host()) {
    Clear();
  }
}
ExplorerTransferRecord& ExplorerTransferStore::Add(
    std::unique_ptr<content::WebContents> contents,
    ExplorerPolicy policy,
    std::string document_id,
    mojo::PendingRemote<mojom::ExplorerTransferClient> client,
    ExplorerPolicyProvider* provider) {
  auto record = std::make_unique<ExplorerTransferRecord>(
      *this, std::move(contents), std::move(policy), std::move(document_id),
      std::move(client), provider);
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
    mojo::PendingRemote<mojom::ExplorerTransferClient> client,
    ExplorerPolicyProvider* provider)
    : WebContentsObserver(contents.get()),
      store_(&store),
      context_(store.render_frame_host().GetBrowserContext()),
      provider_(provider),
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
  if (is_window_) {
    ExplorerSurfaceHost::FinishRequest(
        context_, "newwindow", token_,
        state_ != "transferred" && state_ != "discarded");
  }
  SetState(state_ == "transferred" ? "transferred" : "closed");
  ExplorerRouter::Get(context_).RemoveTransfer(*this);
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
        if (record && record->pending()) {
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
  if (store_) {
    store_->Remove(token);
  } else {
    ExplorerRouter::Get(context_).DropHeldTransfer(token);
  }
}
base::WeakPtr<ExplorerTransferRecord> ExplorerTransferRecord::GetWeakPtr() {
  return weak_factory_.GetWeakPtr();
}
void ExplorerTransferRecord::OfferWindow(content::RenderFrameHost& frame) {
  is_window_ = true;
  claimant_ = frame.GetWeakDocumentPtr();
}
bool ExplorerTransferRecord::OfferedTo(content::RenderFrameHost& frame) const {
  return is_window_ && claimant_.AsRenderFrameHostIfValid() == &frame;
}
bool ExplorerTransferRecord::DiscardWindow(content::RenderFrameHost& frame,
                                           const std::string& token,
                                           bool canceled) {
  auto* record =
      ExplorerRouter::Get(frame.GetBrowserContext()).FindTransfer(token);
  if (!record || !record->pending() || !record->OfferedTo(frame)) {
    return false;
  }
  record->SetState(canceled ? "canceled" : "discarded");
  ExplorerSurfaceHost::FinishRequest(frame.GetBrowserContext(), "newwindow",
                                     token, canceled);
  // Let Chromium complete the renderer's close handshake. Destroying a newly
  // created page here can strand its opener inside window.open().
  record->contents()->ClosePage();
  return true;
}
void ExplorerTransferRecord::CloseContents(content::WebContents* contents) {
  if (contents != contents_.get()) {
    return;
  }
  if (pending()) {
    SetState(is_window_ ? "discarded" : "closed");
  }
  Remove();
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
void ExplorerTransferRecord::RequestKeyboardLock(content::WebContents* contents,
                                                 bool esc_key_locked) {
  // Detached pages have neither a visible owner nor consent/exit UI.
  contents->GotResponseToKeyboardLockRequest(false);
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

namespace tuffite {
std::unique_ptr<ExplorerTransferRecord> ExplorerTransferStore::Take(
    const std::string& token) {
  auto node = records_.extract(token);
  return node.empty() ? nullptr : std::move(node.mapped());
}
void ExplorerTransferRecord::RebindClient(
    mojo::PendingRemote<mojom::ExplorerTransferClient> client) {
  if (!pending() || !transferred_handle_) {
    return;
  }
  client_.reset();
  client_.Bind(std::move(client));
}
void ExplorerTransferRecord::BeginTransfer() {
  if (pending() && !is_window_) {
    transferred_handle_ = true;
  }
}
}  // namespace tuffite
