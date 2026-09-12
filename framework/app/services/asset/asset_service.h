// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"
#include "mojo/public/cpp/bindings/pending_remote.h"
#include "services/network/public/mojom/url_loader_factory.mojom-forward.h"

namespace tuffite {
class AssetService : public KeyedService {
 public:
  virtual mojo::PendingRemote<network::mojom::URLLoaderFactory>
  CreateLoaderFactory() = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_ASSET_SERVICE_H_
