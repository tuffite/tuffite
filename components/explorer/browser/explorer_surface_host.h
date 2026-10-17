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

// Document-scoped request controllers, ancestry metadata and WindowInfo
// delivery.
class ExplorerSurfaceHost final
    : public content::DocumentService<mojom::ExplorerSurfaceHost> {
 public:
  static void Bind(content::RenderFrameHost* frame,
                   mojo::PendingReceiver<mojom::ExplorerSurfaceHost> receiver,
                   ExplorerPolicyProvider* policy);

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
                           uint64_t request_id,
                           bool canceled = true);
  static void FinishRequest(content::BrowserContext*,
                            const std::string& kind,
                            const std::string& key,
                            bool canceled);

  ExplorerSurfaceHost(const ExplorerSurfaceHost&) = delete;
  ExplorerSurfaceHost& operator=(const ExplorerSurfaceHost&) = delete;

  void PostWindowMessage(int64_t,
                         blink::TransferableMessage,
                         const std::string&) override;
  void GetFrameAncestors(GetFrameAncestorsCallback) override;
  static void UpdateFrameAncestors(content::WebContents*,
                                   base::OnceCallback<void(bool)>,
                                   unsigned attempt = 0);
  void BindClient(
      mojo::PendingRemote<mojom::ExplorerSurfaceClient> client) override;
  void DiscardWindow(const std::string&, DiscardWindowCallback) override;
  void CompleteDialog(uint64_t request_id, bool propagate) override;
  void CompleteOpen(const std::string& token, bool propagate) override;
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

  ExplorerSurfaceHost(
      content::RenderFrameHost& frame,
      mojo::PendingReceiver<mojom::ExplorerSurfaceHost> receiver,
      ExplorerPolicyProvider& policy);
  ~ExplorerSurfaceHost() override;

  bool IsCurrentPrimaryDocument() const;
  void FlushPendingRequests();
  void OnClientDisconnected();
  void DismissActiveDialogs();

  const std::string surface_ref_;
  mojo::Remote<mojom::ExplorerSurfaceClient> client_;
  std::deque<PendingOpen> pending_opens_;
  std::deque<PendingDialog> pending_dialogs_;
  std::set<uint64_t> active_dialogs_;
  std::set<std::string> active_windows_;
  base::WeakPtrFactory<ExplorerSurfaceHost> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_SURFACE_HOST_H_
