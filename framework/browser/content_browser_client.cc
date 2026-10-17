// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/content_browser_client.h"

#include <algorithm>
#include <memory>
#include <utility>

#include "base/command_line.h"
#include "base/functional/bind.h"
#include "components/embedder_support/user_agent_utils.h"
#include "components/version_info/version_info.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/navigation_throttle.h"
#include "content/public/browser/navigation_throttle_registry.h"
#include "content/public/browser/overlay_window.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/storage_partition.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_view_delegate.h"
#include "content/public/common/content_switches.h"
#include "mojo/public/cpp/bindings/binder_map.h"
#include "net/http/http_util.h"
#include "services/network/public/mojom/network_context.mojom.h"
#include "third_party/blink/public/common/renderer_preferences/renderer_preferences.h"
#include "tuffite/components/dispatch/browser/dispatch_host.h"
#include "tuffite/components/dispatch/common/dispatcher.mojom.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/browser/explorer_surface_host.h"
#include "tuffite/components/explorer/common/explorer.mojom.h"
#include "tuffite/framework/app/abi_host_bridge.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/asset/asset_service.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/browser_main_parts.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service_factory.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"
#include "tuffite/framework/browser/services/explorer/explorer_service.h"
#include "tuffite/framework/browser/services/explorer/explorer_service_factory.h"
#include "tuffite/framework/browser/services/network_context/network_context_service.h"
#include "tuffite/framework/browser/services/network_context/network_context_service_factory.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service_factory.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service_factory.h"
#include "tuffite/framework/browser/services/window/video_overlay_window.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/browser/web_contents_view_delegate.h"
#include "tuffite/framework/common/languages.h"
#include "tuffite/framework/common/switches.h"
#include "url/gurl.h"
#include "url/origin.h"
#include "url/url_constants.h"

namespace tuffite {
ContentBrowserClient::ContentBrowserClient(AbiHostBridge& bridge)
    : bridge_(bridge) {}
ContentBrowserClient::~ContentBrowserClient() = default;

std::vector<base::FilePath>
ContentBrowserClient::GetNetworkContextsParentDirectory() {
  return BrowserContext::GetNetworkStoragePaths();
}

std::string ContentBrowserClient::GetProduct() {
  return "Tuffite/" + std::string(version_info::GetVersionNumber());
}

std::string ContentBrowserClient::GetAcceptLangs(
    content::BrowserContext* context) {
  return kAcceptLanguages;
}

void ContentBrowserClient::UpdateRendererPreferencesForWorker(
    content::BrowserContext* context,
    blink::RendererPreferences* preferences) {
  preferences->accept_languages = GetAcceptLangs(context);
}

void ContentBrowserClient::ConfigureNetworkContextParams(
    content::BrowserContext* context,
    bool in_memory,
    const base::FilePath& relative_partition_path,
    network::mojom::NetworkContextParams* params,
    cert_verifier::mojom::CertVerifierCreationParams* cert_params) {
  content::ContentBrowserClient::ConfigureNetworkContextParams(
      context, in_memory, relative_partition_path, params, cert_params);
  params->accept_language =
      net::HttpUtil::GenerateAcceptLanguageHeader(GetAcceptLangs(context));
  NetworkContextServiceFactory::GetForBrowserContext(context)->Configure(
      in_memory, relative_partition_path, params);
}

bool ContentBrowserClient::IsSecurityLevelAcceptableForWebAuthn(
    content::RenderFrameHost* frame,
    const url::Origin& origin) {
  return application_ && WebAuthenticationServiceFactory::GetForBrowserContext(
                             frame->GetBrowserContext())
                             ->IsSecurityLevelAcceptable(frame);
}

content::WebAuthenticationDelegate*
ContentBrowserClient::GetWebAuthenticationDelegate() {
  return WebAuthenticationServiceFactory::GetDelegate();
}

std::unique_ptr<content::AuthenticatorRequestClientDelegate>
ContentBrowserClient::GetWebAuthenticationRequestDelegate(
    content::RenderFrameHost* frame) {
  if (!application_) {
    return content::ContentBrowserClient::GetWebAuthenticationRequestDelegate(
        frame);
  }
  return WebAuthenticationServiceFactory::GetForBrowserContext(
             frame->GetBrowserContext())
      ->CreateRequest(frame);
}

std::unique_ptr<content::DevToolsManagerDelegate>
ContentBrowserClient::CreateDevToolsManagerDelegate() {
  return tuffite::CreateDevToolsManagerDelegate();
}

bool ContentBrowserClient::IsHandledURL(const GURL& url) {
  if (!url.is_valid()) {
    return false;
  }
  const ManifestService* manifest =
      application_ ? application_->GetService<ManifestService>() : nullptr;
  return (url.scheme() ==
          (manifest
               ? manifest->app_info().scheme
               : base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
                     kAppSchemeSwitch))) ||
         url.SchemeIsHTTPOrHTTPS() || url.SchemeIs(url::kWsScheme) ||
         url.SchemeIs(url::kWssScheme) || url.SchemeIs(url::kBlobScheme) ||
         url.SchemeIs(url::kFileSystemScheme) ||
         url.SchemeIs(url::kDataScheme) || url.SchemeIs(url::kFileScheme);
}

std::string ContentBrowserClient::GetUserAgent() {
  if (auto user_agent = embedder_support::GetUserAgentFromCommandLine()) {
    return *user_agent;
  }
  return embedder_support::GetUserAgent();
}

blink::UserAgentMetadata ContentBrowserClient::GetUserAgentMetadata() {
  return embedder_support::GetUserAgentMetadata();
}

std::unique_ptr<content::WebContentsViewDelegate>
ContentBrowserClient::GetWebContentsViewDelegate(
    content::WebContents* web_contents) {
  return CreateWebContentsViewDelegate(
      web_contents, *ContextMenuServiceFactory::GetForBrowserContext(
                        web_contents->GetBrowserContext()));
}

std::unique_ptr<content::VideoOverlayWindow>
ContentBrowserClient::CreateWindowForVideoPictureInPicture(
    content::VideoPictureInPictureWindowController* controller) {
  return CreateVideoOverlayWindow(controller);
}

void ContentBrowserClient::AppendExtraCommandLineSwitches(
    base::CommandLine* command_line,
    int child_process_id) {
  const ManifestService* manifest =
      application_ ? application_->GetService<ManifestService>() : nullptr;
  if (manifest) {
    command_line->AppendSwitchASCII(kShellApiOriginsSwitch,
                                    manifest->SerializeShellApiOrigins());
    command_line->AppendSwitchASCII(kAppSchemeSwitch,
                                    manifest->app_info().scheme);
    const std::string explorer_origins = manifest->SerializeExplorerOrigins();
    command_line->AppendSwitchASCII(kExplorerOriginsSwitch, explorer_origins);
    if (!explorer_origins.empty()) {
      std::string features =
          command_line->GetSwitchValueASCII(switches::kEnableBlinkFeatures);
      if (!features.empty()) {
        features.push_back(',');
      }
      features.append("TuffiteExplorer");
      command_line->AppendSwitchASCII(switches::kEnableBlinkFeatures, features);
    }
  }
}

void ContentBrowserClient::RegisterBrowserInterfaceBindersForFrame(
    content::RenderFrameHost* render_frame_host,
    mojo::BinderMapWithContext<content::RenderFrameHost*>* map) {
  map->Add<mojom::Dispatcher>(base::BindRepeating(
      [](content::RenderFrameHost* frame,
         mojo::PendingReceiver<mojom::Dispatcher> receiver) {
        DispatchHost::Create(ShellApiServiceFactory::GetForBrowserContext(
                                 frame->GetBrowserContext()),
                             frame, std::move(receiver));
      }));
  // Binder maps are created before the first document necessarily has a
  // committed origin. Register the interface unconditionally here and enforce
  // the application role and exact origin in ExplorerHost::Bind().
  map->Add<mojom::ExplorerHost>(base::BindRepeating(
      [](content::RenderFrameHost* frame,
         mojo::PendingReceiver<mojom::ExplorerHost> receiver) {
        ExplorerHost::Bind(frame, std::move(receiver),
                           ExplorerServiceFactory::GetForBrowserContext(
                               frame->GetBrowserContext()));
      }));
  map->Add<mojom::ExplorerSurfaceHost>(base::BindRepeating(
      [](content::RenderFrameHost* frame,
         mojo::PendingReceiver<mojom::ExplorerSurfaceHost> receiver) {
        ExplorerSurfaceHost::Bind(frame, std::move(receiver),
                                  ExplorerServiceFactory::GetForBrowserContext(
                                      frame->GetBrowserContext()));
      }));
}

void ContentBrowserClient::CreateThrottlesForNavigation(
    content::NavigationThrottleRegistry& registry) {
  CHECK(application_);
  ExplorerServiceFactory::GetForBrowserContext(
      registry.GetNavigationHandle().GetWebContents()->GetBrowserContext())
      ->AddNavigationThrottles(registry);
}

std::unique_ptr<content::BrowserMainParts>
ContentBrowserClient::CreateBrowserMainParts(bool is_integration_test) {
  CHECK(application_);
  auto parts = std::make_unique<BrowserMainParts>(*bridge_, *application_);
  return parts;
}

mojo::PendingRemote<network::mojom::URLLoaderFactory>
ContentBrowserClient::CreateNonNetworkNavigationURLLoaderFactory(
    const std::string& scheme,
    content::FrameTreeNodeId frame_tree_node_id) {
  const ManifestService* manifest =
      application_ ? application_->GetService<ManifestService>() : nullptr;
  if (manifest && scheme == manifest->app_info().scheme) {
    return application_->GetService<AssetService>()->CreateLoaderFactory();
  }
  return {};
}

void ContentBrowserClient::RegisterNonNetworkSubresourceURLLoaderFactories(
    int render_process_id,
    int render_frame_id,
    const std::optional<url::Origin>& request_initiator_origin,
    NonNetworkURLLoaderFactoryMap* factories) {
  const ManifestService* manifest =
      application_ ? application_->GetService<ManifestService>() : nullptr;
  if (manifest) {
    factories->insert_or_assign(
        manifest->app_info().scheme,
        application_->GetService<AssetService>()->CreateLoaderFactory());
  }
}

}  // namespace tuffite
