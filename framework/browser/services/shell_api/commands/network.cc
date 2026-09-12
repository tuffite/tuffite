// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/network.h"

#include <memory>
#include <optional>
#include <string>
#include <utility>

#include "base/functional/bind.h"
#include "base/time/time.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/storage_partition.h"
#include "net/base/isolation_info.h"
#include "net/cookies/site_for_cookies.h"
#include "net/traffic_annotation/network_traffic_annotation.h"
#include "services/network/public/cpp/resource_request.h"
#include "services/network/public/cpp/shared_url_loader_factory.h"
#include "services/network/public/cpp/simple_url_loader.h"
#include "services/network/public/mojom/url_loader_factory.mojom.h"
#include "services/network/public/mojom/url_response_head.mojom.h"
#include "tuffite/framework/browser/services/shell_api/commands/application_policy.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
#include "tuffite/framework/common/app_manifest.h"
#include "url/gurl.h"

namespace tuffite::shell_api::net {

using generated::net::RequestCallback;
using generated::net::RequestRequest;
using generated::net::RequestResult;

namespace {

class NetworkRequest {
 public:
  static void Start(scoped_refptr<::network::SharedURLLoaderFactory> factory,
                    const AppManifest& manifest,
                    const GURL& url,
                    RequestCallback callback) {
    auto request = std::make_unique<::network::ResourceRequest>();
    request->url = url;
    request->method = "GET";
    request->credentials_mode = ::network::mojom::CredentialsMode::kOmit;
    request->redirect_mode = ::network::mojom::RedirectMode::kManual;
    request->trusted_params = ::network::ResourceRequest::TrustedParams();
    request->trusted_params->isolation_info =
        ::net::IsolationInfo::CreateForInternalRequest(
            url::Origin::Create(url));
    request->site_for_cookies =
        request->trusted_params->isolation_info.site_for_cookies();

    auto operation = std::unique_ptr<NetworkRequest>(new NetworkRequest(
        std::move(factory), manifest, std::move(request), std::move(callback)));
    NetworkRequest* operation_ptr = operation.get();
    operation_ptr->loader_->DownloadToString(
        operation_ptr->factory_.get(),
        base::BindOnce(&NetworkRequest::OnComplete, std::move(operation)),
        manifest.network_max_response_bytes());
  }

 private:
  NetworkRequest(scoped_refptr<::network::SharedURLLoaderFactory> factory,
                 const AppManifest& manifest,
                 std::unique_ptr<::network::ResourceRequest> request,
                 RequestCallback callback)
      : factory_(std::move(factory)), callback_(std::move(callback)) {
    static constexpr ::net::NetworkTrafficAnnotationTag kTrafficAnnotation =
        ::net::DefineNetworkTrafficAnnotation("tuffite_shell_api_network", R"(
          semantics {
            sender: "Tuffite ShellAPI"
            description: "Performs a request initiated by a Tuffite app."
            trigger: "The app calls window.tuffite.net.request()."
            data: "URL and standard Chromium network metadata."
            destination: WEBSITE
          }
          policy {
            cookies_allowed: NO
            setting: "Controlled by the Tuffite capability manifest."
            policy_exception_justification: "Not implemented."
          })");
    loader_ = ::network::SimpleURLLoader::Create(std::move(request),
                                                 kTrafficAnnotation);
    loader_->SetURLLoaderFactoryOptions(
        ::network::mojom::kURLLoadOptionBlockLocalRequest);
    loader_->SetAllowHttpErrorResults(true);
    loader_->SetTimeoutDuration(
        base::Seconds(manifest.network_timeout_seconds()));
  }

  static void OnComplete(std::unique_ptr<NetworkRequest> request,
                         std::optional<std::string> body) {
    if (request->loader_->NetError() != 0) {
      std::move(request->callback_).Run(kStatusUnavailable, {});
      return;
    }
    RequestResult result;
    if (request->loader_->ResponseInfo() &&
        request->loader_->ResponseInfo()->headers) {
      result.status =
          request->loader_->ResponseInfo()->headers->response_code();
    }
    if (body) {
      result.body.assign(body->begin(), body->end());
    }
    std::move(request->callback_).Run(kStatusOk, std::move(result));
  }

  scoped_refptr<::network::SharedURLLoaderFactory> factory_;
  RequestCallback callback_;
  std::unique_ptr<::network::SimpleURLLoader> loader_;
};

}  // namespace

void Request(const ::tuffite::dispatch::Context& context,
             RequestRequest&& request,
             RequestCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  const AppManifest* manifest = ManifestFor(context.frame());
  const GURL target(request.url);
  if (!manifest || !manifest->IsNetworkUrlAllowed(
                       context.sender->GetLastCommittedOrigin(), target)) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  NetworkRequest::Start(context.sender->GetStoragePartition()
                            ->GetURLLoaderFactoryForBrowserProcess(),
                        *manifest, target, std::move(callback));
}

}  // namespace tuffite::shell_api::net
