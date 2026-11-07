// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_LOADER_FACTORY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_LOADER_FACTORY_H_

#include <optional>
#include <string>

#include "base/memory/self_deleting.h"
#include "mojo/public/cpp/bindings/pending_receiver.h"
#include "mojo/public/cpp/bindings/pending_remote.h"
#include "services/network/public/cpp/self_deleting_url_loader_factory.h"
#include "tuffite/framework/app/services/asset/asset_configuration.h"

namespace network {
struct ResourceRequest;
}

namespace tuffite {

struct URLResolution {
  std::string kind;
  base::FilePath file;
  int status = 200;
  std::string mime_type;
  std::string contents;
};
using URLResolver = base::RepeatingCallback<void(
    const network::ResourceRequest&,
    base::OnceCallback<void(std::optional<URLResolution>)>)>;

class URLLoaderFactory final : public network::SelfDeletingURLLoaderFactory {
 public:
  static mojo::PendingRemote<network::mojom::URLLoaderFactory> Create(
      const asset::AssetConfiguration&,
      asset::MemoryAssetResolver resolver = {},
      URLResolver business_resolver = {});

  URLLoaderFactory(
      mojo::PendingReceiver<network::mojom::URLLoaderFactory> receiver,
      const asset::AssetConfiguration& assets,
      asset::MemoryAssetResolver resolver,
      URLResolver business_resolver,
      base::SelfDeletingPassKey key);

  URLLoaderFactory(const URLLoaderFactory&) = delete;
  URLLoaderFactory& operator=(const URLLoaderFactory&) = delete;

 private:
  ~URLLoaderFactory() override;
  const asset::AssetConfiguration assets_;
  asset::MemoryAssetResolver resolver_;
  URLResolver business_resolver_;

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

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_LOADER_FACTORY_H_
