// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/path.h"

#include <utility>

#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"

namespace tuffite::shell_api::path {

using DirectoriesCallback = generated::Path::DirectoriesCallback;
using DirectoriesRequestPtr = generated::Path::DirectoriesRequestPtr;
using DirectoriesResult = generated::Path::DirectoriesResult;
using ResolveCallback = generated::Path::ResolveCallback;
using ResolveRequestPtr = generated::Path::ResolveRequestPtr;

void PathImpl::Directories(const dispatch::Context& context,
                           DirectoriesRequestPtr,
                           DirectoriesCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (!IsUnscopedAllowed(context, generated::Path::kDirectories)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  const ManifestService* manifest = ManifestFor(context.frame());
  if (!manifest) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
    return;
  }
  auto result = generated::PathDirectories::New();
  const auto resource = manifest->ResolveBaseDirectory("resource");
  const auto data = manifest->ResolveBaseDirectory("data");
  const auto cache = manifest->ResolveBaseDirectory("cache");
  const auto temp = manifest->ResolveBaseDirectory("temp");
  if (!resource || !data || !cache || !temp) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
    return;
  }
  result->resource = resource->AsUTF8Unsafe();
  result->data = data->AsUTF8Unsafe();
  result->cache = cache->AsUTF8Unsafe();
  result->temp = temp->AsUTF8Unsafe();
  std::move(callback).Run(rpc::Status::kOk, std::move(result));
}

void PathImpl::Resolve(const dispatch::Context& context,
                       ResolveRequestPtr request,
                       ResolveCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  const ManifestService* manifest = ManifestFor(context.frame());
  if (!manifest) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
    return;
  }
  auto resolved = manifest->ResolveFileSystemPath(
      context.sender()->GetLastCommittedOrigin(),
      BaseDirectoryName(request->base), request->relative.value_or("."), false,
      "tuffite.Path.Resolve");
  if (!resolved) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  std::move(callback).Run(rpc::Status::kOk, resolved->AsUTF8Unsafe());
}

}  // namespace tuffite::shell_api::path
