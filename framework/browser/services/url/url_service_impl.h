// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_SERVICE_IMPL_H_

#include "base/memory/weak_ptr.h"
#include "tuffite/framework/browser/services/url/url_loader_factory.h"
#include "tuffite/framework/browser/services/url/url_service.h"

namespace tuffite {
class BrowserContext;
class URLServiceImpl final : public URLService {
 public:
  explicit URLServiceImpl(BrowserContext&);
  ~URLServiceImpl() override;
  mojo::PendingRemote<network::mojom::URLLoaderFactory>
  CreateNavigationLoaderFactory(content::WebContents&) override;
  mojo::PendingRemote<network::mojom::URLLoaderFactory>
  CreateSubresourceLoaderFactory(content::RenderFrameHost&) override;
  mojo::PendingRemote<network::mojom::URLLoaderFactory>
  CreateWorkerLoaderFactory() override;
  void Shutdown() override;

 private:
  URLResolver Resolver(content::WebContents* = nullptr);
  base::WeakPtr<BrowserContext> browser_;
  const asset::AssetConfiguration assets_;
  bool stopped_ = false;
  base::WeakPtrFactory<URLServiceImpl> weak_factory_{this};
};

}  // namespace tuffite
#endif
