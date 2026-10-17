// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/fs.h"

#include <optional>
#include <utility>

#include "base/files/file_util.h"
#include "base/files/important_file_writer.h"
#include "base/functional/bind.h"
#include "base/strings/string_view_util.h"
#include "base/task/thread_pool.h"
#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
#include "tuffite/framework/common/access.h"

namespace tuffite::shell_api::fs {

using generated::ToString;
using generated::fs::CreateDirCallback;
using generated::fs::CreateDirRequest;
using generated::fs::ExistsCallback;
using generated::fs::ExistsRequest;
using generated::fs::ReadFileCallback;
using generated::fs::ReadFileRequest;
using generated::fs::RemoveFileCallback;
using generated::fs::RemoveFileRequest;
using generated::fs::WriteFileCallback;
using generated::fs::WriteFileRequest;

namespace {

constexpr size_t kMaxFileBytes = 16 * 1024 * 1024;

struct ResolvedPath {
  base::FilePath root;
  base::FilePath path;
};

std::optional<ResolvedPath> Resolve(content::RenderFrameHost& frame,
                                    std::string_view base,
                                    std::string_view relative,
                                    bool write,
                                    std::string_view method) {
  const ManifestService* manifest = ManifestFor(frame);
  auto path =
      manifest ? manifest->ResolveFileSystemPath(frame.GetLastCommittedOrigin(),
                                                 base, relative, write, method)
               : std::nullopt;
  auto root = manifest ? manifest->ResolveBaseDirectory(base) : std::nullopt;
  if (!path || !root) {
    return std::nullopt;
  }
  return ResolvedPath{*root, *path};
}

template <typename Callback, typename Work>
void RunBoolean(ResolvedPath path, Work work, Callback callback) {
  base::ThreadPool::PostTaskAndReplyWithResult(
      FROM_HERE,
      {base::MayBlock(), base::TaskShutdownBehavior::SKIP_ON_SHUTDOWN},
      base::BindOnce(
          [](ResolvedPath path, Work work) {
            return access::HasNoSymlinkComponents(path.root, path.path) &&
                   std::move(work).Run(path.path);
          },
          std::move(path), std::move(work)),
      base::BindOnce(
          [](Callback callback, bool ok) {
            std::move(callback).Run(ok ? kStatusOk : kStatusUnavailable, {});
          },
          std::move(callback)));
}

}  // namespace

void ReadFile(const ::tuffite::dispatch::Context& context,
              ReadFileRequest&& request,
              ReadFileCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto path = Resolve(context.frame(), ToString(request.base), request.path,
                      false, "tuffite.fs.readFile");
  if (!path) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  base::ThreadPool::PostTaskAndReplyWithResult(
      FROM_HERE,
      {base::MayBlock(), base::TaskShutdownBehavior::SKIP_ON_SHUTDOWN},
      base::BindOnce(
          [](ResolvedPath path) -> std::optional<std::vector<uint8_t>> {
            if (!access::HasNoSymlinkComponents(path.root, path.path)) {
              return std::nullopt;
            }
            const auto size = base::GetFileSize(path.path);
            if (!size || *size < 0 ||
                static_cast<uint64_t>(*size) > kMaxFileBytes) {
              return std::nullopt;
            }
            return base::ReadFileToBytes(path.path);
          },
          std::move(*path)),
      base::BindOnce(
          [](ReadFileCallback callback,
             std::optional<std::vector<uint8_t>> contents) {
            if (!contents) {
              std::move(callback).Run(kStatusUnavailable, {});
              return;
            }
            std::move(callback).Run(kStatusOk, std::move(*contents));
          },
          std::move(callback)));
}

void WriteFile(const ::tuffite::dispatch::Context& context,
               WriteFileRequest&& request,
               WriteFileCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  if (request.contents.size() > kMaxFileBytes) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto path = Resolve(context.frame(), ToString(request.base), request.path,
                      true, "tuffite.fs.writeFile");
  if (!path) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  auto contents = std::move(request.contents);
  RunBoolean(std::move(*path),
             base::BindOnce(
                 [](std::vector<uint8_t> contents, const base::FilePath& path) {
                   return base::CreateDirectory(path.DirName()) &&
                          base::ImportantFileWriter::WriteFileAtomically(
                              path, base::as_string_view(contents),
                              "TuffiteFs");
                 },
                 std::move(contents)),
             std::move(callback));
}

void Exists(const ::tuffite::dispatch::Context& context,
            ExistsRequest&& request,
            ExistsCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto path = Resolve(context.frame(), ToString(request.base), request.path,
                      false, "tuffite.fs.exists");
  if (!path) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  base::ThreadPool::PostTaskAndReplyWithResult(
      FROM_HERE,
      {base::MayBlock(), base::TaskShutdownBehavior::SKIP_ON_SHUTDOWN},
      base::BindOnce(
          [](ResolvedPath path) -> std::optional<bool> {
            if (!access::HasNoSymlinkComponents(path.root, path.path)) {
              return std::nullopt;
            }
            return base::PathExists(path.path);
          },
          std::move(*path)),
      base::BindOnce(
          [](ExistsCallback callback, std::optional<bool> exists) {
            if (!exists) {
              std::move(callback).Run(kStatusUnavailable, false);
              return;
            }
            std::move(callback).Run(kStatusOk, *exists);
          },
          std::move(callback)));
}

void CreateDir(const ::tuffite::dispatch::Context& context,
               CreateDirRequest&& request,
               CreateDirCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto path = Resolve(context.frame(), ToString(request.base), request.path,
                      true, "tuffite.fs.createDir");
  if (!path) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  RunBoolean(std::move(*path), base::BindOnce([](const base::FilePath& path) {
               return base::CreateDirectory(path);
             }),
             std::move(callback));
}

void RemoveFile(const ::tuffite::dispatch::Context& context,
                RemoveFileRequest&& request,
                RemoveFileCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto path = Resolve(context.frame(), ToString(request.base), request.path,
                      true, "tuffite.fs.removeFile");
  if (!path) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  RunBoolean(std::move(*path), base::BindOnce([](const base::FilePath& path) {
               return base::DeleteFile(path);
             }),
             std::move(callback));
}

}  // namespace tuffite::shell_api::fs
