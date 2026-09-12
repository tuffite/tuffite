// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_SURFACE_HOST_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_SURFACE_HOST_H_

#include <deque>
#include <optional>
#include <set>
#include <string>

#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "content/public/browser/document_service.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/explorer/common/explorer.mojom.h"

namespace content {
class BrowserContext;
class RenderFrameHost;
class WebContents;
}  // namespace content

namespace tuffite {
class ExplorerPolicyProvider;

// Browser-owned route between App and nested Explorer documents. Instances
// are document-scoped: navigation destroys the old SurfaceRef and its route.
class ExplorerSurfaceHost final
    : public content::DocumentService<mojom::ExplorerSurfaceHost> {
 public:
  static void Bind(content::RenderFrameHost* frame,
                   mojo::PendingReceiver<mojom::ExplorerSurfaceHost> receiver,
                   ExplorerPolicyProvider* policy);

  static bool DeliverToWebContents(content::WebContents* destination,
                                   const std::string& json,
                                   const std::string& from);
  static std::string RefForWebContents(content::WebContents* contents);
  static bool DeliverOpenToWebContents(content::WebContents* destination,
                                       const std::string& event_type,
                                       const std::string& request_json);
  static bool DeliverDialogToWebContents(content::WebContents* destination,
                                         uint64_t request_id,
                                         const std::string& type,
                                         const std::string& message,
                                         const std::string& default_value,
                                         bool is_reload,
                                         const std::string& from);
  static void ForgetDialog(content::BrowserContext* context,
                           uint64_t request_id);

  ExplorerSurfaceHost(const ExplorerSurfaceHost&) = delete;
  ExplorerSurfaceHost& operator=(const ExplorerSurfaceHost&) = delete;

  void BindClient(
      mojo::PendingRemote<mojom::ExplorerSurfaceClient> client) override;
  void PostMessage(const std::string& json,
                   const std::string& to,
                   PostMessageCallback callback) override;
  void RespondToDialog(uint64_t request_id,
                       bool accept,
                       const std::string& prompt_text,
                       RespondToDialogCallback callback) override;

 private:
  friend class ExplorerRouter;
  const raw_ref<ExplorerPolicyProvider> policy_provider_;
  struct PendingOpen {
    std::string event_type;
    std::string request_json;
  };
  struct PendingDialog {
    uint64_t request_id = 0;
    std::string type;
    std::string message;
    std::string default_value;
    bool is_reload = false;
    std::string from;
  };
  struct PendingMessage {
    std::string json;
    std::string from;
  };

  ExplorerSurfaceHost(
      content::RenderFrameHost& frame,
      mojo::PendingReceiver<mojom::ExplorerSurfaceHost> receiver,
      ExplorerPolicyProvider& policy);
  ~ExplorerSurfaceHost() override;

  bool IsCurrentPrimaryDocument() const;
  content::WebContents* ResolveDestination(const std::string& to) const;
  bool QueueMessage(const std::string& json, const std::string& from);
  void FlushPendingMessages();
  void OnClientDisconnected();
  void DismissActiveDialogs();
  bool Deliver(const std::string& json, const std::string& from);

  const std::string surface_ref_;
  mojo::Remote<mojom::ExplorerSurfaceClient> client_;
  std::deque<PendingMessage> pending_messages_;
  std::deque<PendingOpen> pending_opens_;
  std::deque<PendingDialog> pending_dialogs_;
  std::set<uint64_t> active_dialogs_;
  size_t pending_message_bytes_ = 0;
  base::WeakPtrFactory<ExplorerSurfaceHost> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_SURFACE_HOST_H_
