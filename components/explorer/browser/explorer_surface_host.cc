// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_surface_host.h"

#include <algorithm>
#include <deque>
#include <map>
#include <vector>

#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/no_destructor.h"
#include "base/unguessable_token.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/global_routing_id.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/browser/explorer_policy_provider.h"
#include "tuffite/components/explorer/browser/explorer_router.h"
#include "tuffite/components/explorer/browser/explorer_surface_data.h"

namespace tuffite {
namespace {

constexpr size_t kMaxMessageBytes = 1024 * 1024;
constexpr size_t kMaxPendingMessages = 256;
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
  if (auto queued = router.TakePending(frame)) {
    for (QueuedMessage& message : queued->messages) {
      pending_messages_.push_back(
          {std::move(message.json), std::move(message.from)});
    }
    pending_message_bytes_ = queued->message_bytes;
  }
}

ExplorerSurfaceHost::~ExplorerSurfaceHost() {
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
                                         std::string());
  }
}

bool ExplorerSurfaceHost::IsCurrentPrimaryDocument() const {
  content::WebContents* contents = ContentsFor(render_frame_host());
  return contents && contents->GetPrimaryMainFrame() == &render_frame_host();
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
    FlushPendingMessages();
  }
}

content::WebContents* ExplorerSurfaceHost::ResolveDestination(
    const std::string& to) const {
  content::WebContents* source = ContentsFor(render_frame_host());
  if (!source) {
    return nullptr;
  }
  if (to == "parent") {
    const ExplorerPolicy* policy =
        ExplorerSurfaceData::GetExplorerPolicy(source);
    return policy && policy->allow_parent_messages
               ? source->GetOuterWebContents()
               : nullptr;
  }
  if (to == "top") {
    if (ExplorerRouter::Get(source).FindTransferForContents(
            source->GetOutermostWebContents())) {
      return nullptr;
    }
    const ExplorerPolicy* policy =
        ExplorerSurfaceData::GetExplorerPolicy(source);
    return policy && policy->allow_top_messages
               ? source->GetOutermostWebContents()
               : nullptr;
  }
  ExplorerSurfaceHost* destination_surface =
      ExplorerRouter::Get(source).FindSurfaceByRef(to);
  if (!destination_surface) {
    return nullptr;
  }
  content::WebContents* destination =
      ContentsFor(destination_surface->render_frame_host());
  const bool trusted_root = policy_provider_->IsApp(source) &&
                            source == source->GetOutermostWebContents();
  const bool direct_parent =
      destination && destination->GetOuterWebContents() == source;
  return destination &&
                 destination->GetOutermostWebContents() ==
                     source->GetOutermostWebContents() &&
                 (trusted_root || direct_parent)
             ? destination
             : nullptr;
}

void ExplorerSurfaceHost::PostMessage(const std::string& json,
                                      const std::string& to,
                                      PostMessageCallback callback) {
  const bool valid_json =
      base::JSONReader::Read(json, base::JSON_PARSE_RFC).has_value();
  content::WebContents* destination =
      json.size() <= kMaxMessageBytes && valid_json ? ResolveDestination(to)
                                                    : nullptr;
  const bool accepted =
      destination && DeliverToWebContents(destination, json, surface_ref_);
  std::move(callback).Run(accepted);
}

void ExplorerSurfaceHost::RespondToDialog(uint64_t request_id,
                                          bool accept,
                                          const std::string& prompt_text,
                                          RespondToDialogCallback callback) {
  std::move(callback).Run(ExplorerHost::RespondToDialogForTree(
      ContentsFor(render_frame_host()), request_id, accept, prompt_text));
}

bool ExplorerSurfaceHost::Deliver(const std::string& json,
                                  const std::string& from) {
  if (!IsCurrentPrimaryDocument()) {
    return false;
  }
  if (!client_) {
    return QueueMessage(json, from);
  }
  client_->DidReceiveMessage(json, from);
  return true;
}

bool ExplorerSurfaceHost::QueueMessage(const std::string& json,
                                       const std::string& from) {
  if (pending_messages_.size() >= kMaxPendingMessages ||
      json.size() > kMaxMessageBytes - pending_message_bytes_) {
    return false;
  }
  pending_message_bytes_ += json.size();
  pending_messages_.push_back({json, from});
  return true;
}

void ExplorerSurfaceHost::FlushPendingMessages() {
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
  while (client_ && !pending_messages_.empty() && IsCurrentPrimaryDocument()) {
    PendingMessage message = std::move(pending_messages_.front());
    pending_messages_.pop_front();
    pending_message_bytes_ -= message.json.size();
    client_->DidReceiveMessage(message.json, message.from);
  }
}

void ExplorerSurfaceHost::OnClientDisconnected() {
  client_.reset();
  DismissActiveDialogs();
}

void ExplorerSurfaceHost::DismissActiveDialogs() {
  content::WebContents* contents = ContentsFor(render_frame_host());
  std::vector<uint64_t> request_ids(active_dialogs_.begin(),
                                    active_dialogs_.end());
  active_dialogs_.clear();
  for (uint64_t request_id : request_ids) {
    ExplorerHost::RespondToDialogForTree(contents, request_id, false,
                                         std::string());
  }
}

// static
bool ExplorerSurfaceHost::DeliverToWebContents(
    content::WebContents* destination,
    const std::string& json,
    const std::string& from) {
  if (!destination) {
    return false;
  }
  content::RenderFrameHost* destination_frame =
      destination ? destination->GetPrimaryMainFrame() : nullptr;
  if (!destination_frame) {
    return false;
  }
  ExplorerRouter& router = ExplorerRouter::Get(destination);
  ExplorerSurfaceHost* surface = router.FindSurfaceForContents(destination);
  return surface ? surface->Deliver(json, from)
                 : router.QueueForDocument(*destination_frame, json, from);
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
                                       uint64_t request_id) {
  for (ExplorerSurfaceHost* host : ExplorerRouter::Get(context).surfaces()) {
    host->active_dialogs_.erase(request_id);
    std::erase_if(host->pending_dialogs_, [&](const PendingDialog& dialog) {
      return dialog.request_id == request_id;
    });
  }
}

}  // namespace tuffite
