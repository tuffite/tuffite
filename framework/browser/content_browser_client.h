// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_CONTENT_BROWSER_CLIENT_H_
#define TUFFITE_FRAMEWORK_BROWSER_CONTENT_BROWSER_CLIENT_H_

#include "base/functional/callback.h"
#include "base/memory/raw_ref.h"
#include "content/public/browser/content_browser_client.h"

namespace base {
class CommandLine;
}

namespace tuffite {
namespace abi {
class Application;
class Host;
}  // namespace abi
class ApplicationContext;

class ContentBrowserClient final : public content::ContentBrowserClient {
 public:
  ContentBrowserClient(abi::Application& application,
                       abi::Host& host,
                       base::RepeatingClosure prepare_browser,
                       base::RepeatingClosure begin_shutdown,
                       base::RepeatingClosure shutdown);
  ~ContentBrowserClient() override;

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
  bool ShouldUseProcessPerSite(
      content::BrowserContext* browser_context,
      const content::SecurityPrincipal& principal) override;
  std::vector<std::unique_ptr<blink::URLLoaderThrottle>>
  CreateURLLoaderThrottles(
      const network::ResourceRequest&,
      content::BrowserContext*,
      const base::RepeatingCallback<content::WebContents*()>&,
      content::NavigationUIData*,
      content::FrameTreeNodeId,
      std::optional<int64_t>) override;
  bool HandleExternalProtocol(
      const GURL& url,
      base::RepeatingCallback<content::WebContents*()> web_contents_getter,
      content::FrameTreeNodeId frame_tree_node_id,
      content::NavigationUIData* navigation_data,
      bool is_primary_main_frame,
      bool is_in_fenced_frame_tree,
      network::mojom::WebSandboxFlags sandbox_flags,
      ui::PageTransition page_transition,
      bool has_user_gesture,
      const std::optional<url::Origin>& initiating_origin,
      content::RenderFrameHost* initiator_document,
      const net::IsolationInfo& isolation_info,
      mojo::PendingRemote<network::mojom::URLLoaderFactory>* out_factory)
      override;
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
  void OnWebContentsCreated(content::WebContents*) override;
  bool ShouldAllowMojoJsBindingsForFrame(content::RenderFrameHost&) override;
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
  const raw_ref<abi::Application> application_;
  const raw_ref<abi::Host> host_;
  const base::RepeatingClosure prepare_browser_;
  const base::RepeatingClosure begin_shutdown_;
  const base::RepeatingClosure shutdown_;
  ApplicationContext* application_context() const;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_CONTENT_BROWSER_CLIENT_H_
