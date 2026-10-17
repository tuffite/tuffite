// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_TRANSFER_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_TRANSFER_H_

#include <map>
#include <memory>
#include <set>
#include <string>
#include <vector>

#include "base/functional/callback.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "content/public/browser/document_user_data.h"
#include "content/public/browser/weak_document_ptr.h"
#include "content/public/browser/web_contents_delegate.h"
#include "content/public/browser/web_contents_observer.h"
#include "mojo/public/cpp/bindings/receiver.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/explorer/browser/explorer_policy.h"
#include "tuffite/components/explorer/common/explorer.mojom.h"
#include "url/origin.h"

namespace tuffite {
class ExplorerTransferStore;
class ExplorerPolicyProvider;

// Indexed by DocumentUserData. Locally detached pages are released when their
// exclusive control pipe closes; named resources belong to the receiving doc.
// Detached pages retain their renderer but cannot open native windows/dialogs.
class ExplorerTransferRecord final : public mojom::ExplorerTransferControl,
                                     public content::WebContentsDelegate,
                                     public content::WebContentsObserver {
 public:
  ExplorerTransferRecord(
      ExplorerTransferStore& store,
      std::unique_ptr<content::WebContents> contents,
      ExplorerPolicy policy,
      std::string document_id,
      mojo::PendingRemote<mojom::ExplorerTransferClient> client,
      ExplorerPolicyProvider* provider);
  ~ExplorerTransferRecord() override;
  const std::string& token() const { return token_; }
  content::WebContents* contents() const { return contents_.get(); }
  const ExplorerPolicy& policy() const { return policy_; }
  const std::string& document_id() const { return document_id_; }
  const url::Origin& owner_origin() const { return owner_origin_; }
  void OfferWindow(content::RenderFrameHost& frame);
  bool is_window() const { return is_window_; }
  bool OfferedTo(content::RenderFrameHost& frame) const;
  static bool DiscardWindow(content::RenderFrameHost& frame,
                            const std::string& token,
                            bool canceled = false);
  bool pending() const { return state_ == "pending"; }
  void SetState(const std::string& state);
  mojo::PendingRemote<mojom::ExplorerTransferControl> BindControl();
  std::unique_ptr<content::WebContents> TakeContents();
  void Remove();
  base::WeakPtr<ExplorerTransferRecord> GetWeakPtr();
  void Accept() override;
  void Close(CloseCallback callback) override;
  void BeginTransfer() override;
  void RebindClient(
      mojo::PendingRemote<mojom::ExplorerTransferClient>) override;
  bool transferred_handle() const { return transferred_handle_; }
  bool pipe_owned() const { return !is_window_ && receiver_.is_bound(); }
  void ReleaseDocument() { store_ = nullptr; }

 private:
  void CloseContents(content::WebContents*) override;
  void RequestKeyboardLock(content::WebContents* contents,
                           bool esc_key_locked) override;
  void DidFinishNavigation(content::NavigationHandle*) override;
  content::WebContents* OpenURLFromTab(
      content::WebContents*,
      const content::OpenURLParams&,
      base::OnceCallback<void(content::NavigationHandle&)>) override;
  bool IsWebContentsCreationOverridden(content::RenderFrameHost*,
                                       content::SiteInstance*,
                                       content::mojom::WindowContainerType,
                                       const GURL&,
                                       const std::string&,
                                       const GURL&) override;
  content::WebContents* CreateCustomWebContents(
      content::RenderFrameHost*,
      content::SiteInstance*,
      bool,
      const GURL&,
      const std::string&,
      const GURL&,
      WindowOpenDisposition,
      const blink::mojom::WindowFeatures&,
      const content::StoragePartitionConfig&,
      content::SessionStorageNamespace*) override;
  raw_ptr<ExplorerTransferStore> store_;
  raw_ptr<content::BrowserContext> context_;
  bool transferred_handle_ = false;
  raw_ptr<ExplorerPolicyProvider> provider_;
  const std::string token_;
  url::Origin owner_origin_;
  std::unique_ptr<content::WebContents> contents_;
  const ExplorerPolicy policy_;
  std::string document_id_;
  std::string state_ = "pending";
  bool accepted_ = false;
  bool is_window_ = false;
  content::WeakDocumentPtr claimant_;
  mojo::Remote<mojom::ExplorerTransferClient> client_;
  mojo::Receiver<mojom::ExplorerTransferControl> receiver_{this};
  base::WeakPtrFactory<ExplorerTransferRecord> weak_factory_{this};
};

class ExplorerTransferStore final
    : public content::DocumentUserData<ExplorerTransferStore>,
      public content::WebContentsObserver {
 public:
  ~ExplorerTransferStore() override;
  ExplorerTransferRecord& Add(
      std::unique_ptr<content::WebContents>,
      ExplorerPolicy,
      std::string,
      mojo::PendingRemote<mojom::ExplorerTransferClient>,
      ExplorerPolicyProvider* provider = nullptr);
  void Remove(const std::string& token);
  std::unique_ptr<ExplorerTransferRecord> Take(const std::string&);

 private:
  explicit ExplorerTransferStore(content::RenderFrameHost*);
  void Clear();
  void RenderFrameDeleted(content::RenderFrameHost*) override;
  void RenderFrameHostChanged(content::RenderFrameHost*,
                              content::RenderFrameHost*) override;
  void PrimaryPageChanged(content::Page&) override;
  friend class content::DocumentUserData<ExplorerTransferStore>;
  DOCUMENT_USER_DATA_KEY_DECL();
  std::map<std::string, std::unique_ptr<ExplorerTransferRecord>> records_;
};
}  // namespace tuffite
#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_TRANSFER_H_
