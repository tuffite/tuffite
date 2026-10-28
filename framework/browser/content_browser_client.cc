// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/content_browser_client.h"

#include <algorithm>
#include <memory>
#include <utility>

#include "base/command_line.h"
#include "base/functional/bind.h"
#include "base/logging.h"
#include "base/strings/string_util.h"
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
#include "net/http/http_response_headers.h"
#include "net/http/http_util.h"
#include "services/network/public/cpp/content_security_policy/content_security_policy.h"
#include "services/network/public/cpp/web_sandbox_flags.h"
#include "services/network/public/mojom/fetch_api.mojom.h"
#include "services/network/public/mojom/network_context.mojom.h"
#include "services/network/public/mojom/parsed_headers.mojom.h"
#include "services/network/public/mojom/url_response_head.mojom.h"
#include "third_party/blink/public/common/loader/url_loader_throttle.h"
#include "third_party/blink/public/common/renderer_preferences/renderer_preferences.h"
#include "tuffite/components/dispatch/browser/dispatch_host.h"
#include "tuffite/components/dispatch/common/dispatcher.mojom.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/browser/explorer_surface_host.h"
#include "tuffite/components/explorer/common/explorer.mojom.h"
#include "tuffite/components/explorer/common/external_protocol.h"
#include "tuffite/components/isolated/browser/isolated_host.h"
#include "tuffite/framework/app/abi/application.h"
#include "tuffite/framework/app/abi/host.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/asset/app_url_loader_factory.h"
#include "tuffite/framework/app/services/asset/asset_service.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/browser_main_parts.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service_factory.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"
#include "tuffite/framework/browser/services/explorer/explorer_service.h"
#include "tuffite/framework/browser/services/explorer/explorer_service_factory.h"
#include "tuffite/framework/browser/services/explorer/external_protocol.h"
#include "tuffite/framework/browser/services/isolated/isolated_service_factory.h"
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
namespace {
// Own a configuration snapshot, never a service pointer across loader lifetime.
class OriginCspThrottle final : public blink::URLLoaderThrottle {
 public:
  explicit OriginCspThrottle(asset::AssetConfiguration assets)
      : assets_(std::move(assets)) {}
  void WillProcessResponse(const GURL& url,
                           network::mojom::URLResponseHead* head,
                           bool* defer) override {
    if (!url.SchemeIsHTTPOrHTTPS()) {
      return;
    }
    const auto policy = assets_.ContentSecurityPolicy(url, false);
    if (policy.empty() || !head->headers) {
      return;
    }
    head->headers = base::MakeRefCounted<net::HttpResponseHeaders>(
        head->headers->raw_headers());
    head->headers->AddHeader("Content-Security-Policy", policy);
    if (!head->parsed_headers) {
      head->parsed_headers = network::mojom::ParsedHeaders::New();
    }
    auto parsed = network::ParseContentSecurityPolicies(
        policy, network::mojom::ContentSecurityPolicyType::kEnforce,
        network::mojom::ContentSecurityPolicySource::kHTTP, url);
    for (auto& entry : parsed) {
      head->parsed_headers->content_security_policy.push_back(std::move(entry));
    }
  }

 private:
  asset::AssetConfiguration assets_;
};
}  // namespace

std::vector<std::unique_ptr<blink::URLLoaderThrottle>>
ContentBrowserClient::CreateURLLoaderThrottles(
    const network::ResourceRequest&,
    content::BrowserContext* context,
    const base::RepeatingCallback<content::WebContents*()>&,
    content::NavigationUIData*,
    content::FrameTreeNodeId,
    std::optional<int64_t>) {
  std::vector<std::unique_ptr<blink::URLLoaderThrottle>> throttles;
  if (context) {
    throttles.push_back(std::make_unique<OriginCspThrottle>(
        BrowserContext::GetFrom(*context).manifest_service().assets()));
  }
  return throttles;
}

ContentBrowserClient::ContentBrowserClient(
    abi::Application& application,
    abi::Host& host,
    base::RepeatingClosure prepare_browser,
    base::RepeatingClosure begin_shutdown,
    base::RepeatingClosure shutdown)
    : application_(application),
      host_(host),
      prepare_browser_(std::move(prepare_browser)),
      begin_shutdown_(std::move(begin_shutdown)),
      shutdown_(std::move(shutdown)) {}
ApplicationContext* ContentBrowserClient::application_context() const {
  return host_->application_context();
}

ContentBrowserClient::~ContentBrowserClient() = default;

std::vector<base::FilePath>
ContentBrowserClient::GetNetworkContextsParentDirectory() {
  return BrowserContext::GetNetworkStoragePaths();
}

bool ContentBrowserClient::HandleExternalProtocol(
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
    mojo::PendingRemote<network::mojom::URLLoaderFactory>* out_factory) {
  auto* contents = web_contents_getter.Run();
  using SandboxFlags = network::mojom::WebSandboxFlags;
  const bool allowed_by_sandbox =
      is_primary_main_frame ||
      (sandbox_flags & SandboxFlags::kTopNavigationToCustomProtocols) ==
          SandboxFlags::kNone ||
      ((sandbox_flags & SandboxFlags::kTopNavigationByUserActivation) ==
           SandboxFlags::kNone &&
       has_user_gesture);
  VLOG(1) << "External protocol scheme=" << url.scheme()
          << " gesture=" << has_user_gesture
          << " primary=" << is_primary_main_frame
          << " sandbox_allowed=" << allowed_by_sandbox << " explorer="
          << (WebContentsRoleData::GetRole(contents) ==
              WebContentsRole::kExplorer);
  if (contents && allowed_by_sandbox && !is_in_fenced_frame_tree &&
      has_user_gesture && IsExternalProtocol(url, ApplicationSchemes()) &&
      WebContentsRoleData::GetRole(contents) == WebContentsRole::kExplorer) {
    ConfirmExternalProtocol(contents, url, initiating_origin,
                            initiator_document);
  }
  // External URLs never commit in Explorer, including when blocked or
  // dismissed.
  return true;
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
  return application_context() &&
         WebAuthenticationServiceFactory::GetForBrowserContext(
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
  if (!application_context()) {
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
      application_context()
          ? application_context()->GetService<ManifestService>()
          : nullptr;
  return std::ranges::contains(
             manifest ? manifest->app_info().schemes : ApplicationSchemes(),
             url.scheme()) ||
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
      application_context()
          ? application_context()->GetService<ManifestService>()
          : nullptr;
  if (manifest) {
    command_line->AppendSwitchASCII(kShellApiOriginsSwitch,
                                    manifest->SerializeShellApiOrigins());
    command_line->AppendSwitchASCII(
        kAppSchemesSwitch, base::JoinString(manifest->app_info().schemes, ","));
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
  map->Add<mojom::IsolatedHost>(base::BindRepeating(
      [](content::RenderFrameHost* frame,
         mojo::PendingReceiver<mojom::IsolatedHost> receiver) {
        IsolatedHost::Create(IsolatedServiceFactory::GetForBrowserContext(
                                 frame->GetBrowserContext())
                                 ->GetWeakPtr(),
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
  CHECK(application_context());
  ExplorerServiceFactory::GetForBrowserContext(
      registry.GetNavigationHandle().GetWebContents()->GetBrowserContext())
      ->AddNavigationThrottles(registry);
}

std::unique_ptr<content::BrowserMainParts>
ContentBrowserClient::CreateBrowserMainParts(bool is_integration_test) {
  CHECK(application_context());
  auto parts = std::make_unique<BrowserMainParts>(
      *application_, *host_, prepare_browser_, begin_shutdown_, shutdown_);
  return parts;
}

mojo::PendingRemote<network::mojom::URLLoaderFactory>
ContentBrowserClient::CreateNonNetworkNavigationURLLoaderFactory(
    const std::string& scheme,
    content::FrameTreeNodeId frame_tree_node_id) {
  const ManifestService* manifest =
      application_context()
          ? application_context()->GetService<ManifestService>()
          : nullptr;
  if (manifest && std::ranges::contains(manifest->app_info().schemes, scheme)) {
    return application_context()
        ->GetService<AssetService>()
        ->CreateLoaderFactory();
  }
  return {};
}

void ContentBrowserClient::
    RegisterNonNetworkWorkerMainResourceURLLoaderFactories(
        content::BrowserContext* browser_context,
        const std::optional<url::Origin>& request_initiator,
        network::mojom::RequestDestination request_destination,
        NonNetworkURLLoaderFactoryMap* factories) {
  const ManifestService* manifest =
      application_context()
          ? application_context()->GetService<ManifestService>()
          : nullptr;
  if (!manifest || !request_initiator ||
      !std::ranges::contains(manifest->app_info().schemes,
                             request_initiator->scheme()) ||
      (request_destination != network::mojom::RequestDestination::kWorker &&
       request_destination !=
           network::mojom::RequestDestination::kSharedWorker)) {
    return;
  }
  for (const auto& scheme : manifest->app_info().schemes) {
    factories->insert_or_assign(scheme, application_context()
                                            ->GetService<AssetService>()
                                            ->CreateLoaderFactory());
  }
}

void ContentBrowserClient::RegisterNonNetworkSubresourceURLLoaderFactories(
    int render_process_id,
    int render_frame_id,
    const std::optional<url::Origin>& request_initiator_origin,
    NonNetworkURLLoaderFactoryMap* factories) {
  const ManifestService* manifest =
      application_context()
          ? application_context()->GetService<ManifestService>()
          : nullptr;
  if (manifest) {
    for (const auto& scheme : manifest->app_info().schemes) {
      auto* frame =
          content::RenderFrameHost::FromID(render_process_id, render_frame_id);
      asset::MemoryAssetResolver resolver;
      if (frame) {
        auto* service = IsolatedServiceFactory::GetForBrowserContext(
            frame->GetBrowserContext());
        resolver = base::BindRepeating(
            [](base::WeakPtr<IsolatedService> service,
               content::GlobalRenderFrameHostId id,
               const GURL& url) -> std::optional<asset::MemoryAsset> {
              auto* frame = content::RenderFrameHost::FromID(id);
              return service && frame ? service->ResourceForFrame(*frame, url)
                                      : std::nullopt;
            },
            service->GetWeakPtr(), frame->GetGlobalId());
      }
      factories->insert_or_assign(
          scheme,
          AppURLLoaderFactory::Create(manifest->assets(), std::move(resolver)));
    }
  }
}

}  // namespace tuffite
