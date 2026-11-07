// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"
#include "mojo/public/cpp/bindings/pending_remote.h"
#include "services/network/public/mojom/url_loader_factory.mojom-forward.h"

namespace content {
class WebContents;
class RenderFrameHost;
}  // namespace content

namespace tuffite {
// Per-profile URL loading. Factories retain snapshots and weak service
// callbacks.
class URLService : public KeyedService {
 public:
  virtual mojo::PendingRemote<network::mojom::URLLoaderFactory>
  CreateNavigationLoaderFactory(content::WebContents&) = 0;
  virtual mojo::PendingRemote<network::mojom::URLLoaderFactory>
  CreateSubresourceLoaderFactory(content::RenderFrameHost&) = 0;
  virtual mojo::PendingRemote<network::mojom::URLLoaderFactory>
  CreateWorkerLoaderFactory() = 0;
};
}  // namespace tuffite
#endif
