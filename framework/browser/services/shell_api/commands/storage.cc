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
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"
#include "tuffite/framework/common/access.h"

namespace tuffite::shell_api::storage {

using ClearCallback = generated::Storage::ClearCallback;
using ClearRequestPtr = generated::Storage::ClearRequestPtr;
using GetCallback = generated::Storage::GetCallback;
using GetRequestPtr = generated::Storage::GetRequestPtr;
using GetResult = generated::Storage::GetResult;
using RemoveCallback = generated::Storage::RemoveCallback;
using RemoveRequestPtr = generated::Storage::RemoveRequestPtr;
using SetCallback = generated::Storage::SetCallback;
using SetRequestPtr = generated::Storage::SetRequestPtr;

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
  const ManifestService* manifest = ManifestFor(frame);
  auto data = manifest ? manifest->ResolveBaseDirectory("data") : std::nullopt;
  return data ? std::optional(data->AppendASCII("storage")) : std::nullopt;
}

base::FilePath StorageFile(const base::FilePath& root, std::string_view key) {
  return root.AppendASCII(std::string(key) + ".bin");
}

}  // namespace

void StorageImpl::Get(const dispatch::Context& context,
                      GetRequestPtr request,
                      GetCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (!IsUnscopedAllowed(context, generated::Storage::kGet)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  auto root = StorageRoot(context.frame());
  if (!root) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
    return;
  }
  if (!IsKey(request->key)) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  const auto file = StorageFile(*root, request->key);
  StorageTaskRunner()->PostTaskAndReplyWithResult(
      FROM_HERE,
      base::BindOnce(
          [](base::FilePath root,
             base::FilePath file) -> std::optional<GetResult> {
            if (!access::HasNoSymlinkComponents(root, file)) {
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
              rpc::Reject(callback, rpc::Status::kUnavailable);
              return;
            }
            std::move(callback).Run(rpc::Status::kOk, std::move(*result));
          },
          std::move(callback)));
}

void StorageImpl::Set(const dispatch::Context& context,
                      SetRequestPtr request,
                      SetCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (!IsUnscopedAllowed(context, generated::Storage::kSet)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  auto root = StorageRoot(context.frame());
  if (!root) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
    return;
  }
  if (!IsKey(request->key) || request->value.size() > kMaxValueBytes) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  const auto file = StorageFile(*root, request->key);
  StorageTaskRunner()->PostTaskAndReplyWithResult(
      FROM_HERE,
      base::BindOnce(
          [](base::FilePath root, base::FilePath file,
             std::vector<uint8_t> value) {
            return access::HasNoSymlinkComponents(root, file) &&
                   base::CreateDirectory(root) &&
                   base::ImportantFileWriter::WriteFileAtomically(
                       file, base::as_string_view(value), "TuffiteStorage");
          },
          *root, file, std::move(request->value)),
      base::BindOnce(
          [](SetCallback callback, bool ok) {
            std::move(callback).Run(ok ? rpc::Status::kOk
                                       : rpc::Status::kUnavailable);
          },
          std::move(callback)));
}

void StorageImpl::Remove(const dispatch::Context& context,
                         RemoveRequestPtr request,
                         RemoveCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (!IsUnscopedAllowed(context, generated::Storage::kRemove)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  auto root = StorageRoot(context.frame());
  if (!root) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
    return;
  }
  if (!IsKey(request->key)) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  const auto file = StorageFile(*root, request->key);
  StorageTaskRunner()->PostTaskAndReplyWithResult(
      FROM_HERE,
      base::BindOnce(
          [](base::FilePath root, base::FilePath file) {
            return access::HasNoSymlinkComponents(root, file) &&
                   (!base::PathExists(file) || base::DeleteFile(file));
          },
          *root, file),
      base::BindOnce(
          [](RemoveCallback callback, bool ok) {
            std::move(callback).Run(ok ? rpc::Status::kOk
                                       : rpc::Status::kUnavailable);
          },
          std::move(callback)));
}

void StorageImpl::Clear(const dispatch::Context& context,
                        ClearRequestPtr,
                        ClearCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (!IsUnscopedAllowed(context, generated::Storage::kClear)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  auto root = StorageRoot(context.frame());
  if (!root) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
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
            std::move(callback).Run(ok ? rpc::Status::kOk
                                       : rpc::Status::kUnavailable);
          },
          std::move(callback)));
}

}  // namespace tuffite::shell_api::storage
