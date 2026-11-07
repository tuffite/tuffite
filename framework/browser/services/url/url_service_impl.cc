// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/url/url_service_impl.h"

#include "base/base64.h"
#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/strings/string_number_conversions.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "services/network/public/cpp/resource_request.h"
#include "services/network/public/mojom/fetch_api.mojom.h"
#include "tuffite/components/dispatch/browser/host_dispatch.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/profile/profile_manager.h"
#include "tuffite/framework/app/services/window/window.h"
#include "tuffite/framework/app/services/window/window_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/isolated/isolated_service.h"
#include "tuffite/framework/browser/services/isolated/isolated_service_factory.h"

namespace tuffite {
URLServiceImpl::URLServiceImpl(BrowserContext& browser)
    : browser_(browser.GetWeakPtr()),
      assets_(browser.manifest_service().assets()) {}
URLServiceImpl::~URLServiceImpl() = default;
void URLServiceImpl::Shutdown() {
  stopped_ = true;
  weak_factory_.InvalidateWeakPtrs();
}
URLResolver URLServiceImpl::Resolver(content::WebContents* contents) {
  auto* browser = browser_.get();
  std::string profile;
  if (browser) {
    profile = BrowserContext::GetFrom(*browser)
                  .application_context()
                  .GetService<ProfileManager>()
                  ->IdFor(browser);
  }
  return base::BindRepeating(
      [](base::WeakPtr<URLServiceImpl> service,
         base::WeakPtr<BrowserContext> browser,
         base::WeakPtr<content::WebContents> contents, bool document_bound,
         std::string profile, const network::ResourceRequest& resource,
         base::OnceCallback<void(std::optional<URLResolution>)> done) {
        if (!service || service->stopped_ || !browser ||
            (document_bound && !contents)) {
          std::move(done).Run(std::nullopt);
          return;
        }
        auto data =
            base::DictValue()
                .Set("url", resource.url.spec())
                .Set("method", resource.method)
                .Set("navigation",
                     resource.destination ==
                             network::mojom::RequestDestination::kDocument ||
                         resource.destination ==
                             network::mojom::RequestDestination::kIframe);
        if (!profile.empty()) {
          data.Set("profile", profile);
        }
        if (contents) {
          auto* window =
              Window::FromWebContents(contents->GetOutermostWebContents());
          auto* windows =
              BrowserContext::GetFrom(*contents->GetBrowserContext())
                  .application_context()
                  .GetService<window::WindowService>();
          if (auto info = windows->GetInfo(window)) {
            data.Set("window", base::NumberToString(info->id));
          }
        }
        if (resource.request_initiator &&
            !resource.request_initiator->opaque()) {
          data.Set("initiator", resource.request_initiator->Serialize());
        }
        auto request = base::WriteJson(data);
        browser->host_dispatch().DispatchFramework(
            UINT64_MAX, *request,
            base::BindOnce(
                [](base::WeakPtr<URLServiceImpl> service,
                   base::OnceCallback<void(std::optional<URLResolution>)> done,
                   bool ok, std::vector<uint8_t> bytes) {
                  auto value =
                      ok && service && !service->stopped_
                          ? base::JSONReader::ReadDict(
                                std::string_view(
                                    reinterpret_cast<const char*>(bytes.data()),
                                    bytes.size()),
                                base::JSON_PARSE_RFC)
                          : std::nullopt;
                  const auto* kind =
                      value ? value->FindString("kind") : nullptr;
                  URLResolution response;
                  if (!kind) {
                    std::move(done).Run(std::nullopt);
                    return;
                  }
                  response.kind = *kind;
                  if (*kind != "resource") {
                    std::move(done).Run(
                        (*kind == "pass" || *kind == "consumed" ||
                         *kind == "discard")
                            ? std::make_optional(std::move(response))
                            : std::nullopt);
                    return;
                  }
                  const auto* mime = value->FindString("mimeType");
                  const auto* body = value->FindString("body");
                  const auto* file = value->FindString("file");
                  const auto status = value->FindInt("status");
                  if ((mime && (mime->empty() || mime->find_first_of("\r\n") !=
                                                     std::string::npos)) ||
                      (body != nullptr) == (file != nullptr) || !status ||
                      *status < 200 || *status > 599 ||
                      (*status >= 300 && *status < 400) ||
                      (body &&
                       (!base::Base64Decode(*body, &response.contents) ||
                        response.contents.size() > 8 * 1024 * 1024))) {
                    std::move(done).Run(std::nullopt);
                    return;
                  }
                  if (file) {
                    response.file = base::FilePath::FromUTF8Unsafe(*file);
                    if (!response.file.IsAbsolute() ||
                        file->find('\0') != std::string::npos) {
                      std::move(done).Run(std::nullopt);
                      return;
                    }
                  }
                  response.status = *status;
                  if (mime) {
                    response.mime_type = *mime;
                  }
                  std::move(done).Run(std::move(response));
                },
                service, std::move(done)));
      },
      weak_factory_.GetWeakPtr(),
      browser ? BrowserContext::GetFrom(*browser).GetWeakPtr()
              : base::WeakPtr<BrowserContext>(),
      contents ? contents->GetWeakPtr() : base::WeakPtr<content::WebContents>(),
      contents != nullptr, std::move(profile));
}

mojo::PendingRemote<network::mojom::URLLoaderFactory>
URLServiceImpl::CreateNavigationLoaderFactory(content::WebContents& contents) {
  if (stopped_ || !browser_ || contents.GetBrowserContext() != browser_.get()) {
    return {};
  }
  return URLLoaderFactory::Create(assets_, {}, Resolver(&contents));
}
mojo::PendingRemote<network::mojom::URLLoaderFactory>
URLServiceImpl::CreateWorkerLoaderFactory() {
  if (stopped_ || !browser_) {
    return {};
  }
  return URLLoaderFactory::Create(assets_, {}, Resolver());
}
mojo::PendingRemote<network::mojom::URLLoaderFactory>
URLServiceImpl::CreateSubresourceLoaderFactory(
    content::RenderFrameHost& frame) {
  if (stopped_ || !browser_ || frame.GetBrowserContext() != browser_.get()) {
    return {};
  }
  auto* isolated = IsolatedServiceFactory::GetForBrowserContext(browser_.get());
  auto memory = base::BindRepeating(
      [](base::WeakPtr<IsolatedService> service,
         content::GlobalRenderFrameHostId id,
         const GURL& url) -> std::optional<asset::MemoryAsset> {
        auto* frame = content::RenderFrameHost::FromID(id);
        return service && frame ? service->ResourceForFrame(*frame, url)
                                : std::nullopt;
      },
      isolated->GetWeakPtr(), frame.GetGlobalId());
  return URLLoaderFactory::Create(
      assets_, std::move(memory),
      Resolver(content::WebContents::FromRenderFrameHost(&frame)));
}
}  // namespace tuffite
