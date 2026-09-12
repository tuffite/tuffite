// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/storage.h"

#include <algorithm>
#include <optional>
#include <utility>

#include "base/files/file_util.h"
#include "base/files/important_file_writer.h"
#include "base/functional/bind.h"
#include "base/no_destructor.h"
#include "base/strings/string_util.h"
#include "base/strings/string_view_util.h"
#include "base/task/sequenced_task_runner.h"
#include "base/task/thread_pool.h"
#include "tuffite/framework/browser/services/shell_api/commands/application_policy.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
#include "tuffite/framework/common/app_manifest.h"

namespace tuffite::shell_api::storage {

using generated::storage::ClearCallback;
using generated::storage::ClearRequest;
using generated::storage::GetCallback;
using generated::storage::GetRequest;
using generated::storage::GetResult;
using generated::storage::RemoveCallback;
using generated::storage::RemoveRequest;
using generated::storage::SetCallback;
using generated::storage::SetRequest;

namespace {

constexpr size_t kMaxValueBytes = 4 * 1024 * 1024;

bool IsKey(std::string_view key) {
  return !key.empty() && key.size() <= 128 &&
         std::ranges::all_of(key, [](char character) {
           return base::IsAsciiAlphaNumeric(character) || character == '-' ||
                  character == '_' || character == '.';
         });
}

scoped_refptr<base::SequencedTaskRunner> StorageTaskRunner() {
  static const base::NoDestructor<scoped_refptr<base::SequencedTaskRunner>>
      runner(base::ThreadPool::CreateSequencedTaskRunner(
          {base::MayBlock(), base::TaskPriority::BEST_EFFORT,
           base::TaskShutdownBehavior::BLOCK_SHUTDOWN}));
  return *runner;
}

std::optional<base::FilePath> StorageRoot(content::RenderFrameHost& frame) {
  const AppManifest* manifest = ManifestFor(frame);
  auto data = manifest ? manifest->ResolveBaseDirectory("data") : std::nullopt;
  return data ? std::optional(data->AppendASCII("storage")) : std::nullopt;
}

base::FilePath StorageFile(const base::FilePath& root, std::string_view key) {
  return root.AppendASCII(std::string(key) + ".bin");
}

}  // namespace

void Get(const ::tuffite::dispatch::Context& context,
         GetRequest&& request,
         GetCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto root = StorageRoot(context.frame());
  if (!root) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  if (!IsKey(request.key)) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  const auto file = StorageFile(*root, request.key);
  StorageTaskRunner()->PostTaskAndReplyWithResult(
      FROM_HERE,
      base::BindOnce(
          [](base::FilePath root,
             base::FilePath file) -> std::optional<GetResult> {
            if (!AppManifest::HasNoSymlinkComponents(root, file)) {
              return std::nullopt;
            }
            if (!base::PathExists(file)) {
              return GetResult();
            }
            const auto size = base::GetFileSize(file);
            if (!size || *size < 0 ||
                static_cast<uint64_t>(*size) > kMaxValueBytes) {
              return std::nullopt;
            }
            auto bytes = base::ReadFileToBytes(file);
            return bytes ? std::optional(GetResult(std::move(*bytes)))
                         : std::nullopt;
          },
          *root, file),
      base::BindOnce(
          [](GetCallback callback, std::optional<GetResult> result) {
            if (!result) {
              std::move(callback).Run(kStatusUnavailable, {});
              return;
            }
            std::move(callback).Run(kStatusOk, std::move(*result));
          },
          std::move(callback)));
}

void Set(const ::tuffite::dispatch::Context& context,
         SetRequest&& request,
         SetCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto root = StorageRoot(context.frame());
  if (!root) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  if (!IsKey(request.key) || request.value.size() > kMaxValueBytes) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  const auto file = StorageFile(*root, request.key);
  StorageTaskRunner()->PostTaskAndReplyWithResult(
      FROM_HERE,
      base::BindOnce(
          [](base::FilePath root, base::FilePath file,
             std::vector<uint8_t> value) {
            return AppManifest::HasNoSymlinkComponents(root, file) &&
                   base::CreateDirectory(root) &&
                   base::ImportantFileWriter::WriteFileAtomically(
                       file, base::as_string_view(value), "TuffiteStorage");
          },
          *root, file, std::move(request.value)),
      base::BindOnce(
          [](SetCallback callback, bool ok) {
            std::move(callback).Run(ok ? kStatusOk : kStatusUnavailable, {});
          },
          std::move(callback)));
}

void Remove(const ::tuffite::dispatch::Context& context,
            RemoveRequest&& request,
            RemoveCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto root = StorageRoot(context.frame());
  if (!root) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  if (!IsKey(request.key)) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  const auto file = StorageFile(*root, request.key);
  StorageTaskRunner()->PostTaskAndReplyWithResult(
      FROM_HERE,
      base::BindOnce(
          [](base::FilePath root, base::FilePath file) {
            return AppManifest::HasNoSymlinkComponents(root, file) &&
                   (!base::PathExists(file) || base::DeleteFile(file));
          },
          *root, file),
      base::BindOnce(
          [](RemoveCallback callback, bool ok) {
            std::move(callback).Run(ok ? kStatusOk : kStatusUnavailable, {});
          },
          std::move(callback)));
}

void Clear(const ::tuffite::dispatch::Context& context,
           ClearRequest&&,
           ClearCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto root = StorageRoot(context.frame());
  if (!root) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  StorageTaskRunner()->PostTaskAndReplyWithResult(
      FROM_HERE,
      base::BindOnce(
          [](base::FilePath root) {
            return !base::PathExists(root) || base::DeletePathRecursively(root);
          },
          *root),
      base::BindOnce(
          [](ClearCallback callback, bool ok) {
            std::move(callback).Run(ok ? kStatusOk : kStatusUnavailable, {});
          },
          std::move(callback)));
}

}  // namespace tuffite::shell_api::storage
