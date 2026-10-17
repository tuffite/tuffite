// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_APP_URL_LOADER_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_APP_URL_LOADER_FACTORY_H_

#include "base/memory/self_deleting.h"
#include "mojo/public/cpp/bindings/pending_receiver.h"
#include "mojo/public/cpp/bindings/pending_remote.h"
#include "services/network/public/cpp/self_deleting_url_loader_factory.h"
#include "tuffite/framework/app/services/asset/asset_configuration.h"

namespace tuffite {

class AppURLLoaderFactory final : public network::SelfDeletingURLLoaderFactory {
 public:
  static mojo::PendingRemote<network::mojom::URLLoaderFactory> Create(
      const asset::AssetConfiguration&);

  AppURLLoaderFactory(
      mojo::PendingReceiver<network::mojom::URLLoaderFactory> receiver,
      const asset::AssetConfiguration& assets,
      base::SelfDeletingPassKey key);

  AppURLLoaderFactory(const AppURLLoaderFactory&) = delete;
  AppURLLoaderFactory& operator=(const AppURLLoaderFactory&) = delete;

 private:
  ~AppURLLoaderFactory() override;
  const asset::AssetConfiguration assets_;

  void CreateLoaderAndStart(
      mojo::PendingReceiver<network::mojom::URLLoader> loader,
      int32_t request_id,
      uint32_t options,
      const network::ResourceRequest& request,
      mojo::PendingRemote<network::mojom::URLLoaderClient> client,
      const net::MutableNetworkTrafficAnnotationTag& traffic_annotation)
      override;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_ASSET_APP_URL_LOADER_FACTORY_H_
