// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/mojom_bindings.h"

#include <map>

#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_observer.h"
#include "content/public/browser/web_contents_user_data.h"
#include "tuffite/components/dispatch/browser/mojom_interface_broker.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service_factory.h"
#include "tuffite/framework/browser/web_contents_role.h"
namespace tuffite {
namespace {
class MojomDocuments final
    : public content::WebContentsObserver,
      public content::WebContentsUserData<MojomDocuments> {
 public:
  void ReadyToCommitNavigation(content::NavigationHandle* navigation) override {
    if (navigation->IsSameDocument() || navigation->IsPageActivation() ||
        !navigation->IsInPrimaryMainFrame()) {
      return;
    }
    auto* frame = navigation->GetRenderFrameHost();
    const auto origin = navigation->GetOriginToCommit();
    const auto role = WebContentsRoleData::GetRole(web_contents());
    const auto* policy = WebContentsRoleData::GetExplorerPolicy(web_contents());
    if (!origin || origin->opaque() || !frame ||
        !(role == WebContentsRole::kApp ||
          (role == WebContentsRole::kExplorer && policy &&
           policy->allow_shell_api))) {
      return;
    }
    auto* context = static_cast<BrowserContext*>(frame->GetBrowserContext());
    if (!context->manifest_service().IsShellApiOriginAllowed(*origin)) {
      return;
    }
    mojo::PendingRemote<blink::mojom::BrowserInterfaceBroker> broker;
    pending_.emplace(navigation->GetNavigationId(),
                     broker.InitWithNewPipeAndPassReceiver());
    frame->EnableMojoJsBindingsWithBroker(std::move(broker));
  }
  void DidFinishNavigation(content::NavigationHandle* navigation) override {
    auto pending = pending_.extract(navigation->GetNavigationId());
    if (pending.empty() || !navigation->HasCommitted() ||
        navigation->IsSameDocument()) {
      return;
    }
    auto* frame = navigation->GetRenderFrameHost();
    BindMojomInterfaceBroker(ShellApiServiceFactory::GetForBrowserContext(
                                 frame->GetBrowserContext()),
                             frame, std::move(pending.mapped()));
  }

 private:
  friend class content::WebContentsUserData<MojomDocuments>;
  explicit MojomDocuments(content::WebContents* contents)
      : WebContentsObserver(contents), WebContentsUserData(*contents) {}
  std::map<int64_t, mojo::PendingReceiver<blink::mojom::BrowserInterfaceBroker>>
      pending_;
  WEB_CONTENTS_USER_DATA_KEY_DECL();
};
WEB_CONTENTS_USER_DATA_KEY_IMPL(MojomDocuments);
}  // namespace
void ObserveMojomDocuments(content::WebContents* contents) {
  MojomDocuments::CreateForWebContents(contents);
}
}  // namespace tuffite
