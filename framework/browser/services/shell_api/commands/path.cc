// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/path.h"

#include <utility>

#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"

namespace tuffite::shell_api::path {

using generated::ToString;
using generated::path::DirectoriesCallback;
using generated::path::DirectoriesRequest;
using generated::path::DirectoriesResult;
using generated::path::ResolveCallback;
using generated::path::ResolveRequest;

void Directories(const ::tuffite::dispatch::Context& context,
                 DirectoriesRequest&&,
                 DirectoriesCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  const ManifestService* manifest = ManifestFor(context.frame());
  if (!manifest) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  DirectoriesResult result;
  const auto resource = manifest->ResolveBaseDirectory("resource");
  const auto data = manifest->ResolveBaseDirectory("data");
  const auto cache = manifest->ResolveBaseDirectory("cache");
  const auto temp = manifest->ResolveBaseDirectory("temp");
  if (!resource || !data || !cache || !temp) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  result.resource = resource->AsUTF8Unsafe();
  result.data = data->AsUTF8Unsafe();
  result.cache = cache->AsUTF8Unsafe();
  result.temp = temp->AsUTF8Unsafe();
  std::move(callback).Run(kStatusOk, std::move(result));
}

void Resolve(const ::tuffite::dispatch::Context& context,
             ResolveRequest&& request,
             ResolveCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  const ManifestService* manifest = ManifestFor(context.frame());
  if (!manifest) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  auto resolved = manifest->ResolveFileSystemPath(
      context.sender->GetLastCommittedOrigin(), ToString(request.base),
      request.relative.value_or("."), false, "tuffite.path.resolve");
  if (!resolved) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  std::move(callback).Run(kStatusOk, resolved->AsUTF8Unsafe());
}

}  // namespace tuffite::shell_api::path
