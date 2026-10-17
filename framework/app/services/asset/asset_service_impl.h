// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_IMPL_H_

#include "base/memory/raw_ref.h"
#include "tuffite/framework/app/services/asset/asset_service.h"
namespace tuffite {
class ManifestService;
class AssetServiceImpl final : public AssetService {
 public:
  explicit AssetServiceImpl(const ManifestService& manifest);
  mojo::PendingRemote<network::mojom::URLLoaderFactory> CreateLoaderFactory()
      override;

 private:
  const raw_ref<const ManifestService> manifest_;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_IMPL_H_
