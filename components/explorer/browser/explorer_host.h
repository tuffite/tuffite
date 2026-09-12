// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_HOST_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_HOST_H_

#include <map>
#include <memory>
#include <optional>
#include <string>

#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "content/public/browser/document_service.h"
#include "content/public/browser/javascript_dialog_manager.h"
#include "content/public/browser/web_contents_delegate.h"
#include "content/public/browser/web_contents_observer.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/explorer/browser/explorer_surface_data.h"
#include "tuffite/components/explorer/common/explorer.mojom.h"
#include "url/gurl.h"

namespace download {
class DownloadItem;
}

namespace content {
class NavigationEntry;
class RenderFrameHost;
class SessionStorageNamespace;
class StoragePartitionConfig;
class WebContents;
}  // namespace content

namespace tuffite {
class ExplorerPolicyProvider;
class ExplorerTransferRecord;

class ExplorerHost final : public content::DocumentService<mojom::ExplorerHost>,
                           public content::WebContentsDelegate,
                           public content::WebContentsObserver {
 public:
  static void Bind(content::RenderFrameHost* render_frame_host,
                   mojo::PendingReceiver<mojom::ExplorerHost> receiver,
                   ExplorerPolicyProvider* policy);

  ExplorerHost(const ExplorerHost&) = delete;
  ExplorerHost& operator=(const ExplorerHost&) = delete;

  void Detach(mojo::PendingRemote<mojom::ExplorerTransferClient> client,
              DetachCallback callback) override;
  void Attach(const std::string& token,
              mojom::ExplorerCreateParamsPtr params,
              mojo::PendingRemote<mojom::ExplorerClient> client,
              AttachCallback callback) override;
  void Create(mojom::ExplorerCreateParamsPtr params,
              mojo::PendingRemote<mojom::ExplorerClient> client,
              CreateCallback callback) override;
  void Navigate(const std::string& url, NavigateCallback callback) override;
  void Go(int32_t offset, GoCallback callback) override;
  void Reload(bool bypass_cache, ReloadCallback callback) override;
  void Stop(StopCallback callback) override;
  void GetNavigationState(GetNavigationStateCallback callback) override;
  void ExecuteJavaScript(const std::string& source,
                         bool main_world,
                         const std::string& expected_document_id,
                         ExecuteJavaScriptCallback callback) override;
  void RespondToDialog(uint64_t request_id,
                       bool accept,
                       const std::string& prompt_text,
                       RespondToDialogCallback callback) override;
  void CompleteOpen(uint64_t request_id, bool propagate) override;
  void CompleteDialog(uint64_t request_id, bool propagate) override;
  void PostMessage(const std::string& json,
                   PostMessageCallback callback) override;
  void SetContextMenuEnabled(bool enabled) override;
  static std::string DownloadDetail(const download::DownloadItem& item);
  void NotifyDownload(const std::string& json);
  void Download(const std::string& url, DownloadCallback callback) override;
  void GetDownloads(GetDownloadsCallback callback) override;
  void CancelDownload(uint32_t id, CancelDownloadCallback callback) override;
  void ReadDownload(uint32_t id, ReadDownloadCallback callback) override;
  void CapturePreview(CapturePreviewCallback callback) override;

  // Called by the document-scoped surface router. The browser process owns
  // this lookup, so nested messages never depend on intermediate renderer JS.
  static bool DeliverMessageToOwner(content::WebContents* inner,
                                    const std::string& json,
                                    const std::string& from);
  // Returns true when a renderer-initiated current-surface navigation was
  // converted to an open event and must be cancelled by the throttle.
  static bool MaybeEmitNavigation(content::WebContents* contents,
                                  const GURL& url,
                                  bool user_gesture);
  static void ContinueOpenBubble(content::WebContents* source,
                                 const std::string& event_type,
                                 const std::string& request_json);
  static void ContinueDialogBubble(content::WebContents* source,
                                   uint64_t request_id,
                                   const std::string& kind,
                                   const std::string& message,
                                   const std::string& default_value,
                                   bool is_reload,
                                   const std::string& from);
  static bool RespondToDialogForTree(content::WebContents* requester,
                                     uint64_t request_id,
                                     bool accept,
                                     const std::string& prompt_text);
  void RequestDialog(
      const std::string& kind,
      const std::u16string& message,
      const std::u16string& default_value,
      bool is_reload,
      content::JavaScriptDialogManager::DialogClosedCallback callback,
      bool* did_suppress_message);

 private:
  friend class ExplorerRouter;
  const raw_ref<ExplorerPolicyProvider> policy_provider_;
  friend class ExplorerDialogManager;
  ExplorerHost(content::RenderFrameHost& render_frame_host,
               mojo::PendingReceiver<mojom::ExplorerHost> receiver,
               ExplorerPolicyProvider& policy);
  ~ExplorerHost() override;

  content::RenderFrameHost* ResolveDirectChild(
      const blink::LocalFrameToken& frame_token) const;
  bool CanNavigateTo(const GURL& url) const;
  bool ResolvePolicy(const std::string& partition,
                     ExplorerPolicy* policy) const;
  void AttachPrepared(std::unique_ptr<content::WebContents> inner_web_contents,
                      CreateCallback callback,
                      content::RenderFrameHost* prepared_frame);
  void BeginAttachAfterInitialNavigation();
  void CapturePreviewWhenReady(CapturePreviewCallback callback, int attempts);
  void AttachTransferred(AttachCallback callback,
                         content::RenderFrameHost* prepared_frame);
  void OnPartitionSnapshotComplete(GURL initial_url, bool success);
  bool LoadURL(const GURL& url);
  void NotifyLoadComplete();
  void NotifyRequestedOpen(const char* event_type,
                           const GURL& url,
                           WindowOpenDisposition disposition,
                           bool user_gesture);
  void EmitOpenBubble(const std::string& event_type,
                      const std::string& request_json);
  void EmitDialogBubble(uint64_t request_id,
                        const std::string& kind,
                        const std::string& message,
                        const std::string& default_value,
                        bool is_reload,
                        const std::string& from);
  std::string NavigationStateJson() const;
  std::string DocumentId() const;

  content::WebContents* OpenURLFromTab(
      content::WebContents* source,
      const content::OpenURLParams& params,
      base::OnceCallback<void(content::NavigationHandle&)>
          navigation_handle_callback) override;
  bool IsWebContentsCreationOverridden(
      content::RenderFrameHost* opener,
      content::SiteInstance* source_site_instance,
      content::mojom::WindowContainerType window_container_type,
      const GURL& opener_url,
      const std::string& frame_name,
      const GURL& target_url) override;
  content::WebContents* CreateCustomWebContents(
      content::RenderFrameHost* opener,
      content::SiteInstance* source_site_instance,
      bool is_new_browsing_instance,
      const GURL& opener_url,
      const std::string& frame_name,
      const GURL& target_url,
      WindowOpenDisposition disposition,
      const blink::mojom::WindowFeatures& window_features,
      const content::StoragePartitionConfig& partition_config,
      content::SessionStorageNamespace* session_storage_namespace) override;
  content::WebContents* AddNewContents(
      content::WebContents* source,
      std::unique_ptr<content::WebContents> new_contents,
      const GURL& target_url,
      WindowOpenDisposition disposition,
      const blink::mojom::WindowFeatures& window_features,
      bool user_gesture,
      bool* was_blocked) override;
  content::JavaScriptDialogManager* GetJavaScriptDialogManager(
      content::WebContents* source) override;
  void RunFileChooser(content::RenderFrameHost* frame,
                      scoped_refptr<content::FileSelectListener> listener,
                      const blink::mojom::FileChooserParams& params) override;
  void EnumerateDirectory(content::WebContents* contents,
                          scoped_refptr<content::FileSelectListener> listener,
                          const base::FilePath& path) override;
  void EnterFullscreenModeForTab(
      content::RenderFrameHost* requesting_frame,
      const blink::mojom::FullscreenOptions& options) override;
  void ExitFullscreenModeForTab(content::WebContents* web_contents) override;
  bool IsFullscreenForTabOrPending(
      const content::WebContents* web_contents) override;
  content::PictureInPictureResult EnterPictureInPicture(
      content::WebContents* web_contents) override;
  void ExitPictureInPicture() override;

  void DidStartLoading() override;
  void DidStopLoading() override;
  void DidFinishNavigation(
      content::NavigationHandle* navigation_handle) override;
  void TitleWasSet(content::NavigationEntry* entry) override;

  base::WeakPtr<ExplorerTransferRecord> attaching_transfer_;
  mojo::Remote<mojom::ExplorerClient> client_;
  std::unique_ptr<content::WebContents> pending_inner_web_contents_;
  std::optional<blink::LocalFrameToken> pending_frame_token_;
  std::optional<int> primary_navigation_error_;
  std::optional<ExplorerPolicy> policy_;
  std::string created_partition_domain_;
  std::string created_partition_name_;
  bool created_partition_in_memory_ = false;
  bool created_partition_is_default_ = true;
  int owner_renderer_process_id_ = -1;
  std::unique_ptr<class ExplorerDialogManager> dialog_manager_;
  struct PendingOpenBubble {
    std::string event_type;
    std::string request_json;
  };
  struct PendingDialogBubble {
    uint64_t dialog_request_id = 0;
    std::string kind;
    std::string message;
    std::string default_value;
    bool is_reload = false;
    std::string from;
  };
  std::map<uint64_t, PendingOpenBubble> pending_open_bubbles_;
  std::map<uint64_t, PendingDialogBubble> pending_dialog_bubbles_;
  std::string document_id_;
  CreateCallback pending_create_callback_;
  bool preparing_attachment_ = false;
  bool partition_snapshot_pending_ = false;
  base::WeakPtrFactory<ExplorerHost> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_HOST_H_
