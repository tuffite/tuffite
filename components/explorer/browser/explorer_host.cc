// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#include "tuffite/components/explorer/browser/explorer_host.h"

#include <algorithm>
#include <atomic>
#include <utility>

#include "base/base64.h"
#include "base/command_line.h"
#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/logging.h"
#include "base/memory/raw_ref.h"
#include "base/no_destructor.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_util.h"
#include "base/strings/utf_string_conversions.h"
#include "base/task/single_thread_task_runner.h"
#include "base/task/thread_pool.h"
#include "base/unguessable_token.h"
#include "components/download/public/common/download_source.h"
#include "components/viz/common/frame_sinks/copy_output_result.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/child_process_security_policy.h"
#include "content/public/browser/download_item_utils.h"
#include "content/public/browser/download_manager.h"
#include "content/public/browser/file_select_listener.h"
#include "content/public/browser/global_routing_id.h"
#include "content/public/browser/navigation_controller.h"
#include "content/public/browser/navigation_entry.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/page_navigator.h"
#include "content/public/browser/reload_type.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/render_process_host.h"
#include "content/public/browser/render_widget_host_view.h"
#include "content/public/browser/security_principal.h"
#include "content/public/browser/site_instance.h"
#include "content/public/browser/storage_partition.h"
#include "content/public/browser/storage_partition_config.h"
#include "content/public/browser/web_contents.h"
#include "content/public/common/content_switches.h"
#include "content/public/common/isolated_world_ids.h"
#include "content/public/common/javascript_dialog_type.h"
#include "net/base/net_errors.h"
#include "third_party/blink/public/common/messaging/transferable_message.h"
#include "third_party/blink/public/common/renderer_preferences/renderer_preferences.h"
#include "third_party/blink/public/mojom/window_features/window_features.mojom.h"
#include "tuffite/components/explorer/browser/content/explorer_isolation.h"
#include "tuffite/components/explorer/browser/explorer_browser_support.h"
#include "tuffite/components/explorer/browser/explorer_partition.h"
#include "tuffite/components/explorer/browser/explorer_partition_snapshot.h"
#include "tuffite/components/explorer/browser/explorer_policy_provider.h"
#include "tuffite/components/explorer/browser/explorer_router.h"
#include "tuffite/components/explorer/browser/explorer_surface_host.h"
#include "tuffite/components/explorer/browser/explorer_transfer.h"
#include "ui/base/page_transition_types.h"
#include "ui/base/window_open_disposition.h"
#include "ui/gfx/codec/png_codec.h"
#include "ui/gfx/geometry/rect.h"

namespace tuffite {
namespace {

constexpr size_t kMaxScriptBytes = 1024 * 1024;
constexpr size_t kMaxPendingBubbleRequests = 256;
constexpr int32_t kExplorerIsolatedWorldId =
    content::ISOLATED_WORLD_ID_CONTENT_END;
constexpr char kExplorerPartitionDomain[] = "tuffite-explorer";

int64_t NextExplorerIsolationId() {
  // Keep Explorer ids in a range navigation ids will not realistically reach
  // while still satisfying EmbedderIsolationInfo's non-negative contract.
  static std::atomic<int64_t> next_id{int64_t{1} << 62};
  return next_id.fetch_add(1, std::memory_order_relaxed);
}

}  // namespace

class ExplorerDialogManager final : public content::JavaScriptDialogManager,
                                    public content::WebContentsObserver {
 public:
  explicit ExplorerDialogManager(ExplorerHost& host)
      : WebContentsObserver(host.WebContentsObserver::web_contents()),
        host_(host) {}
  ~ExplorerDialogManager() override = default;

  void RunJavaScriptDialog(content::WebContents* web_contents,
                           content::RenderFrameHost* render_frame_host,
                           content::JavaScriptDialogType dialog_type,
                           const std::u16string& message_text,
                           const std::u16string& default_prompt_text,
                           DialogClosedCallback callback,
                           bool* did_suppress_message) override {
    const char* kind = "alert";
    if (dialog_type == content::JAVASCRIPT_DIALOG_TYPE_CONFIRM) {
      kind = "confirm";
    } else if (dialog_type == content::JAVASCRIPT_DIALOG_TYPE_PROMPT) {
      kind = "prompt";
    }
    source_document_ = render_frame_host->GetWeakDocumentPtr();
    host_->RequestDialog(kind, message_text, default_prompt_text, false,
                         std::move(callback), did_suppress_message);
  }

  void RunBeforeUnloadDialog(content::WebContents* web_contents,
                             content::RenderFrameHost* render_frame_host,
                             bool is_reload,
                             DialogClosedCallback callback) override {
    source_document_ = render_frame_host->GetWeakDocumentPtr();
    host_->RequestDialog("beforeunload", std::u16string(), std::u16string(),
                         is_reload, std::move(callback), nullptr);
  }

  bool HandleJavaScriptDialog(content::WebContents* web_contents,
                              bool accept,
                              const std::u16string* prompt_override) override {
    return Respond(state_.pending_request_id(), accept,
                   prompt_override ? *prompt_override : std::u16string());
  }

  void CancelDialogs(content::WebContents* web_contents,
                     bool reset_state) override {
    const uint64_t request_id = state_.pending_request_id();
    if (request_id) {
      ExplorerSurfaceHost::ForgetDialog(
          host_->render_frame_host().GetBrowserContext(), request_id);
    }
    state_.Cancel();
  }

  bool Respond(uint64_t request_id,
               bool accept,
               const std::u16string& prompt_text,
               bool canceled = false) {
    if (!state_.has_pending_dialog() ||
        request_id != state_.pending_request_id()) {
      return false;
    }
    auto* source = source_document_.AsRenderFrameHostIfValid();
    if (!source || !source->IsActive()) {
      CancelDialogs(nullptr, true);
      return false;
    }
    ExplorerSurfaceHost::ForgetDialog(
        host_->render_frame_host().GetBrowserContext(), request_id, canceled);
    return state_.Respond(request_id, accept, prompt_text);
  }

  uint64_t Begin(DialogClosedCallback callback) {
    const uint64_t previous_request_id = state_.pending_request_id();
    if (previous_request_id) {
      ExplorerSurfaceHost::ForgetDialog(
          host_->render_frame_host().GetBrowserContext(), previous_request_id);
    }
    // Begin dismisses an older callback before installing the new one. Its
    // surface bookkeeping has already been removed above.
    return state_.Begin(std::move(callback));
  }

 private:
  void RenderFrameDeleted(content::RenderFrameHost* frame) override {
    if (state_.has_pending_dialog() &&
        (!source_document_.AsRenderFrameHostIfValid() ||
         source_document_.AsRenderFrameHostIfValid() == frame)) {
      CancelDialogs(nullptr, true);
    }
  }
  void RenderFrameHostChanged(content::RenderFrameHost* old_frame,
                              content::RenderFrameHost*) override {
    if (state_.has_pending_dialog() &&
        (!source_document_.AsRenderFrameHostIfValid() ||
         source_document_.AsRenderFrameHostIfValid() == old_frame)) {
      CancelDialogs(nullptr, true);
    }
  }
  void DidFinishNavigation(content::NavigationHandle* navigation) override {
    if (navigation->HasCommitted() && !navigation->IsSameDocument() &&
        state_.has_pending_dialog() &&
        !source_document_.AsRenderFrameHostIfValid()) {
      CancelDialogs(nullptr, true);
    }
  }
  content::WeakDocumentPtr source_document_;
  raw_ref<ExplorerHost> host_;
  ExplorerDialogState state_;
};

// static
void ExplorerHost::Bind(content::RenderFrameHost* render_frame_host,
                        mojo::PendingReceiver<mojom::ExplorerHost> receiver,
                        ExplorerPolicyProvider* policy) {
  if (!render_frame_host || !policy ||
      !policy->IsOwnerAllowed(*render_frame_host)) {
    return;
  }
  new ExplorerHost(*render_frame_host, std::move(receiver), *policy);
}

ExplorerHost::ExplorerHost(content::RenderFrameHost& render_frame_host,
                           mojo::PendingReceiver<mojom::ExplorerHost> receiver,
                           ExplorerPolicyProvider& policy)
    : DocumentService(render_frame_host, std::move(receiver)),
      policy_provider_(policy) {
  ExplorerRouter::Get(render_frame_host.GetBrowserContext()).AddHost(*this);
}

ExplorerHost::~ExplorerHost() {
  auto windows = std::move(offered_windows_);
  for (const auto& token : windows) {
    ExplorerTransferRecord::DiscardWindow(render_frame_host(), token, true);
  }
  auto dialogs = std::move(offered_dialogs_);
  for (auto id : dialogs) {
    RespondToDialogForTree(WebContentsObserver::web_contents(), id, false, "",
                           true);
  }
  if (attaching_transfer_) {
    attaching_transfer_->SetState("pending");
  }
  ExplorerRouter::Get(render_frame_host().GetBrowserContext())
      .RemoveHost(*this);
  if (dialog_manager_) {
    dialog_manager_->CancelDialogs(nullptr, true);
  }
  if (content::WebContents* contents = WebContentsObserver::web_contents();
      contents && contents->GetDelegate() == this) {
    contents->SetDelegate(nullptr);
  }
}

content::RenderFrameHost* ExplorerHost::ResolveDirectChild(
    const blink::LocalFrameToken& frame_token) const {
  auto* child = content::RenderFrameHost::FromFrameToken(
      content::GlobalRenderFrameHostToken(
          render_frame_host().GetProcess()->GetDeprecatedID(), frame_token));
  return child && child->GetParent() == &render_frame_host() ? child : nullptr;
}

bool ExplorerHost::ResolvePolicy(const std::string& partition,
                                 ExplorerPolicy* policy) const {
  return policy_provider_->Resolve(render_frame_host(), partition, policy);
}

bool ExplorerHost::CanNavigateTo(const GURL& url) const {
  return policy_provider_->NavigationAllowed(render_frame_host(), url) &&
         (url.SchemeIsFile() ||
          content::ChildProcessSecurityPolicy::GetInstance()->CanRequestURL(
              render_frame_host().GetProcess()->GetDeprecatedID(), url));
}

void ExplorerHost::Detach(
    mojo::PendingRemote<mojom::ExplorerTransferClient> client,
    DetachCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  if (!render_frame_host().IsActive() || !contents ||
      !contents->GetOuterWebContents() || !policy_ || preparing_attachment_ ||
      attaching_transfer_) {
    std::move(callback).Run("", {}, "InvalidStateError");
    return;
  }
  if (dialog_manager_) {
    dialog_manager_->CancelDialogs(nullptr, true);
  }
  dialog_manager_.reset();
  active_find_request_id_ = -1;
  find_text_.clear();
  contents->StopFinding(content::STOP_FIND_ACTION_CLEAR_SELECTION);
  pending_open_bubbles_.clear();
  pending_dialog_bubbles_.clear();
  Observe(nullptr);
  contents->SetDelegate(nullptr);
  client_.reset();
  SetExplorerOwner(contents, nullptr, false);
  auto owned = DetachExplorerWebContents(contents);
  auto& record =
      ExplorerTransferStore::GetOrCreateForCurrentDocument(&render_frame_host())
          ->Add(std::move(owned), *policy_, document_id_, std::move(client),
                &*policy_provider_);
  policy_.reset();
  document_id_.clear();
  auto token = record.token();
  auto control = record.BindControl();
  ExplorerSurfaceHost::UpdateFrameAncestors(
      record.contents(),
      base::BindOnce(
          [](DetachCallback callback, std::string token,
             mojo::PendingRemote<mojom::ExplorerTransferControl> control,
             bool ready) {
            std::move(callback).Run(token, std::move(control),
                                    ready ? "" : "AbortError");
          },
          std::move(callback), token, std::move(control)));
}

void ExplorerHost::Attach(const std::string& token,
                          mojom::ExplorerCreateParamsPtr params,
                          mojo::PendingRemote<mojom::ExplorerClient> client,
                          AttachCallback callback) {
  auto& router = ExplorerRouter::Get(render_frame_host().GetBrowserContext());
  auto* record = router.FindTransfer(token);
  if (!render_frame_host().IsActive() || !record || !record->pending() ||
      !params || WebContentsObserver::web_contents() || client_.is_bound() ||
      pending_inner_web_contents_ || attaching_transfer_) {
    std::move(callback).Run("InvalidStateError");
    return;
  }
  if (ExplorerAttachWouldCycle(record->contents(), render_frame_host())) {
    std::move(callback).Run("InvalidStateError");
    return;
  }
  auto* child = ResolveDirectChild(params->frame_to_replace);
  if (!child) {
    std::move(callback).Run("InvalidStateError");
    return;
  }
  ExplorerPolicy policy;
  if (params->partition == "shared" ||
      !ResolvePolicy(params->partition.empty() ? "shared" : params->partition,
                     &policy)) {
    std::move(callback).Run("SecurityError");
    return;
  }
  if (params->navigation != "emit" && params->navigation != "allow") {
    std::move(callback).Run("SecurityError");
    return;
  }
  if (params->dialogs != "emit" && params->dialogs != "suppress") {
    std::move(callback).Run("SecurityError");
    return;
  }
  policy.isolated_browsing_context = params->isolated;
  policy.navigation = params->navigation == "emit"
                          ? ExplorerNavigationPolicy::kEmit
                          : ExplorerNavigationPolicy::kAllow;
  policy.dialogs = params->dialogs == "emit" ? ExplorerDialogPolicy::kEmit
                                             : ExplorerDialogPolicy::kSuppress;
  // Exact policy equality keeps existing nested-tree grants valid.
  // A page cannot use a transfer to elevate capabilities or change partitions.
  if (policy != record->policy() ||
      (record->is_window()
           ? !record->OfferedTo(render_frame_host())
           : !record->transferred_handle() &&
                 record->owner_origin() !=
                     render_frame_host().GetLastCommittedOrigin())) {
    LOG(ERROR) << "Explorer transfer policy mismatch: navigation="
               << static_cast<int>(policy.navigation) << "/"
               << static_cast<int>(record->policy().navigation)
               << " dialogs=" << static_cast<int>(policy.dialogs) << "/"
               << static_cast<int>(record->policy().dialogs)
               << " isolated=" << policy.isolated_browsing_context << "/"
               << record->policy().isolated_browsing_context
               << " execute=" << policy.allow_execute_javascript << "/"
               << record->policy().allow_execute_javascript
               << " origins=" << policy.execute_javascript_origins.size() << "/"
               << record->policy().execute_javascript_origins.size()
               << " partition=" << policy.partition << "/"
               << record->policy().partition << " owner="
               << (record->owner_origin() ==
                   render_frame_host().GetLastCommittedOrigin());
    std::move(callback).Run("SecurityError");
    return;
  }
  const auto requested = ExplorerPartition::Parse(policy.partition);
  auto partition = render_frame_host()
                       .GetSiteInstance()
                       ->GetSecurityPrincipal()
                       .GetStoragePartitionConfig();
  if (!requested || !requested->shared()) {
    if (!requested) {
      std::move(callback).Run("SecurityError");
      return;
    }
    partition = content::StoragePartitionConfig::Create(
        render_frame_host().GetBrowserContext(), kExplorerPartitionDomain,
        requested->name, requested->in_memory());
  }
  if (partition != record->contents()
                       ->GetPrimaryMainFrame()
                       ->GetSiteInstance()
                       ->GetSecurityPrincipal()
                       .GetStoragePartitionConfig()) {
    std::move(callback).Run("SecurityError");
    return;
  }
  auto* outer = content::WebContents::FromRenderFrameHost(&render_frame_host());
  for (auto* ancestor = outer; ancestor;
       ancestor = ancestor->GetOuterWebContents()) {
    if (ancestor == record->contents()) {
      std::move(callback).Run("SecurityError");
      return;
    }
  }
  attaching_transfer_ = record->GetWeakPtr();
  record->SetState("attaching");
  client_.Bind(std::move(client));
  child->PrepareForInnerWebContentsAttach(base::BindOnce(
      &ExplorerHost::AttachTransferred, weak_factory_.GetWeakPtr(),
      params->frame_policy, std::move(callback)));
}

void ExplorerHost::AttachTransferred(blink::FramePolicy frame_policy,
                                     AttachCallback callback,
                                     content::RenderFrameHost* frame) {
  if (!attaching_transfer_ || !render_frame_host().IsActive() || !frame ||
      !frame->IsActive() || frame->GetParent() != &render_frame_host()) {
    if (attaching_transfer_) {
      attaching_transfer_->SetState("pending");
    }
    attaching_transfer_.reset();
    client_.reset();
    std::move(callback).Run("AbortError");
    return;
  }
  auto record = attaching_transfer_;
  auto owned = record->TakeContents();
  policy_ = record->policy();
  document_id_ = record->document_id();
  primary_navigation_error_.reset();
  const auto& partition = owned->GetPrimaryMainFrame()
                              ->GetSiteInstance()
                              ->GetSecurityPrincipal()
                              .GetStoragePartitionConfig();
  created_partition_domain_ = partition.partition_domain();
  created_partition_name_ = partition.partition_name();
  created_partition_in_memory_ = partition.in_memory();
  created_partition_is_default_ = partition.is_default();
  owner_renderer_process_id_ =
      render_frame_host().GetProcess()->GetDeprecatedID();
  Observe(owned.get());
  owned->SetDelegate(this);
  dialog_manager_ = std::make_unique<ExplorerDialogManager>(*this);
  auto* outer = content::WebContents::FromRenderFrameHost(&render_frame_host());
  outer->AttachInnerWebContents(std::move(owned), frame, false);
  SetExplorerOwner(WebContentsObserver::web_contents(), &render_frame_host(),
                   record->policy().allow_parent_messages, frame_policy,
                   record->policy().isolated_browsing_context);
  WebContentsObserver::web_contents()->UpdateWebContentsVisibility(
      outer->GetVisibility());
  attaching_transfer_.reset();
  WebContentsObserver::web_contents()->ResumeLoadingCreatedWebContents();
  ExplorerSurfaceHost::UpdateFrameAncestors(
      WebContentsObserver::web_contents(),
      base::BindOnce(
          [](base::WeakPtr<ExplorerHost> host,
             base::WeakPtr<ExplorerTransferRecord> record,
             AttachCallback callback, bool ready) {
            if (record) {
              record->SetState(ready ? "transferred" : "closed");
              record->Remove();
            }
            std::move(callback).Run(host && ready ? "" : "AbortError");
          },
          weak_factory_.GetWeakPtr(), record->GetWeakPtr(),
          std::move(callback)));
}

void ExplorerHost::Create(mojom::ExplorerCreateParamsPtr params,
                          mojo::PendingRemote<mojom::ExplorerClient> client,
                          CreateCallback callback) {
  if (WebContentsObserver::web_contents() || client_.is_bound() || !params) {
    std::move(callback).Run(false);
    return;
  }
  ExplorerPolicy policy;
  const auto navigation = params->navigation == "emit"
                              ? std::optional(ExplorerNavigationPolicy::kEmit)
                          : params->navigation == "allow"
                              ? std::optional(ExplorerNavigationPolicy::kAllow)
                              : std::nullopt;
  const auto dialogs = params->dialogs == "emit"
                           ? std::optional(ExplorerDialogPolicy::kEmit)
                       : params->dialogs == "suppress"
                           ? std::optional(ExplorerDialogPolicy::kSuppress)
                           : std::nullopt;
  // The empty public attribute is the only spelling for sharing the exact
  // parent partition. `shared` remains a manifest capability kind and an
  // internal policy label, avoiding an attribute that looks like a named
  // partition but is actually contextual inheritance.
  if (params->partition == "shared" ||
      !ResolvePolicy(params->partition.empty() ? "shared" : params->partition,
                     &policy) ||
      !navigation || !dialogs) {
    std::move(callback).Run(false);
    return;
  }
  policy.isolated_browsing_context = params->isolated;
  policy.navigation = *navigation;
  policy.dialogs = *dialogs;
  content::RenderFrameHost* child =
      ResolveDirectChild(params->frame_to_replace);
  GURL url(params->initial_url.empty() ? "about:blank" : params->initial_url);
  policy_ = std::move(policy);
  if (!child || !CanNavigateTo(url)) {
    policy_.reset();
    std::move(callback).Run(false);
    return;
  }

  content::BrowserContext* browser_context =
      render_frame_host().GetBrowserContext();
  const content::StoragePartitionConfig inherited_partition_config =
      render_frame_host()
          .GetSiteInstance()
          ->GetSecurityPrincipal()
          .GetStoragePartitionConfig();
  const auto requested_partition = ExplorerPartition::Parse(policy_->partition);
  CHECK(requested_partition);
  content::StoragePartitionConfig partition_config = inherited_partition_config;
  if (!requested_partition->shared()) {
    partition_config = content::StoragePartitionConfig::Create(
        browser_context, kExplorerPartitionDomain, requested_partition->name,
        requested_partition->in_memory());
  }
  // A single renderer process cannot belong to two StoragePartitions. This
  // mode is only intended for constrained smoke/debug environments, but a
  // page can still request an isolated Explorer partition there. Reject that
  // request before WebContents creation instead of reaching Chromium's
  // InSameStoragePartition CHECK and terminating the whole application. A
  // nested Explorer that explicitly selects its parent's partition remains
  // valid because the effective configs compare equal.
  if (partition_config != inherited_partition_config &&
      base::CommandLine::ForCurrentProcess()->HasSwitch(
          switches::kSingleProcess)) {
    LOG(ERROR) << "Independent Explorer partitions are unavailable in "
                  "single-process mode";
    policy_.reset();
    std::move(callback).Run(false);
    return;
  }
  // about:blank does not assign SiteInfo, so a fixed SiteInstance created for
  // it would not retain the requested StoragePartitionConfig. Seed the
  // SiteInstance from the trusted owner URL; the actual navigation below is
  // still to the requested URL and remains fixed to this partition.
  const GURL site_instance_url =
      !partition_config.is_default() && url.IsAboutBlank()
          ? render_frame_host().GetLastCommittedURL()
          : url;
  const int64_t explorer_isolation_id = NextExplorerIsolationId();
  scoped_refptr<content::SiteInstance> site_instance =
      CreateExplorerSiteInstance(browser_context, site_instance_url,
                                 partition_config, explorer_isolation_id);
  content::WebContents::CreateParams create_params(browser_context,
                                                   site_instance);
  owner_renderer_process_id_ =
      render_frame_host().GetProcess()->GetDeprecatedID();
  pending_inner_web_contents_ = content::WebContents::Create(create_params);
  pending_inner_web_contents_->GetMutableRendererPrefs()->accept_languages =
      content::WebContents::FromRenderFrameHost(&render_frame_host())
          ->GetMutableRendererPrefs()
          ->accept_languages;
  pending_inner_web_contents_->SyncRendererPrefs();
  SetExplorerIsolationId(pending_inner_web_contents_.get(),
                         explorer_isolation_id);
  const content::StoragePartitionConfig& created_partition =
      pending_inner_web_contents_->GetPrimaryMainFrame()
          ->GetSiteInstance()
          ->GetSecurityPrincipal()
          .GetStoragePartitionConfig();
  created_partition_domain_ = created_partition.partition_domain();
  created_partition_name_ = created_partition.partition_name();
  created_partition_in_memory_ = created_partition.in_memory();
  created_partition_is_default_ = created_partition.is_default();
  if (requested_partition->shared()) {
    CHECK_EQ(created_partition, partition_config);
  } else {
    CHECK_EQ(created_partition.partition_domain(), kExplorerPartitionDomain);
    CHECK_EQ(created_partition.partition_name(), requested_partition->name);
    CHECK_EQ(created_partition.in_memory(), requested_partition->in_memory());
    CHECK(!created_partition.is_default());
  }
  if (content::WebContents* outer =
          content::WebContents::FromRenderFrameHost(&render_frame_host());
      outer && !outer->GetSize().IsEmpty()) {
    pending_inner_web_contents_->Resize(gfx::Rect(outer->GetSize()));
  }
  ExplorerSurfaceData::SetExplorerPolicy(pending_inner_web_contents_.get(),
                                         *policy_);
  ExplorerSurfaceData::SetExplorerContextMenuEnabled(
      pending_inner_web_contents_.get(), params->context_menu_enabled);
  pending_frame_token_ = params->frame_to_replace;
  pending_create_callback_ = std::move(callback);
  client_.Bind(std::move(client));
  Observe(pending_inner_web_contents_.get());
  pending_inner_web_contents_->SetDelegate(this);
  dialog_manager_ = std::make_unique<ExplorerDialogManager>(*this);
  if (policy_->inherit_partition_storage) {
    partition_snapshot_pending_ = true;
    SnapshotExplorerPartition(
        browser_context->GetStoragePartition(inherited_partition_config),
        browser_context->GetStoragePartition(partition_config),
        base::BindOnce(&ExplorerHost::OnPartitionSnapshotComplete,
                       weak_factory_.GetWeakPtr(), url, params->frame_policy));
    return;
  }
  BeginAttach(url, params->frame_policy);
}

void ExplorerHost::OnPartitionSnapshotComplete(GURL initial_url,
                                               blink::FramePolicy frame_policy,
                                               bool success) {
  partition_snapshot_pending_ = false;
  if (!success || !pending_inner_web_contents_ || !pending_create_callback_) {
    Observe(nullptr);
    pending_inner_web_contents_.reset();
    pending_frame_token_.reset();
    client_.reset();
    if (pending_create_callback_) {
      std::move(pending_create_callback_).Run(false);
    }
    return;
  }
  BeginAttach(initial_url, frame_policy);
}

void ExplorerHost::AttachPrepared(
    std::unique_ptr<content::WebContents> inner_web_contents,
    CreateCallback callback,
    GURL initial_url,
    blink::FramePolicy frame_policy,
    content::RenderFrameHost* prepared_frame) {
  if (!prepared_frame || prepared_frame->GetParent() != &render_frame_host()) {
    preparing_attachment_ = false;
    Observe(nullptr);
    client_.reset();
    std::move(callback).Run(false);
    return;
  }
  auto* outer = content::WebContents::FromRenderFrameHost(&render_frame_host());
  outer->AttachInnerWebContents(std::move(inner_web_contents), prepared_frame,
                                false);
  auto* attached = WebContentsObserver::web_contents();
  // Native attachment owns the outer-delegate proxy. Expose composed ancestry
  // only after that proxy exists, and navigate only after attachment. Otherwise
  // startup postMessages can create an ordinary proxy in the same group before
  // AttachInnerWebContents attempts to create its outer-delegate proxy.
  SetExplorerOwner(attached, &render_frame_host(),
                   policy_->allow_parent_messages, frame_policy,
                   policy_->isolated_browsing_context);
  attached->UpdateWebContentsVisibility(outer->GetVisibility());
  pending_create_callback_ = std::move(callback);
  preparing_attachment_ = false;
  if (!LoadURL(initial_url)) {
    std::move(pending_create_callback_).Run(false);
  }
}

void ExplorerHost::BeginAttach(GURL initial_url,
                               blink::FramePolicy frame_policy) {
  if (!pending_inner_web_contents_ || !pending_frame_token_ ||
      !pending_create_callback_ || preparing_attachment_) {
    return;
  }
  content::RenderFrameHost* child = ResolveDirectChild(*pending_frame_token_);
  if (!child) {
    Observe(nullptr);
    pending_inner_web_contents_.reset();
    pending_frame_token_.reset();
    client_.reset();
    std::move(pending_create_callback_).Run(false);
    return;
  }
  preparing_attachment_ = true;
  pending_frame_token_.reset();
  child->PrepareForInnerWebContentsAttach(base::BindOnce(
      &ExplorerHost::AttachPrepared, weak_factory_.GetWeakPtr(),
      std::move(pending_inner_web_contents_),
      std::move(pending_create_callback_), initial_url, frame_policy));
}

void ExplorerHost::Navigate(const std::string& url, NavigateCallback callback) {
  std::move(callback).Run(LoadURL(GURL(url)));
}
content::WebContents* ExplorerHost::OwnerContents() const {
  return render_frame_host().IsActive()
             ? content::WebContents::FromRenderFrameHost(&render_frame_host())
             : nullptr;
}

void ExplorerHost::Go(int32_t offset, GoCallback callback) {
  content::WebContents* contents = WebContentsObserver::web_contents();
  const bool accepted = contents && offset != 0 &&
                        contents->GetController().CanGoToOffset(offset);
  if (accepted) {
    if (dialog_manager_) {
      dialog_manager_->CancelDialogs(contents, true);
    }
    contents->GetController().GoToOffset(offset);
  }
  std::move(callback).Run(accepted);
}

void ExplorerHost::Reload(bool bypass_cache, ReloadCallback callback) {
  content::WebContents* contents = WebContentsObserver::web_contents();
  if (contents) {
    if (dialog_manager_) {
      dialog_manager_->CancelDialogs(contents, true);
    }
    contents->GetController().Reload(bypass_cache
                                         ? content::ReloadType::BYPASSING_CACHE
                                         : content::ReloadType::NORMAL,
                                     true);
  }
  std::move(callback).Run(contents != nullptr);
}

void ExplorerHost::Stop(StopCallback callback) {
  content::WebContents* contents = WebContentsObserver::web_contents();
  if (contents) {
    contents->Stop();
  }
  std::move(callback).Run(contents != nullptr);
}

void ExplorerHost::GetNavigationState(GetNavigationStateCallback callback) {
  std::move(callback).Run(NavigationStateJson());
}

void ExplorerHost::ExecuteJavaScript(const std::string& source,
                                     bool main_world,
                                     const std::string& expected_document_id,
                                     ExecuteJavaScriptCallback callback) {
  content::WebContents* contents = WebContentsObserver::web_contents();
  if (!contents || !policy_ || !policy_->allow_execute_javascript ||
      !policy_provider_->ExecuteJavaScriptAllowed(
          render_frame_host(),
          contents->GetPrimaryMainFrame()->GetLastCommittedOrigin(),
          *policy_) ||
      source.size() > kMaxScriptBytes ||
      (main_world && !policy_->allow_execute_javascript_main_world) ||
      (!expected_document_id.empty() && expected_document_id != DocumentId())) {
    std::move(callback).Run(false, std::string(),
                            "executeJavaScript denied by Explorer policy");
    return;
  }
  if (main_world) {
    // The public main-world API intentionally CHECKs non chrome/devtools URLs.
    // Tuffite does not use the test-only bypass, so main-world execution
    // remains reserved until Content exposes an framework-safe API.
    std::move(callback).Run(false, std::string(),
                            "main-world execution is not available");
    return;
  }
  contents->GetPrimaryMainFrame()->ExecuteJavaScriptInIsolatedWorld(
      base::UTF8ToUTF16(source),
      base::BindOnce(
          [](ExecuteJavaScriptCallback callback, base::Value result) {
            std::optional<std::string> json = base::WriteJson(result);
            if (!json) {
              std::move(callback).Run(false, std::string(),
                                      "script result is not JSON serializable");
              return;
            }
            std::move(callback).Run(true, std::move(*json), std::string());
          },
          std::move(callback)),
      kExplorerIsolatedWorldId);
}

void ExplorerHost::RespondToDialog(uint64_t request_id,
                                   bool accept,
                                   const std::string& prompt_text,
                                   RespondToDialogCallback callback) {
  const bool accepted =
      offered_dialogs_.contains(request_id) &&
      RespondToDialogForTree(WebContentsObserver::web_contents(), request_id,
                             accept, prompt_text);
  std::move(callback).Run(accepted);
}

void ExplorerHost::DiscardWindow(const std::string& token,
                                 DiscardWindowCallback callback) {
  std::move(callback).Run(
      ExplorerTransferRecord::DiscardWindow(render_frame_host(), token));
}
void ExplorerHost::FinishRequest(const std::string& kind,
                                 const std::string& key,
                                 bool canceled) {
  if (client_) {
    client_->DidFinishRequest(kind, key, canceled);
  }
  if (kind == "dialog") {
    uint64_t id = 0;
    if (base::StringToUint64(key, &id)) {
      offered_dialogs_.erase(id);
      pending_dialog_bubbles_.erase(id);
    }
  } else {
    offered_windows_.erase(key);
    std::erase_if(pending_open_bubbles_, [&](const auto& item) {
      auto data = base::JSONReader::ReadDict(item.second.request_json,
                                             base::JSON_PARSE_RFC);
      return data && data->FindString("token") &&
             *data->FindString("token") == key;
    });
  }
}
void ExplorerHost::CompleteOpen(uint64_t request_id, bool propagate) {
  auto pending = pending_open_bubbles_.find(request_id);
  if (pending == pending_open_bubbles_.end()) {
    return;
  }
  PendingOpenBubble bubble = std::move(pending->second);
  pending_open_bubbles_.erase(pending);
  auto data =
      base::JSONReader::ReadDict(bubble.request_json, base::JSON_PARSE_RFC);
  auto* token = data ? data->FindString("token") : nullptr;
  if (token) {
    auto* record = ExplorerRouter::Get(render_frame_host().GetBrowserContext())
                       .FindTransfer(*token);
    if (!record || !record->pending() ||
        !record->OfferedTo(render_frame_host())) {
      return;
    }
    if (propagate) {
      offered_windows_.erase(*token);
    }
  }
  if (propagate) {
    ContinueOpenBubble(WebContentsObserver::web_contents(), bubble.event_type,
                       bubble.request_json);
  }
}

void ExplorerHost::CompleteDialog(uint64_t request_id, bool propagate) {
  auto pending = pending_dialog_bubbles_.find(request_id);
  if (pending == pending_dialog_bubbles_.end()) {
    return;
  }
  PendingDialogBubble bubble = std::move(pending->second);
  pending_dialog_bubbles_.erase(pending);
  if (propagate) {
    offered_dialogs_.erase(request_id);
    ContinueDialogBubble(WebContentsObserver::web_contents(),
                         bubble.dialog_request_id, bubble.kind, bubble.message,
                         bubble.default_value, bubble.is_reload, bubble.from);
  }
}

std::string ExplorerHost::DownloadDetail(const download::DownloadItem& item) {
  const char* state = "in-progress";
  switch (item.GetState()) {
    case download::DownloadItem::COMPLETE:
      state = "complete";
      break;
    case download::DownloadItem::CANCELLED:
      state = "cancelled";
      break;
    case download::DownloadItem::INTERRUPTED:
      state = "interrupted";
      break;
    case download::DownloadItem::IN_PROGRESS:
      break;
    case download::DownloadItem::MAX_DOWNLOAD_STATE:
      return "{}";
  }
  base::DictValue detail;
  detail.Set("id", static_cast<int>(item.GetId()));
  detail.Set("url", item.GetURL().spec());
  detail.Set("filename", item.GetFileNameToReportUser().AsUTF8Unsafe());
  detail.Set("path", item.GetTargetFilePath().AsUTF8Unsafe());
  detail.Set("mimeType", item.GetMimeType());
  detail.Set("state", state);
  detail.Set("receivedBytes", static_cast<double>(item.GetReceivedBytes()));
  detail.Set("totalBytes", static_cast<double>(item.GetTotalBytes()));
  detail.Set("error", static_cast<int>(item.GetLastReason()));
  const auto* contents = content::DownloadItemUtils::GetWebContents(&item);
  detail.Set("navigation", contents && ExplorerSurfaceData::NavigationDownload(
                                           contents) == item.GetId());
  return base::WriteJson(detail).value_or("{}");
}
void ExplorerHost::NotifyDownload(const std::string& json) {
  if (client_) {
    client_->DidDownload(json);
  }
}
void ExplorerHost::Download(const std::string& value,
                            DownloadCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  const GURL url(value);
  const bool allowed =
      contents && url.is_valid() &&
      (url.SchemeIsHTTPOrHTTPS() || url.SchemeIsBlob() ||
       url.SchemeIs(url::kDataScheme)) &&
      content::ChildProcessSecurityPolicy::GetInstance()->CanRequestURL(
          contents->GetPrimaryMainFrame()->GetProcess()->GetDeprecatedID(),
          url);
  if (allowed) {
    contents->SaveFrameWithHeaders(url, content::Referrer(), "", u"",
                                   contents->GetPrimaryMainFrame(), true);
  }
  std::move(callback).Run(allowed);
}
void ExplorerHost::GetDownloads(GetDownloadsCallback callback) {
  content::DownloadManager::DownloadVector items;
  render_frame_host()
      .GetBrowserContext()
      ->GetDownloadManager()
      ->GetAllDownloads(&items);
  base::ListValue list;
  for (const auto& item : items) {
    if (WebContentsObserver::web_contents() &&
        content::DownloadItemUtils::GetWebContents(item) ==
            WebContentsObserver::web_contents()) {
      auto detail =
          base::JSONReader::Read(DownloadDetail(*item), base::JSON_PARSE_RFC);
      if (detail) {
        list.Append(std::move(*detail));
      }
    }
  }
  std::move(callback).Run(base::WriteJson(list).value_or("[]"));
}
void ExplorerHost::CancelDownload(uint32_t id,
                                  CancelDownloadCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  auto* item = render_frame_host()
                   .GetBrowserContext()
                   ->GetDownloadManager()
                   ->GetDownload(id);
  const bool owned =
      contents && item &&
      content::DownloadItemUtils::GetWebContents(item) == contents;
  if (owned) {
    item->Cancel(true);
  }
  std::move(callback).Run(owned);
}
void ExplorerHost::ReadDownload(uint32_t id, ReadDownloadCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  auto* item = render_frame_host()
                   .GetBrowserContext()
                   ->GetDownloadManager()
                   ->GetDownload(id);
  if (!contents || !item ||
      content::DownloadItemUtils::GetWebContents(item) != contents ||
      item->GetState() != download::DownloadItem::COMPLETE) {
    std::move(callback).Run(
        "", "Download is not complete or owned by this Explorer");
    return;
  }
  base::ThreadPool::PostTaskAndReplyWithResult(
      FROM_HERE, {base::MayBlock()},
      base::BindOnce(
          [](base::FilePath path) {
            std::string bytes;
            if (!base::ReadFileToStringWithMaxSize(path, &bytes,
                                                   64 * 1024 * 1024)) {
              return std::string();
            }
            return base::Base64Encode(bytes);
          },
          item->GetFullPath()),
      base::BindOnce(
          [](ReadDownloadCallback callback, std::string data) {
            const std::string error =
                data.empty() ? "Cannot read download (64 MiB limit)" : "";
            std::move(callback).Run(data, error);
          },
          std::move(callback)));
}
void ExplorerHost::CapturePreview(CapturePreviewCallback callback) {
  CapturePreviewWhenReady(std::move(callback), 20);
}

void ExplorerHost::CapturePreviewWhenReady(CapturePreviewCallback callback,
                                           int attempts) {
  auto* contents = WebContentsObserver::web_contents();
  auto* view = contents ? contents->GetRenderWidgetHostView() : nullptr;
  if (contents && (!view || !view->IsSurfaceAvailableForCopy()) &&
      attempts > 0) {
    base::SingleThreadTaskRunner::GetCurrentDefault()->PostDelayedTask(
        FROM_HERE,
        base::BindOnce(&ExplorerHost::CapturePreviewWhenReady,
                       weak_factory_.GetWeakPtr(), std::move(callback),
                       attempts - 1),
        base::Milliseconds(50));
    return;
  }
  if (!view || !view->IsSurfaceAvailableForCopy()) {
    std::move(callback).Run("");
    return;
  }
  const auto bounds = view->GetViewBounds().size();
  const double scale =
      std::min(1.0, std::min(1280.0 / std::max(1, bounds.width()),
                             960.0 / std::max(1, bounds.height())));
  const gfx::Size output(
      std::max(1, static_cast<int>(bounds.width() * scale)),
      std::max(1, static_cast<int>(bounds.height() * scale)));
  view->CopyFromSurface(
      gfx::Rect(), output, base::Seconds(1),
      base::BindOnce(
          [](CapturePreviewCallback callback,
             const content::CopyFromSurfaceResult& result) {
            std::string url;
            if (result.has_value()) {
              auto png =
                  gfx::PNGCodec::EncodeBGRASkBitmap(result->bitmap, false);
              if (png) {
                url = "data:image/png;base64," + base::Base64Encode(*png);
              }
            }
            std::move(callback).Run(url);
          },
          std::move(callback)));
}

void ExplorerHost::SetFramePolicy(const blink::FramePolicy& policy) {
  if (auto* contents = WebContentsObserver::web_contents()) {
    SetExplorerFramePolicy(contents, policy);
    ExplorerSurfaceHost::UpdateFrameAncestors(contents, base::DoNothing());
  }
}
void ExplorerHost::SetContextMenuEnabled(bool enabled) {
  if (content::WebContents* contents = WebContentsObserver::web_contents()) {
    ExplorerSurfaceData::SetExplorerContextMenuEnabled(contents, enabled);
  }
}

bool ExplorerHost::LoadURL(const GURL& url) {
  if (!WebContentsObserver::web_contents() || !CanNavigateTo(url)) {
    if (client_) {
      client_->DidFailLoad(net::ERR_INVALID_URL,
                           net::ErrorToString(net::ERR_INVALID_URL),
                           NavigationStateJson());
    }
    return false;
  }
  content::NavigationController::LoadURLParams params(url);
  // An application navigation must dismiss a held guest dialog before asking
  // the renderer to leave its document; otherwise that renderer is blocked.
  if (dialog_manager_) {
    dialog_manager_->CancelDialogs(WebContentsObserver::web_contents(), true);
  }
  params.transition_type = ui::PAGE_TRANSITION_LINK;
  WebContentsObserver::web_contents()->GetController().LoadURLWithParams(
      params);
  return true;
}

std::string ExplorerHost::DocumentId() const {
  return document_id_;
}

std::string ExplorerHost::NavigationStateJson() const {
  content::WebContents* contents = WebContentsObserver::web_contents();
  if (!contents) {
    return "{}";
  }
  base::DictValue state;
  const auto resource_id = ExplorerSurfaceData::NavigationDownload(contents);
  auto* resource =
      resource_id
          ? contents->GetBrowserContext()->GetDownloadManager()->GetDownload(
                *resource_id)
          : nullptr;
  state.Set("url", resource ? resource->GetURL().spec()
                            : contents->GetLastCommittedURL().spec());
  state.Set("title", resource
                         ? resource->GetFileNameToReportUser().AsUTF8Unsafe()
                         : base::UTF16ToUTF8(contents->GetTitle()));
  state.Set("documentId", DocumentId());
  state.Set("canGoBack", contents->GetController().CanGoBack());
  state.Set("canGoForward", contents->GetController().CanGoForward());
  state.Set("isLoading", contents->IsLoading());
  state.Set("zoomFactor", ExplorerSurfaceData::GetZoomFactor(contents));
  state.Set("audioMuted", contents->IsAudioMuted());
  state.Set("audible", contents->IsCurrentlyAudible());
  state.Set("crashed", contents->IsCrashed());
  state.Set("fullscreen", contents->IsFullscreen());
  state.Set("pictureInPicture",
            contents->HasPictureInPictureDocument() ? "document"
            : contents->HasPictureInPictureVideo()  ? "video"
                                                    : "none");
  state.Set("favicons", ExplorerSurfaceData::GetFavicons(contents));
  const int renderer_process_id =
      contents->GetPrimaryMainFrame()->GetProcess()->GetDeprecatedID();
  state.Set(
      "rendererProcess",
      base::DictValue()
          .Set("id", renderer_process_id)
          .Set("ownerId", owner_renderer_process_id_)
          .Set("isolated", renderer_process_id != owner_renderer_process_id_));
  const content::StoragePartitionConfig& partition =
      contents->GetPrimaryMainFrame()
          ->GetSiteInstance()
          ->GetSecurityPrincipal()
          .GetStoragePartitionConfig();
  const auto requested_partition =
      policy_ ? ExplorerPartition::Parse(policy_->partition) : std::nullopt;
  const bool shares_parent_partition =
      requested_partition && requested_partition->shared();
  const std::string partition_mode =
      requested_partition ? std::string(requested_partition->mode_name())
                          : "shared";
  state.Set("storagePartition",
            base::DictValue()
                .Set("domain", partition.partition_domain())
                .Set("name", partition.partition_name())
                .Set("inMemory", partition.in_memory())
                .Set("isDefault", partition.is_default())
                .Set("mode", partition_mode)
                .Set("shared", shares_parent_partition)
                .Set("inherited", policy_ && policy_->inherit_partition_storage)
                // The public element value is empty when the parent partition
                // is shared. "shared" is a manifest capability kind and an
                // internal policy label, not an element partition value.
                .Set("requested", shares_parent_partition || !policy_
                                      ? std::string()
                                      : policy_->partition)
                .Set("createdDomain", created_partition_domain_)
                .Set("createdName", created_partition_name_)
                .Set("createdInMemory", created_partition_in_memory_)
                .Set("createdIsDefault", created_partition_is_default_));
  return base::WriteJson(state).value_or("{}");
}

void ExplorerHost::NotifyRequestedOpen(const char* event_type,
                                       const GURL& url,
                                       WindowOpenDisposition disposition,
                                       bool user_gesture) {
  if (!client_ || !CanNavigateTo(url)) {
    return;
  }
  base::DictValue request;
  request.Set("url", url.spec());
  request.Set("disposition", ExplorerDispositionName(disposition));
  request.Set("userGesture", user_gesture);
  request.Set("from", ExplorerSurfaceHost::RefForWebContents(
                          WebContentsObserver::web_contents()));
  EmitOpenBubble(event_type, base::WriteJson(request).value_or("{}"));
}

void ExplorerHost::EmitOpenBubble(const std::string& event_type,
                                  const std::string& request_json) {
  if (!client_) {
    ContinueOpenBubble(WebContentsObserver::web_contents(), event_type,
                       request_json);
    return;
  }
  if (pending_open_bubbles_.size() >= kMaxPendingBubbleRequests) {
    // Never evict an in-flight request: its renderer callback still owns the
    // continuation. Skip this saturated boundary so the new request can keep
    // bubbling without orphaning an older completion.
    ContinueOpenBubble(WebContentsObserver::web_contents(), event_type,
                       request_json);
    return;
  }
  if (event_type == "newwindow") {
    auto data = base::JSONReader::ReadDict(request_json, base::JSON_PARSE_RFC);
    const auto* token = data ? data->FindString("token") : nullptr;
    auto* record =
        token ? ExplorerRouter::Get(render_frame_host().GetBrowserContext())
                    .FindTransfer(*token)
              : nullptr;
    if (!record || !record->pending()) {
      return;
    }
    record->OfferWindow(render_frame_host());
    offered_windows_.insert(*token);
  }
  const uint64_t request_id = NextExplorerRequestId();
  pending_open_bubbles_.insert_or_assign(
      request_id, PendingOpenBubble{event_type, request_json});
  client_->DidRequestOpen(request_id, event_type, request_json);
}

void ExplorerHost::EmitDialogBubble(uint64_t request_id,
                                    const std::string& kind,
                                    const std::string& message,
                                    const std::string& default_value,
                                    bool is_reload,
                                    const std::string& from) {
  if (!client_) {
    ContinueDialogBubble(WebContentsObserver::web_contents(), request_id, kind,
                         message, default_value, is_reload, from);
    return;
  }
  if (pending_dialog_bubbles_.size() >= kMaxPendingBubbleRequests) {
    // A dialog owns a native completion callback. Skipping this saturated
    // boundary preserves both that callback and the upward fallback path.
    ContinueDialogBubble(WebContentsObserver::web_contents(), request_id, kind,
                         message, default_value, is_reload, from);
    return;
  }
  offered_dialogs_.insert(request_id);
  pending_dialog_bubbles_.insert_or_assign(
      request_id, PendingDialogBubble{request_id, kind, message, default_value,
                                      is_reload, from});
  client_->DidRequestDialog(request_id, kind, message, default_value, is_reload,
                            from);
}

void ExplorerHost::NotifyLoadComplete() {
  if (!client_) {
    return;
  }
  if (primary_navigation_error_) {
    const int error_code = *primary_navigation_error_;
    client_->DidFailLoad(error_code, net::ErrorToString(error_code),
                         NavigationStateJson());
  } else {
    client_->DidStopLoading(NavigationStateJson());
  }
}

content::WebContents* ExplorerHost::OpenURLFromTab(
    content::WebContents* source,
    const content::OpenURLParams& params,
    base::OnceCallback<void(content::NavigationHandle&)>
        navigation_handle_callback) {
  if (params.disposition != WindowOpenDisposition::CURRENT_TAB) {
    if (!source || !policy_ || !CanNavigateTo(params.url)) {
      return nullptr;
    }
    auto* instance = source->GetPrimaryMainFrame()->GetSiteInstance();
    content::WebContents::CreateParams create(source->GetBrowserContext(),
                                              instance);
    auto owned = content::WebContents::Create(create);
    auto* created = owned.get();
    WebContentsCreated(source, {}, "", params.url, created);
    bool blocked = false;
    auto* result = AddNewContents(
        source, std::move(owned), params.url, params.disposition,
        blink::mojom::WindowFeatures(), params.user_gesture, &blocked);
    if (result) {
      auto navigation = result->GetController().LoadURLWithParams(
          content::NavigationController::LoadURLParams(params));
      if (navigation_handle_callback && navigation) {
        std::move(navigation_handle_callback).Run(*navigation);
      }
    }
    return result;
  }
  if ((policy_ && policy_->navigation == ExplorerNavigationPolicy::kEmit)) {
    NotifyRequestedOpen(ExplorerOpenEventType(params.disposition, false).data(),
                        params.url, params.disposition, params.user_gesture);
    return nullptr;
  }
  if (source != WebContentsObserver::web_contents() ||
      !CanNavigateTo(params.url)) {
    return nullptr;
  }
  base::WeakPtr<content::NavigationHandle> navigation_handle =
      source->GetController().LoadURLWithParams(
          content::NavigationController::LoadURLParams(params));
  if (navigation_handle_callback && navigation_handle) {
    std::move(navigation_handle_callback).Run(*navigation_handle);
  }
  return source;
}

bool ExplorerHost::IsWebContentsCreationOverridden(
    content::RenderFrameHost* opener,
    content::SiteInstance* source_site_instance,
    content::mojom::WindowContainerType window_container_type,
    const GURL& opener_url,
    const std::string& frame_name,
    const GURL& target_url) {
  return !CanNavigateTo(target_url);
}

content::WebContents* ExplorerHost::CreateCustomWebContents(
    content::RenderFrameHost* opener,
    content::SiteInstance* source_site_instance,
    bool is_new_browsing_instance,
    const GURL& opener_url,
    const std::string& frame_name,
    const GURL& target_url,
    WindowOpenDisposition disposition,
    const blink::mojom::WindowFeatures& window_features,
    const content::StoragePartitionConfig& partition_config,
    content::SessionStorageNamespace* session_storage_namespace) {
  return nullptr;
}

content::WebContents* ExplorerHost::AddNewContents(
    content::WebContents* source,
    std::unique_ptr<content::WebContents> new_contents,
    const GURL& target_url,
    WindowOpenDisposition disposition,
    const blink::mojom::WindowFeatures& window_features,
    bool user_gesture,
    bool* was_blocked) {
  if (disposition == WindowOpenDisposition::NEW_PICTURE_IN_PICTURE) {
    new_contents->ResumeLoadingCreatedWebContents();
    auto* owner =
        content::WebContents::FromRenderFrameHost(&render_frame_host());
    if (owner && owner->GetDelegate() && owner->GetDelegate() != this) {
      return owner->GetDelegate()->AddNewContents(
          source, std::move(new_contents), target_url, disposition,
          window_features, user_gesture, was_blocked);
    }
  }
  auto* source_frame = ExplorerSurfaceData::GetWindowSource(new_contents.get());
  if (!source_frame || !source_frame->IsActive() || !policy_ ||
      !CanNavigateTo(target_url) ||
      ExplorerRouter::Get(source).transfer_count() >=
          kMaxPendingBubbleRequests) {
    if (was_blocked) {
      *was_blocked = true;
    }
    return nullptr;
  }
  auto* contents = new_contents.get();
  contents->WasHidden();
  auto& record =
      ExplorerTransferStore::GetOrCreateForCurrentDocument(source_frame)
          ->Add(std::move(new_contents), *policy_,
                base::UnguessableToken::Create().ToString(), {});
  record.OfferWindow(render_frame_host());
  base::DictValue request;
  request.Set("token", record.token());
  request.Set("url", target_url.spec());
  request.Set("disposition", ExplorerDispositionName(disposition));
  request.Set("userGesture", user_gesture);
  request.Set("from", ExplorerSurfaceHost::RefForWebContents(source));
  EmitOpenBubble("newwindow", base::WriteJson(request).value_or("{}"));
  if (was_blocked) {
    *was_blocked = false;
  }
  return contents;
}

bool ExplorerHost::ShouldResumeRequestsForCreatedWindow() {
  // Complete Chromium's renderer initialization before offering the hidden
  // page. Otherwise an immediate discard can strand window.open's synchronous
  // creation handshake, especially when reusing a related renderer.
  return true;
}
void ExplorerHost::WebContentsCreated(
    content::WebContents* source,
    const content::GlobalRenderFrameHostId& opener_id,
    const std::string&,
    const GURL& target_url,
    content::WebContents* created) {
  if (policy_) {
    ExplorerSurfaceData::SetExplorerPolicy(created, *policy_);
  }
  auto* opener = content::RenderFrameHost::FromID(opener_id);
  ExplorerSurfaceData::SetWindowSource(
      created, opener ? opener : source->GetPrimaryMainFrame());
  if (!target_url.IsAboutBlank()) {
    ExplorerSurfaceData::MarkInitialWindowNavigation(created);
  }
  if (auto isolation = GetExplorerIsolationId(source)) {
    SetExplorerIsolationId(created, *isolation);
  }
}
content::JavaScriptDialogManager* ExplorerHost::GetJavaScriptDialogManager(
    content::WebContents* source) {
  return dialog_manager_.get();
}

void ExplorerHost::RunFileChooser(
    content::RenderFrameHost* frame,
    scoped_refptr<content::FileSelectListener> listener,
    const blink::mojom::FileChooserParams& params) {
  auto* owner = content::WebContents::FromRenderFrameHost(&render_frame_host());
  if (!owner || !owner->GetDelegate() || owner->GetDelegate() == this) {
    listener->FileSelectionCanceled();
    return;
  }
  owner->GetDelegate()->RunFileChooser(frame, std::move(listener), params);
}

void ExplorerHost::EnumerateDirectory(
    content::WebContents* contents,
    scoped_refptr<content::FileSelectListener> listener,
    const base::FilePath& path) {
  auto* owner = content::WebContents::FromRenderFrameHost(&render_frame_host());
  if (!owner || !owner->GetDelegate() || owner->GetDelegate() == this) {
    listener->FileSelectionCanceled();
    return;
  }
  owner->GetDelegate()->EnumerateDirectory(contents, std::move(listener), path);
}

void ExplorerHost::EnterFullscreenModeForTab(
    content::RenderFrameHost* requesting_frame,
    const blink::mojom::FullscreenOptions& options) {
  content::WebContents* owner =
      content::WebContents::FromRenderFrameHost(&render_frame_host());
  if (owner && owner->GetDelegate() && owner->GetDelegate() != this) {
    owner->GetDelegate()->EnterFullscreenModeForTab(requesting_frame, options);
  }
}

void ExplorerHost::ExitFullscreenModeForTab(
    content::WebContents* web_contents) {
  content::WebContents* owner =
      content::WebContents::FromRenderFrameHost(&render_frame_host());
  if (owner && owner->GetDelegate() && owner->GetDelegate() != this) {
    owner->GetDelegate()->ExitFullscreenModeForTab(web_contents);
  }
}

bool ExplorerHost::IsFullscreenForTabOrPending(
    const content::WebContents* web_contents) {
  content::WebContents* owner =
      content::WebContents::FromRenderFrameHost(&render_frame_host());
  return owner && owner->GetDelegate() && owner->GetDelegate() != this &&
         owner->GetDelegate()->IsFullscreenForTabOrPending(web_contents);
}

content::PictureInPictureResult ExplorerHost::EnterPictureInPicture(
    content::WebContents* web_contents) {
  content::WebContents* owner =
      content::WebContents::FromRenderFrameHost(&render_frame_host());
  if (owner && owner->GetDelegate()) {
    return owner->GetDelegate()->EnterPictureInPicture(web_contents);
  }
  return content::PictureInPictureResult::kNotSupported;
}

void ExplorerHost::ExitPictureInPicture() {
  content::WebContents* owner =
      content::WebContents::FromRenderFrameHost(&render_frame_host());
  if (owner && owner->GetDelegate()) {
    owner->GetDelegate()->ExitPictureInPicture();
  }
}

void ExplorerHost::RequestDialog(
    const std::string& kind,
    const std::u16string& message,
    const std::u16string& default_value,
    bool is_reload,
    content::JavaScriptDialogManager::DialogClosedCallback callback,
    bool* did_suppress_message) {
  if (!client_ || !policy_ || !policy_->allow_dialogs ||
      policy_->dialogs == ExplorerDialogPolicy::kSuppress) {
    if (did_suppress_message) {
      *did_suppress_message = true;
    } else {
      std::move(callback).Run(false, std::u16string());
    }
    return;
  }
  if (did_suppress_message) {
    *did_suppress_message = false;
  }
  const uint64_t request_id = dialog_manager_->Begin(std::move(callback));
  EmitDialogBubble(request_id, kind, base::UTF16ToUTF8(message),
                   base::UTF16ToUTF8(default_value), is_reload,
                   ExplorerSurfaceHost::RefForWebContents(
                       WebContentsObserver::web_contents()));
}

void ExplorerHost::DidStartLoading() {
  primary_navigation_error_.reset();
  if (auto* contents = WebContentsObserver::web_contents()) {
    ExplorerSurfaceData::SetNavigationDownload(contents, std::nullopt);
  }
  if (client_) {
    client_->DidStartLoading(NavigationStateJson());
  }
}

void ExplorerHost::DidStopLoading() {
  if (pending_inner_web_contents_ || preparing_attachment_) {
    return;
  }
  if (pending_create_callback_) {
    std::move(pending_create_callback_).Run(true);
  }
  NotifyLoadComplete();
}

void ExplorerHost::DidFinishNavigation(
    content::NavigationHandle* navigation_handle) {
  if (!navigation_handle || !navigation_handle->IsInPrimaryMainFrame() ||
      !navigation_handle->HasCommitted()) {
    return;
  }
  const int error_code = navigation_handle->GetNetErrorCode();
  primary_navigation_error_ = navigation_handle->IsErrorPage() &&
                                      error_code != net::OK &&
                                      error_code != net::ERR_ABORTED
                                  ? std::optional<int>(error_code)
                                  : std::nullopt;
  if (!navigation_handle->IsSameDocument()) {
    active_find_request_id_ = -1;
    find_text_.clear();
    document_id_ = base::UnguessableToken::Create().ToString();
    if (dialog_manager_) {
      dialog_manager_->CancelDialogs(nullptr, true);
    }
  }
  if (policy_ && policy_->partition != "shared") {
    const content::StoragePartitionConfig& committed_partition =
        navigation_handle->GetRenderFrameHost()
            ->GetSiteInstance()
            ->GetSecurityPrincipal()
            .GetStoragePartitionConfig();
    CHECK_EQ(committed_partition.partition_domain(), created_partition_domain_);
    CHECK_EQ(committed_partition.partition_name(), created_partition_name_);
    CHECK_EQ(committed_partition.in_memory(), created_partition_in_memory_);
    CHECK_EQ(committed_partition.is_default(), created_partition_is_default_);
  }
  if (client_) {
    client_->DidCommitNavigation(NavigationStateJson());
  }
}

void ExplorerHost::TitleWasSet(content::NavigationEntry* entry) {
  if (client_) {
    client_->DidChangePageState("titlechange", NavigationStateJson());
  }
}

// static
bool ExplorerHost::MaybeEmitNavigation(content::WebContents* contents,
                                       const GURL& url,
                                       bool user_gesture) {
  if (ExplorerSurfaceData::IsInitialWindowNavigation(contents)) {
    return false;
  }
  ExplorerHost* host =
      ExplorerRouter::Get(contents).FindHostForContents(contents);
  // Detached pages have no UI to handle emitted navigation requests.
  // Preserve their emit policy instead of falling through to navigation.
  if (!host) {
    auto* transfer =
        ExplorerRouter::Get(contents).FindTransferForContents(contents);
    return transfer &&
           transfer->policy().navigation == ExplorerNavigationPolicy::kEmit;
  }
  if (!host->policy_ ||
      host->policy_->navigation != ExplorerNavigationPolicy::kEmit) {
    return false;
  }
  host->NotifyRequestedOpen("navigation", url,
                            WindowOpenDisposition::CURRENT_TAB, user_gesture);
  return true;
}

// static
void ExplorerHost::ContinueOpenBubble(content::WebContents* source,
                                      const std::string& event_type,
                                      const std::string& request_json) {
  content::WebContents* owner =
      source ? source->GetOuterWebContents() : nullptr;
  if (!owner) {
    return;
  }
  ExplorerHost* host = ExplorerRouter::Get(owner).FindHostForContents(owner);
  if (host) {
    host->EmitOpenBubble(event_type, request_json);
    return;
  }
  if (!ExplorerSurfaceHost::DeliverOpenToWebContents(owner, event_type,
                                                     request_json)) {
    auto data = base::JSONReader::ReadDict(request_json, base::JSON_PARSE_RFC);
    auto* token = data ? data->FindString("token") : nullptr;
    auto* record =
        token ? ExplorerRouter::Get(source).FindTransfer(*token) : nullptr;
    if (record && record->pending()) {
      record->Remove();
    }
  }
}

// static
void ExplorerHost::ContinueDialogBubble(content::WebContents* source,
                                        uint64_t request_id,
                                        const std::string& kind,
                                        const std::string& message,
                                        const std::string& default_value,
                                        bool is_reload,
                                        const std::string& from) {
  content::WebContents* owner =
      source ? source->GetOuterWebContents() : nullptr;
  if (!owner) {
    RespondToDialogForTree(source, request_id, false, std::string());
    return;
  }
  ExplorerHost* host = ExplorerRouter::Get(owner).FindHostForContents(owner);
  if (host) {
    host->EmitDialogBubble(request_id, kind, message, default_value, is_reload,
                           from);
    return;
  }
  if (!ExplorerSurfaceHost::DeliverDialogToWebContents(
          owner, request_id, kind, message, default_value, is_reload, from)) {
    // A missing/saturated top-surface listener must not leave the page's modal
    // callback unresolved indefinitely. The safe browser fallback is dismiss.
    RespondToDialogForTree(source, request_id, false, std::string());
  }
}

// static
bool ExplorerHost::RespondToDialogForTree(content::WebContents* requester,
                                          uint64_t request_id,
                                          bool accept,
                                          const std::string& prompt_text,
                                          bool canceled) {
  if (!requester) {
    return false;
  }
  content::WebContents* root = requester->GetOutermostWebContents();
  for (ExplorerHost* host : ExplorerRouter::Get(requester).hosts()) {
    content::WebContents* contents = host->WebContentsObserver::web_contents();
    if (contents && contents->GetOutermostWebContents() == root &&
        host->dialog_manager_ &&
        host->dialog_manager_->Respond(
            request_id, accept, base::UTF8ToUTF16(prompt_text), canceled)) {
      return true;
    }
  }
  return false;
}

}  // namespace tuffite

namespace tuffite {}  // namespace tuffite
