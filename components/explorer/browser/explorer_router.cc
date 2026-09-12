// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_router.h"

#include <algorithm>

#include "base/check.h"
#include "base/memory/raw_ref.h"
#include "base/supports_user_data.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/browser/explorer_surface_host.h"
#include "tuffite/components/explorer/browser/explorer_transfer.h"

namespace tuffite {
namespace {
const char kRouterKey = 0;
constexpr size_t kMaxMessageBytes = 1024 * 1024;
constexpr size_t kMaxPendingMessages = 256;
constexpr size_t kMaxPendingDocuments = 64;
struct RouterRef : base::SupportsUserData::Data {
  explicit RouterRef(ExplorerRouter& router) : router(router) {}
  raw_ref<ExplorerRouter> router;
};
}  // namespace

ExplorerRouter::ExplorerRouter(content::BrowserContext& context)
    : context_(context) {
  context.SetUserData(&kRouterKey, std::make_unique<RouterRef>(*this));
}
ExplorerRouter::~ExplorerRouter() {
  CHECK(hosts_.empty());
  CHECK(transfers_.empty());
  CHECK(surfaces_.empty());
  context_->RemoveUserData(&kRouterKey);
}
ExplorerRouter& ExplorerRouter::Get(content::BrowserContext* context) {
  CHECK(context);
  auto* reference = static_cast<RouterRef*>(context->GetUserData(&kRouterKey));
  CHECK(reference);
  return *reference->router;
}
ExplorerRouter& ExplorerRouter::Get(content::WebContents* contents) {
  CHECK(contents);
  return Get(contents->GetBrowserContext());
}
void ExplorerRouter::AddTransfer(ExplorerTransferRecord& record) {
  CHECK(transfers_.emplace(record.token(), &record).second);
}
void ExplorerRouter::RemoveTransfer(ExplorerTransferRecord& record) {
  auto found = transfers_.find(record.token());
  CHECK(found != transfers_.end());
  CHECK_EQ(found->second, &record);
  transfers_.erase(found);
}
ExplorerTransferRecord* ExplorerRouter::FindTransfer(
    std::string_view token) const {
  auto found = transfers_.find(token);
  return found == transfers_.end() ? nullptr : found->second;
}
ExplorerTransferRecord* ExplorerRouter::FindTransferForContents(
    content::WebContents* contents) const {
  auto found = std::ranges::find_if(transfers_, [contents](const auto& entry) {
    return entry.second->contents() == contents;
  });
  return found == transfers_.end() ? nullptr : found->second;
}
void ExplorerRouter::AddHost(ExplorerHost& host) {
  CHECK(!std::ranges::contains(hosts_, &host));
  hosts_.push_back(&host);
}
void ExplorerRouter::RemoveHost(ExplorerHost& host) {
  CHECK_EQ(std::erase(hosts_, &host), 1u);
}
ExplorerHost* ExplorerRouter::FindHostForContents(
    content::WebContents* contents) const {
  const auto found =
      std::ranges::find_if(hosts_, [contents](ExplorerHost* host) {
        return host->WebContentsObserver::web_contents() == contents;
      });
  return found == hosts_.end() ? nullptr : *found;
}
void ExplorerRouter::AddSurface(ExplorerSurfaceHost& surface) {
  CHECK(!std::ranges::contains(surfaces_, &surface));
  surfaces_.push_back(&surface);
}
void ExplorerRouter::RemoveSurface(ExplorerSurfaceHost& surface) {
  CHECK_EQ(std::erase(surfaces_, &surface), 1u);
}
ExplorerSurfaceHost* ExplorerRouter::FindSurfaceForContents(
    content::WebContents* contents) const {
  content::RenderFrameHost* primary_frame =
      contents ? contents->GetPrimaryMainFrame() : nullptr;
  const auto found = std::ranges::find_if(
      surfaces_, [primary_frame](ExplorerSurfaceHost* surface) {
        return &surface->render_frame_host() == primary_frame;
      });
  return found == surfaces_.end() ? nullptr : *found;
}
ExplorerSurfaceHost* ExplorerRouter::FindSurfaceByRef(
    std::string_view surface_ref) const {
  const auto found = std::ranges::find(surfaces_, surface_ref,
                                       &ExplorerSurfaceHost::surface_ref_);
  return found == surfaces_.end() ? nullptr : *found;
}
void ExplorerRouter::PrunePendingQueues() {
  std::erase_if(pending_, [](const auto& entry) {
    return !entry.second.document.AsRenderFrameHostIfValid();
  });
}
bool ExplorerRouter::QueueForDocument(content::RenderFrameHost& frame,
                                      const std::string& json,
                                      const std::string& from) {
  PrunePendingQueues();
  auto found = pending_.find(frame.GetGlobalId());
  if (found == pending_.end()) {
    if (pending_.size() >= kMaxPendingDocuments) {
      return false;
    }
    found = pending_.try_emplace(frame.GetGlobalId()).first;
  }
  PendingQueue& queue = found->second;
  queue.document = frame.GetWeakDocumentPtr();
  if (queue.messages.size() >= kMaxPendingMessages ||
      json.size() > kMaxMessageBytes - queue.message_bytes) {
    return false;
  }
  queue.message_bytes += json.size();
  queue.messages.push_back({json, from});
  return true;
}
std::optional<PendingQueue> ExplorerRouter::TakePending(
    content::RenderFrameHost& frame) {
  PrunePendingQueues();
  auto node = pending_.extract(frame.GetGlobalId());
  return node.empty() ? std::nullopt
                      : std::optional<PendingQueue>(std::move(node.mapped()));
}
}  // namespace tuffite
