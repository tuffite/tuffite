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
#include "tuffite/framework/common/access.h"

namespace tuffite::shell_api::fs {

using CreateDirCallback = generated::Fs::CreateDirCallback;
using CreateDirRequestPtr = generated::Fs::CreateDirRequestPtr;
using ExistsCallback = generated::Fs::ExistsCallback;
using ExistsRequestPtr = generated::Fs::ExistsRequestPtr;
using ReadFileCallback = generated::Fs::ReadFileCallback;
using ReadFileRequestPtr = generated::Fs::ReadFileRequestPtr;
using RemoveFileCallback = generated::Fs::RemoveFileCallback;
using RemoveFileRequestPtr = generated::Fs::RemoveFileRequestPtr;
using WriteFileCallback = generated::Fs::WriteFileCallback;
using WriteFileRequestPtr = generated::Fs::WriteFileRequestPtr;

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
            std::move(callback).Run(ok ? rpc::Status::kOk
                                       : rpc::Status::kUnavailable);
          },
          std::move(callback)));
}

}  // namespace

void FsImpl::ReadFile(const dispatch::Context& context,
                      ReadFileRequestPtr request,
                      ReadFileCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  auto path = Resolve(context.frame(), BaseDirectoryName(request->base),
                      request->path, false, "tuffite.Fs.ReadFile");
  if (!path) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
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
              rpc::Reject(callback, rpc::Status::kUnavailable);
              return;
            }
            std::move(callback).Run(rpc::Status::kOk, std::move(*contents));
          },
          std::move(callback)));
}

void FsImpl::WriteFile(const dispatch::Context& context,
                       WriteFileRequestPtr request,
                       WriteFileCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (request->contents.size() > kMaxFileBytes) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  auto path = Resolve(context.frame(), BaseDirectoryName(request->base),
                      request->path, true, "tuffite.Fs.WriteFile");
  if (!path) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  auto contents = std::move(request->contents);
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

void FsImpl::Exists(const dispatch::Context& context,
                    ExistsRequestPtr request,
                    ExistsCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  auto path = Resolve(context.frame(), BaseDirectoryName(request->base),
                      request->path, false, "tuffite.Fs.Exists");
  if (!path) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
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
              std::move(callback).Run(rpc::Status::kUnavailable, false);
              return;
            }
            std::move(callback).Run(rpc::Status::kOk, *exists);
          },
          std::move(callback)));
}

void FsImpl::CreateDir(const dispatch::Context& context,
                       CreateDirRequestPtr request,
                       CreateDirCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  auto path = Resolve(context.frame(), BaseDirectoryName(request->base),
                      request->path, true, "tuffite.Fs.CreateDir");
  if (!path) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  RunBoolean(std::move(*path), base::BindOnce([](const base::FilePath& path) {
               return base::CreateDirectory(path);
             }),
             std::move(callback));
}

void FsImpl::RemoveFile(const dispatch::Context& context,
                        RemoveFileRequestPtr request,
                        RemoveFileCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  auto path = Resolve(context.frame(), BaseDirectoryName(request->base),
                      request->path, true, "tuffite.Fs.RemoveFile");
  if (!path) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  RunBoolean(std::move(*path), base::BindOnce([](const base::FilePath& path) {
               return base::DeleteFile(path);
             }),
             std::move(callback));
}

}  // namespace tuffite::shell_api::fs
