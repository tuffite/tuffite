// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/asset/asset_service_impl.h"

#include "tuffite/framework/app/services/asset/app_url_loader_factory.h"

namespace tuffite {
AssetServiceImpl::AssetServiceImpl(const AppManifest& manifest)
    : manifest_(manifest) {}
mojo::PendingRemote<network::mojom::URLLoaderFactory>
AssetServiceImpl::CreateLoaderFactory() {
  // Each Mojo factory retains a manifest snapshot and its own request lifetime.
  return AppURLLoaderFactory::Create(*manifest_);
}
}  // namespace tuffite
