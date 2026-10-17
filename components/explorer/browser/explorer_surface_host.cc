// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_surface_host.h"

#include <algorithm>
#include <deque>
#include <map>
#include <vector>

#include "base/barrier_callback.h"
#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/no_destructor.h"
#include "base/strings/string_number_conversions.h"
#include "base/task/single_thread_task_runner.h"
#include "base/time/time.h"
#include "base/unguessable_token.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/global_routing_id.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "mojo/public/cpp/bindings/callback_helpers.h"
#include "third_party/blink/public/common/messaging/transferable_message.h"
#include "tuffite/components/explorer/browser/content/explorer_isolation.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/browser/explorer_policy_provider.h"
#include "tuffite/components/explorer/browser/explorer_router.h"
#include "tuffite/components/explorer/browser/explorer_surface_data.h"
#include "tuffite/components/explorer/browser/explorer_transfer.h"
#include "url/gurl.h"
#include "url/origin.h"

namespace tuffite {
namespace {

constexpr size_t kMaxPendingSurfaceEvents = 256;

content::WebContents* ContentsFor(content::RenderFrameHost& frame) {
  return content::WebContents::FromRenderFrameHost(&frame);
}

}  // namespace

// static
void ExplorerSurfaceHost::Bind(
    content::RenderFrameHost* frame,
    mojo::PendingReceiver<mojom::ExplorerSurfaceHost> receiver,
    ExplorerPolicyProvider* policy) {
  if (!frame || !policy || !policy->IsSurfaceAllowed(*frame)) {
    return;
  }
  new ExplorerSurfaceHost(*frame, std::move(receiver), *policy);
}

ExplorerSurfaceHost::ExplorerSurfaceHost(
    content::RenderFrameHost& frame,
    mojo::PendingReceiver<mojom::ExplorerSurfaceHost> receiver,
    ExplorerPolicyProvider& policy)
    : DocumentService(frame, std::move(receiver)),
      policy_provider_(policy),
      surface_ref_("surface:" + base::UnguessableToken::Create().ToString()) {
  ExplorerRouter& router = ExplorerRouter::Get(frame.GetBrowserContext());
  router.AddSurface(*this);
}

ExplorerSurfaceHost::~ExplorerSurfaceHost() {
  auto windows = std::move(active_windows_);
  for (const auto& token : windows) {
    ExplorerTransferRecord::DiscardWindow(render_frame_host(), token, true);
  }
  content::WebContents* contents = ContentsFor(render_frame_host());
  std::vector<uint64_t> unresolved_dialogs(active_dialogs_.begin(),
                                           active_dialogs_.end());
  for (const PendingDialog& dialog : pending_dialogs_) {
    unresolved_dialogs.push_back(dialog.request_id);
  }
  active_dialogs_.clear();
  pending_dialogs_.clear();
  ExplorerRouter::Get(render_frame_host().GetBrowserContext())
      .RemoveSurface(*this);
  for (uint64_t request_id : unresolved_dialogs) {
    ExplorerHost::RespondToDialogForTree(contents, request_id, false,
                                         std::string(), true);
  }
}

bool ExplorerSurfaceHost::IsCurrentPrimaryDocument() const {
  content::WebContents* contents = ContentsFor(render_frame_host());
  return contents && contents->GetPrimaryMainFrame() == &render_frame_host();
}

void ExplorerSurfaceHost::GetFrameAncestors(
    GetFrameAncestorsCallback callback) {
  std::move(callback).Run(ExplorerFrameAncestors(render_frame_host()),
                          ExplorerFrameOwnerPolicy(render_frame_host()));
}
void ExplorerSurfaceHost::UpdateFrameAncestors(
    content::WebContents* contents,
    base::OnceCallback<void(bool)> callback,
    unsigned attempt) {
  std::vector<content::WebContents*> tree{contents};
  for (size_t i = 0; i < tree.size(); ++i) {
    for (auto* child : tree[i]->GetInnerWebContents()) {
      tree.push_back(child);
    }
  }
  std::vector<base::WeakPtr<ExplorerSurfaceHost>> targets;
  for (auto* surface : ExplorerRouter::Get(contents).surfaces()) {
    if (surface->IsCurrentPrimaryDocument() && surface->client_ &&
        std::find(tree.begin(), tree.end(),
                  ContentsFor(surface->render_frame_host())) != tree.end()) {
      targets.push_back(surface->weak_factory_.GetWeakPtr());
    }
  }
  auto barrier = base::BarrierCallback<bool>(
      targets.size(),
      base::BindOnce(
          [](base::WeakPtr<content::WebContents> contents,
             base::OnceCallback<void(bool)> callback, unsigned attempt,
             std::vector<bool> results) {
            const bool ready = std::all_of(results.begin(), results.end(),
                                           [](bool value) { return value; });
            if (!ready && contents && attempt < 8) {
              base::SingleThreadTaskRunner::GetCurrentDefault()
                  ->PostDelayedTask(
                      FROM_HERE,
                      base::BindOnce(
                          [](base::WeakPtr<content::WebContents> contents,
                             base::OnceCallback<void(bool)> callback,
                             unsigned attempt) {
                            if (!contents) {
                              std::move(callback).Run(false);
                              return;
                            }
                            UpdateFrameAncestors(contents.get(),
                                                 std::move(callback), attempt);
                          },
                          contents, std::move(callback), attempt + 1),
                      base::Milliseconds(5));
              return;
            }
            std::move(callback).Run(ready);
          },
          contents->GetWeakPtr(), std::move(callback), attempt));
  for (auto surface : targets) {
    if (!surface || !surface->client_) {
      barrier.Run(false);
      continue;
    }
    surface->client_->DidChangeAncestors(
        ExplorerFrameAncestors(surface->render_frame_host()),
        ExplorerFrameOwnerPolicy(surface->render_frame_host()),
        mojo::WrapCallbackWithDefaultInvokeIfNotRun(
            base::BindOnce(
                [](base::WeakPtr<content::WebContents> contents,
                   content::WeakDocumentPtr document,
                   base::RepeatingCallback<void(bool)> barrier, bool ready) {
                  if (!ready && contents) {
                    auto* frame = document.AsRenderFrameHostIfValid();
                    // A popup may commit its requested document while being
                    // attached. The replacement initializes native ancestry via
                    // CreateView/context metadata; closing the old client is
                    // not an attachment failure.
                    ready = !frame || !frame->IsActive() ||
                            contents->GetPrimaryMainFrame() != frame;
                  }
                  barrier.Run(ready);
                },
                ContentsFor(surface->render_frame_host())->GetWeakPtr(),
                surface->render_frame_host().GetWeakDocumentPtr(), barrier),
            false));
  }
}
void ExplorerSurfaceHost::BindClient(
    mojo::PendingRemote<mojom::ExplorerSurfaceClient> client) {
  if (IsCurrentPrimaryDocument()) {
    if (client_.is_bound()) {
      return;
    }
    client_.Bind(std::move(client));
    client_.set_disconnect_handler(
        base::BindOnce(&ExplorerSurfaceHost::OnClientDisconnected,
                       weak_factory_.GetWeakPtr()));
    FlushPendingRequests();
  }
}

void ExplorerSurfaceHost::PostWindowMessage(int64_t id,
                                            blink::TransferableMessage message,
                                            const std::string& target_origin) {
  if (!render_frame_host().IsActive() ||
      !policy_provider_->IsOwnerAllowed(render_frame_host())) {
    return;
  }
  const auto source = render_frame_host().GetLastCommittedOrigin();
  std::optional<url::Origin> origin;
  if (target_origin != "*") {
    GURL url(target_origin);
    if (!url.is_valid()) {
      return;
    }
    origin = url::Origin::Create(url);
  }
  // Use browser-authenticated origins, never renderer-supplied identity.
  message.sender_origin = source;
  policy_provider_->ResolveMessageWindow(
      render_frame_host(), id,
      base::BindOnce(
          [](url::Origin source, std::optional<url::Origin> origin,
             blink::TransferableMessage message,
             content::RenderFrameHost* target) {
            if (!target || !target->IsActive() ||
                (origin &&
                 !origin->IsSameOriginWith(target->GetLastCommittedOrigin()))) {
              return;
            }
            DeliverExplorerWindowMessage(*target, source,
                                         origin ? &*origin : nullptr,
                                         std::move(message));
          },
          source, std::move(origin), std::move(message)));
}
void ExplorerSurfaceHost::RespondToDialog(uint64_t request_id,
                                          bool accept,
                                          const std::string& prompt_text,
                                          RespondToDialogCallback callback) {
  std::move(callback).Run(
      active_dialogs_.contains(request_id) &&
      ExplorerHost::RespondToDialogForTree(ContentsFor(render_frame_host()),
                                           request_id, accept, prompt_text));
}

void ExplorerSurfaceHost::DiscardWindow(const std::string& token,
                                        DiscardWindowCallback callback) {
  std::move(callback).Run(
      ExplorerTransferRecord::DiscardWindow(render_frame_host(), token));
}
void ExplorerSurfaceHost::CompleteDialog(uint64_t id, bool propagate) {
  if (propagate && active_dialogs_.contains(id)) {
    ExplorerHost::RespondToDialogForTree(ContentsFor(render_frame_host()), id,
                                         false, "");
  }
}
void ExplorerSurfaceHost::CompleteOpen(const std::string& token,
                                       bool propagate) {
  if (propagate) {
    ExplorerTransferRecord::DiscardWindow(render_frame_host(), token);
  }
}
void ExplorerSurfaceHost::FlushPendingRequests() {
  while (client_ && !pending_opens_.empty() && IsCurrentPrimaryDocument()) {
    PendingOpen request = std::move(pending_opens_.front());
    pending_opens_.pop_front();
    client_->DidReceiveOpen(request.event_type, request.request_json);
  }
  while (client_ && !pending_dialogs_.empty() && IsCurrentPrimaryDocument()) {
    PendingDialog request = std::move(pending_dialogs_.front());
    pending_dialogs_.pop_front();
    active_dialogs_.insert(request.request_id);
    client_->DidReceiveDialog(request.request_id, request.type, request.message,
                              request.default_value, request.is_reload,
                              request.from);
  }
}

void ExplorerSurfaceHost::OnClientDisconnected() {
  client_.reset();
  auto windows = std::move(active_windows_);
  for (const auto& token : windows) {
    ExplorerTransferRecord::DiscardWindow(render_frame_host(), token, true);
  }
  DismissActiveDialogs();
}

void ExplorerSurfaceHost::DismissActiveDialogs() {
  content::WebContents* contents = ContentsFor(render_frame_host());
  std::vector<uint64_t> request_ids(active_dialogs_.begin(),
                                    active_dialogs_.end());
  active_dialogs_.clear();
  for (uint64_t request_id : request_ids) {
    ExplorerHost::RespondToDialogForTree(contents, request_id, false,
                                         std::string(), true);
  }
}

// static
std::string ExplorerSurfaceHost::RefForWebContents(
    content::WebContents* contents) {
  if (!contents) {
    return std::string();
  }
  ExplorerSurfaceHost* surface =
      ExplorerRouter::Get(contents).FindSurfaceForContents(contents);
  return surface ? surface->surface_ref_ : std::string();
}

// static
bool ExplorerSurfaceHost::DeliverOpenToWebContents(
    content::WebContents* destination,
    const std::string& event_type,
    const std::string& request_json) {
  if (!destination) {
    return false;
  }
  ExplorerSurfaceHost* surface =
      ExplorerRouter::Get(destination).FindSurfaceForContents(destination);
  if (!surface || !surface->IsCurrentPrimaryDocument()) {
    return false;
  }
  if (event_type == "newwindow") {
    auto data = base::JSONReader::ReadDict(request_json, base::JSON_PARSE_RFC);
    auto* token = data ? data->FindString("token") : nullptr;
    auto* record =
        token ? ExplorerRouter::Get(destination).FindTransfer(*token) : nullptr;
    if (!record || !record->pending()) {
      return false;
    }
    record->OfferWindow(surface->render_frame_host());
    surface->active_windows_.insert(*token);
  }
  if (!surface->client_) {
    if (surface->pending_opens_.size() >= kMaxPendingSurfaceEvents) {
      return false;
    }
    surface->pending_opens_.push_back({event_type, request_json});
    return true;
  }
  surface->client_->DidReceiveOpen(event_type, request_json);
  return true;
}

// static
bool ExplorerSurfaceHost::DeliverDialogToWebContents(
    content::WebContents* destination,
    uint64_t request_id,
    const std::string& type,
    const std::string& message,
    const std::string& default_value,
    bool is_reload,
    const std::string& from) {
  if (!destination) {
    return false;
  }
  ExplorerSurfaceHost* surface =
      ExplorerRouter::Get(destination).FindSurfaceForContents(destination);
  if (!surface || !surface->IsCurrentPrimaryDocument()) {
    return false;
  }
  if (!surface->client_) {
    if (surface->pending_dialogs_.size() >= kMaxPendingSurfaceEvents) {
      return false;
    }
    surface->pending_dialogs_.push_back(
        {request_id, type, message, default_value, is_reload, from});
    return true;
  }
  surface->active_dialogs_.insert(request_id);
  surface->client_->DidReceiveDialog(request_id, type, message, default_value,
                                     is_reload, from);
  return true;
}

// static
void ExplorerSurfaceHost::ForgetDialog(content::BrowserContext* context,
                                       uint64_t request_id,
                                       bool canceled) {
  FinishRequest(context, "dialog", base::NumberToString(request_id), canceled);
  for (ExplorerSurfaceHost* host : ExplorerRouter::Get(context).surfaces()) {
    host->active_dialogs_.erase(request_id);
    std::erase_if(host->pending_dialogs_, [&](const PendingDialog& dialog) {
      return dialog.request_id == request_id;
    });
  }
}

void ExplorerSurfaceHost::FinishRequest(content::BrowserContext* context,
                                        const std::string& kind,
                                        const std::string& key,
                                        bool canceled) {
  auto& router = ExplorerRouter::Get(context);
  for (auto* host : router.hosts()) {
    host->FinishRequest(kind, key, canceled);
  }
  for (auto* surface : router.surfaces()) {
    if (surface->client_) {
      surface->client_->DidFinishRequest(kind, key, canceled);
    }
    if (kind == "newwindow") {
      surface->active_windows_.erase(key);
      std::erase_if(surface->pending_opens_, [&](const PendingOpen& open) {
        auto data =
            base::JSONReader::ReadDict(open.request_json, base::JSON_PARSE_RFC);
        auto* token = data ? data->FindString("token") : nullptr;
        return token && *token == key;
      });
    }
  }
}
}  // namespace tuffite
