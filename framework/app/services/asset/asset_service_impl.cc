// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/asset/asset_service_impl.h"

#include "tuffite/framework/app/services/asset/app_url_loader_factory.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"

namespace tuffite {
AssetServiceImpl::AssetServiceImpl(const ManifestService& manifest)
    : manifest_(manifest) {}
mojo::PendingRemote<network::mojom::URLLoaderFactory>
AssetServiceImpl::CreateLoaderFactory() {
  // Each Mojo factory retains a resource configuration snapshot and its own
  // request lifetime.
  return AppURLLoaderFactory::Create(manifest_->assets());
}
}  // namespace tuffite
