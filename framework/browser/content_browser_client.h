// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_CONTENT_BROWSER_CLIENT_H_
#define TUFFITE_FRAMEWORK_BROWSER_CONTENT_BROWSER_CLIENT_H_

#include "base/memory/raw_ptr.h"
#include "base/memory/raw_ref.h"
#include "content/public/browser/content_browser_client.h"

namespace base {
class CommandLine;
}

namespace tuffite {
class AbiHostBridge;
class ApplicationContext;

class ContentBrowserClient final : public content::ContentBrowserClient {
 public:
  explicit ContentBrowserClient(AbiHostBridge& bridge);
  ~ContentBrowserClient() override;
  void BindApplicationServices(ApplicationContext* application) {
    application_ = application;
  }

  std::unique_ptr<content::BrowserMainParts> CreateBrowserMainParts(
      bool is_integration_test) override;
  std::unique_ptr<content::DevToolsManagerDelegate>
  CreateDevToolsManagerDelegate() override;
  bool IsSecurityLevelAcceptableForWebAuthn(content::RenderFrameHost* frame,
                                            const url::Origin& origin) override;
  content::WebAuthenticationDelegate* GetWebAuthenticationDelegate() override;
  std::unique_ptr<content::AuthenticatorRequestClientDelegate>
  GetWebAuthenticationRequestDelegate(content::RenderFrameHost* frame) override;
  bool IsHandledURL(const GURL& url) override;
  std::vector<base::FilePath> GetNetworkContextsParentDirectory() override;
  std::string GetProduct() override;
  std::string GetAcceptLangs(content::BrowserContext* context) override;
  void UpdateRendererPreferencesForWorker(
      content::BrowserContext* context,
      blink::RendererPreferences* preferences) override;
  void ConfigureNetworkContextParams(
      content::BrowserContext* context,
      bool in_memory,
      const base::FilePath& relative_partition_path,
      network::mojom::NetworkContextParams* params,
      cert_verifier::mojom::CertVerifierCreationParams* cert_params) override;
  std::string GetUserAgent() override;
  blink::UserAgentMetadata GetUserAgentMetadata() override;
  std::unique_ptr<content::WebContentsViewDelegate> GetWebContentsViewDelegate(
      content::WebContents* web_contents) override;
  std::unique_ptr<content::VideoOverlayWindow>
  CreateWindowForVideoPictureInPicture(
      content::VideoPictureInPictureWindowController* controller) override;
  void AppendExtraCommandLineSwitches(base::CommandLine* command_line,
                                      int child_process_id) override;
  void RegisterBrowserInterfaceBindersForFrame(
      content::RenderFrameHost* render_frame_host,
      mojo::BinderMapWithContext<content::RenderFrameHost*>* map) override;
  void CreateThrottlesForNavigation(
      content::NavigationThrottleRegistry& registry) override;
  mojo::PendingRemote<network::mojom::URLLoaderFactory>
  CreateNonNetworkNavigationURLLoaderFactory(
      const std::string& scheme,
      content::FrameTreeNodeId frame_tree_node_id) override;
  void RegisterNonNetworkWorkerMainResourceURLLoaderFactories(
      content::BrowserContext* browser_context,
      const std::optional<url::Origin>& request_initiator,
      network::mojom::RequestDestination request_destination,
      NonNetworkURLLoaderFactoryMap* factories) override;
  void RegisterNonNetworkSubresourceURLLoaderFactories(
      int render_process_id,
      int render_frame_id,
      const std::optional<url::Origin>& request_initiator_origin,
      NonNetworkURLLoaderFactoryMap* factories) override;

 private:
  const raw_ref<AbiHostBridge> bridge_;
  raw_ptr<ApplicationContext> application_ = nullptr;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_CONTENT_BROWSER_CLIENT_H_
