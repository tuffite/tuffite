// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/services/utility/host_runtime.h"

#include <algorithm>
#include <memory>
#include <optional>
#include <utility>

#include "base/compiler_specific.h"
#include "base/containers/span.h"
#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/task/sequenced_task_runner.h"
#include "base/task/thread_pool.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
namespace tuffite {
namespace {
struct ModuleResolutionReply {
  UtilityResponseCallback callback;
  scoped_refptr<base::SequencedTaskRunner> runner;
  std::optional<base::FilePath> asset_path;
};

void ReadModuleAsset(std::optional<base::FilePath> path,
                     UtilityResponseCallback callback) {
  base::ThreadPool::PostTaskAndReplyWithResult(
      FROM_HERE, {base::MayBlock()},
      base::BindOnce(
          [](std::optional<base::FilePath> path) -> std::optional<std::string> {
            std::string source;
            if (!path ||
                !base::ReadFileToStringWithMaxSize(*path, &source,
                                                   512 * 1024) ||
                source.empty()) {
              return std::nullopt;
            }
            return source;
          },
          std::move(path)),
      base::BindOnce(
          [](UtilityResponseCallback callback,
             std::optional<std::string> source) {
            std::vector<uint8_t> bytes;
            if (source) {
              bytes.assign(source->begin(), source->end());
            }
            std::move(callback).Run(
                source ? TUFFITE_STATUS_OK : TUFFITE_STATUS_INVALID_ARGUMENT,
                std::move(bytes));
          },
          std::move(callback)));
}

tuffite_status_t CaptureModuleResolution(void* data,
                                         tuffite_status_t status,
                                         tuffite_byte_slice_t source) {
  std::unique_ptr<ModuleResolutionReply> pending(
      static_cast<ModuleResolutionReply*>(data));
  std::vector<uint8_t> bytes;
  if (status == TUFFITE_STATUS_OK && source.data && source.len > 0 &&
      source.len <= 512 * 1024) {
    // SAFETY: The native ABI producer supplies the declared readable byte range
    // for this callback. Copy it before returning; never retain the raw
    // pointer.
    auto view = UNSAFE_BUFFERS(base::span(source.data, source.len));
    bytes.assign(view.begin(), view.end());
  } else if (status == TUFFITE_STATUS_OK) {
    status = TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  pending->runner->PostTask(
      FROM_HERE, base::BindOnce(
                     [](UtilityResponseCallback callback,
                        std::optional<base::FilePath> path,
                        tuffite_status_t status, std::vector<uint8_t> bytes) {
                       if (status == TUFFITE_STATUS_NOT_FOUND) {
                         ReadModuleAsset(std::move(path), std::move(callback));
                       } else {
                         std::move(callback).Run(status, std::move(bytes));
                       }
                     },
                     std::move(pending->callback),
                     std::move(pending->asset_path), status, std::move(bytes)));
  return TUFFITE_STATUS_OK;
}

}  // namespace
void ConfigureUtilityHostRuntime(UtilityService& service,
                                 const ManifestService& manifest,
                                 const tuffite_application& application,
                                 tuffite_handle_t runtime) {
  service.SetUtilityStartupConfigurations(manifest.UtilityConfigurations());
  if (application.resolve_utility_route) {
    service.SetUtilityInterfaceResolver(base::BindRepeating(
        [](tuffite_resolve_utility_route_fn resolve,
           std::string_view name) -> std::string {
          tuffite_byte_slice_t owner{};
          if (resolve(
                  {reinterpret_cast<const uint8_t*>(name.data()), name.size()},
                  &owner) != TUFFITE_STATUS_OK ||
              !owner.data || owner.len == 0 || owner.len > 128) {
            return {};
          }
          // SAFETY: The ABI producer returns application-lifetime readable
          // bytes.
          const auto bytes = UNSAFE_BUFFERS(base::span(owner.data, owner.len));
          if (std::ranges::find(bytes, uint8_t{0}) != bytes.end()) {
            return {};
          }
          return std::string(bytes.begin(), bytes.end());
        },
        application.resolve_utility_route));
  }
  service.SetUtilityModuleResolver(base::BindRepeating(
      [](asset::AssetConfiguration assets, tuffite_resolve_v8_fn resolve,
         tuffite_handle_t runtime, std::string name, std::string url,
         UtilityResponseCallback callback) {
        if (!resolve) {
          ReadModuleAsset(assets.ResolveAssetFile(GURL(url)),
                          std::move(callback));
          return;
        }
        auto pending =
            std::make_unique<ModuleResolutionReply>(ModuleResolutionReply{
                std::move(callback),
                base::SequencedTaskRunner::GetCurrentDefault(),
                assets.ResolveAssetFile(GURL(url))});
        // Release before calling: an accepted reply may arrive
        // immediately on a worker.
        auto* context = pending.release();
        const auto status = resolve(
            runtime,
            {reinterpret_cast<const uint8_t*>(name.data()), name.size()},
            {reinterpret_cast<const uint8_t*>(url.data()), url.size()}, context,
            &CaptureModuleResolution);
        if (status != TUFFITE_STATUS_OK) {
          pending.reset(context);
          std::move(pending->callback).Run(status, {});
        }
      },
      manifest.assets(), application.resolve_v8, runtime));
  const tuffite_utility_is_registered_fn utility_is_registered =
      application.utility.is_registered;
  service.SetUtilityRegistrationChecker(base::BindRepeating(
      [](tuffite_utility_is_registered_fn checker, std::string_view name) {
        return checker && checker(reinterpret_cast<const uint8_t*>(name.data()),
                                  name.size()) == TUFFITE_STATUS_OK;
      },
      utility_is_registered));
}
}  // namespace tuffite
