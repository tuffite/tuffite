// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/asset/app_url_loader_factory.h"

#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

#include "base/byte_size.h"
#include "base/files/file_path.h"
#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/memory/self_deleting.h"
#include "base/memory/weak_ptr.h"
#include "base/numerics/safe_conversions.h"
#include "base/strings/escape.h"
#include "base/task/thread_pool.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "mojo/public/cpp/bindings/self_owned_receiver.h"
#include "mojo/public/cpp/system/data_pipe.h"
#include "mojo/public/cpp/system/data_pipe_producer.h"
#include "mojo/public/cpp/system/string_data_source.h"
#include "net/base/mime_util.h"
#include "net/base/net_errors.h"
#include "net/http/http_response_headers.h"
#include "net/http/http_version.h"
#include "services/network/public/cpp/resource_request.h"
#include "services/network/public/mojom/url_loader.mojom.h"
#include "services/network/public/mojom/url_response_head.mojom.h"
#include "tuffite/framework/app/services/asset/asset_configuration.h"

namespace tuffite {
namespace {

struct Asset {
  std::string mime_type;
  base::FilePath path;
};

std::optional<Asset> FindAsset(const asset::AssetConfiguration& assets,
                               const GURL& url) {
  const auto path = assets.ResolveAssetFile(url);
  if (!path) {
    return std::nullopt;
  }
  const auto relative = path->BaseName();
  std::string mime_type;
  if (!net::GetMimeTypeFromFile(relative, &mime_type)) {
    mime_type = "application/octet-stream";
  }
  return Asset{std::move(mime_type), *path};
}

constexpr size_t kMaxAssetBytes = 64 * 1024 * 1024;

std::optional<std::string> ReadPackagedAsset(const base::FilePath& path) {
  std::string contents;
  if (!base::ReadFileToStringWithMaxSize(path, &contents, kMaxAssetBytes)) {
    return std::nullopt;
  }
  return contents;
}

class AppURLLoader final : public network::mojom::URLLoader {
 public:
  AppURLLoader(mojo::Remote<network::mojom::URLLoaderClient> client,
               Asset asset,
               std::string content_security_policy)
      : client_(std::move(client)),
        asset_(std::move(asset)),
        content_security_policy_(std::move(content_security_policy)) {}

  void Start() {
    if (asset_.path.empty()) {
      OnAssetRead(std::nullopt);
      return;
    }
    base::ThreadPool::PostTaskAndReplyWithResult(
        FROM_HERE, {base::MayBlock()},
        base::BindOnce(&ReadPackagedAsset, asset_.path),
        base::BindOnce(&AppURLLoader::OnAssetRead, weak_factory_.GetWeakPtr()));
  }

  void FollowRedirect(
      network::HttpRequestHeadersUpdateParams headers_update_params,
      const std::optional<GURL>& new_url) override {}

  void SetPriority(net::RequestPriority priority,
                   int32_t intra_priority_value) override {}

 private:
  void OnAssetRead(std::optional<std::string> contents) {
    const bool found = contents.has_value();
    if (!found) {
      asset_.mime_type = "text/plain";
      contents = "Not found";
    }
    auto response = network::mojom::URLResponseHead::New();
    response->mime_type = asset_.mime_type;
    contents_ = std::move(*contents);
    response->content_length = base::checked_cast<int64_t>(contents_.size());
    response->charset =
        asset_.mime_type == "application/octet-stream" ? "" : "utf-8";
    response->headers =
        net::HttpResponseHeaders::Builder(net::HttpVersion(1, 1),
                                          found ? "200 OK" : "404 Not Found")
            .AddHeader("Content-Type", asset_.mime_type)
            .AddHeader("Cache-Control", "no-store")
            .AddHeader("X-Content-Type-Options", "nosniff")
            .AddHeader("Content-Security-Policy", content_security_policy_)
            .Build();
    mojo::ScopedDataPipeProducerHandle producer;
    mojo::ScopedDataPipeConsumerHandle consumer;
    if (mojo::CreateDataPipe(nullptr, producer, consumer) != MOJO_RESULT_OK) {
      client_->OnComplete(
          network::URLLoaderCompletionStatus(net::ERR_INSUFFICIENT_RESOURCES));
      return;
    }
    client_->OnReceiveResponse(std::move(response), std::move(consumer),
                               std::nullopt);
    producer_ = std::make_unique<mojo::DataPipeProducer>(std::move(producer));
    producer_->Write(std::make_unique<mojo::StringDataSource>(
                         std::string_view(contents_),
                         mojo::StringDataSource::AsyncWritingMode::
                             STRING_STAYS_VALID_UNTIL_COMPLETION),
                     base::BindOnce(&AppURLLoader::OnAssetWritten,
                                    weak_factory_.GetWeakPtr()));
  }

  void OnAssetWritten(MojoResult result) {
    producer_.reset();
    if (result != MOJO_RESULT_OK) {
      client_->OnComplete(network::URLLoaderCompletionStatus(net::ERR_FAILED));
      return;
    }
    network::URLLoaderCompletionStatus status(net::OK);
    status.encoded_data_length = base::ByteSize(contents_.size());
    status.encoded_body_length = base::ByteSize(contents_.size());
    status.decoded_body_length = base::ByteSize(contents_.size());
    client_->OnComplete(status);
    contents_.clear();
  }

  mojo::Remote<network::mojom::URLLoaderClient> client_;
  Asset asset_;
  std::string content_security_policy_;
  std::string contents_;
  std::unique_ptr<mojo::DataPipeProducer> producer_;
  base::WeakPtrFactory<AppURLLoader> weak_factory_{this};
};

}  // namespace

AppURLLoaderFactory::AppURLLoaderFactory(
    mojo::PendingReceiver<network::mojom::URLLoaderFactory> receiver,
    const asset::AssetConfiguration& assets,
    base::SelfDeletingPassKey key)
    : network::SelfDeletingURLLoaderFactory(std::move(receiver), key),
      assets_(assets) {}

AppURLLoaderFactory::~AppURLLoaderFactory() = default;

void AppURLLoaderFactory::CreateLoaderAndStart(
    mojo::PendingReceiver<network::mojom::URLLoader> loader,
    int32_t request_id,
    uint32_t options,
    const network::ResourceRequest& request,
    mojo::PendingRemote<network::mojom::URLLoaderClient> client,
    const net::MutableNetworkTrafficAnnotationTag& traffic_annotation) {
  mojo::Remote<network::mojom::URLLoaderClient> client_remote(
      std::move(client));
  if (!assets_.IsAppUrl(request.url)) {
    client_remote->OnComplete(
        network::URLLoaderCompletionStatus(net::ERR_ACCESS_DENIED));
    return;
  }
  if (request.method != "GET") {
    client_remote->OnComplete(
        network::URLLoaderCompletionStatus(net::ERR_METHOD_NOT_SUPPORTED));
    return;
  }
  const std::optional<Asset> asset = FindAsset(assets_, request.url);
  auto request_loader = std::make_unique<AppURLLoader>(
      std::move(client_remote), asset.value_or(Asset{}),
      assets_.content_security_policy);
  AppURLLoader* request_loader_ptr = request_loader.get();
  mojo::MakeSelfOwnedReceiver(std::move(request_loader), std::move(loader));
  request_loader_ptr->Start();
}

// static
mojo::PendingRemote<network::mojom::URLLoaderFactory>
AppURLLoaderFactory::Create(const asset::AssetConfiguration& assets) {
  mojo::PendingRemote<network::mojom::URLLoaderFactory> remote;
  base::MakeSelfDeleting<AppURLLoaderFactory>(
      remote.InitWithNewPipeAndPassReceiver(), assets);
  return remote;
}

}  // namespace tuffite
