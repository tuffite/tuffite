// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_ROUTER_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_ROUTER_H_

#include <deque>
#include <map>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "base/containers/span.h"
#include "base/memory/raw_ref.h"
#include "content/public/browser/global_routing_id.h"
#include "content/public/browser/weak_document_ptr.h"

namespace content {
class BrowserContext;
class RenderFrameHost;
class WebContents;
}  // namespace content

namespace tuffite {
class ExplorerHost;
class ExplorerTransferRecord;
class ExplorerSurfaceHost;

struct QueuedMessage {
  std::string json;
  std::string from;
};

struct PendingQueue {
  std::deque<QueuedMessage> messages;
  size_t message_bytes = 0;
  content::WeakDocumentPtr document;
};

class ExplorerRouter final {
 public:
  explicit ExplorerRouter(content::BrowserContext&);
  ~ExplorerRouter();
  static ExplorerRouter& Get(content::BrowserContext*);
  static ExplorerRouter& Get(content::WebContents*);

  // Pending pages can outlive their elements. Index credentials separately
  // from the small, attached page trees.
  void AddTransfer(ExplorerTransferRecord&);
  void RemoveTransfer(ExplorerTransferRecord&);
  ExplorerTransferRecord* FindTransfer(std::string_view token) const;
  ExplorerTransferRecord* FindTransferForContents(content::WebContents*) const;
  size_t transfer_count() const { return transfers_.size(); }

  void AddHost(ExplorerHost&);
  void RemoveHost(ExplorerHost&);
  ExplorerHost* FindHostForContents(content::WebContents*) const;
  base::span<ExplorerHost* const> hosts() const { return hosts_; }

  void AddSurface(ExplorerSurfaceHost&);
  void RemoveSurface(ExplorerSurfaceHost&);
  ExplorerSurfaceHost* FindSurfaceForContents(content::WebContents*) const;
  ExplorerSurfaceHost* FindSurfaceByRef(std::string_view) const;
  base::span<ExplorerSurfaceHost* const> surfaces() const { return surfaces_; }

  bool QueueForDocument(content::RenderFrameHost&,
                        const std::string& json,
                        const std::string& from);
  std::optional<PendingQueue> TakePending(content::RenderFrameHost&);

 private:
  void PrunePendingQueues();

  const raw_ref<content::BrowserContext> context_;
  std::vector<ExplorerHost*> hosts_;
  std::map<std::string, ExplorerTransferRecord*, std::less<>> transfers_;
  std::vector<ExplorerSurfaceHost*> surfaces_;
  std::map<content::GlobalRenderFrameHostId, PendingQueue> pending_;
};
}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_ROUTER_H_
