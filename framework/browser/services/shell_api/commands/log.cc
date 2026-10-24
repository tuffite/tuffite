// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/log.h"

#include <string>
#include <utility>

#include "base/logging.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"

namespace tuffite::shell_api::log {

using DebugCallback = generated::Log::DebugCallback;
using DebugRequestPtr = generated::Log::DebugRequestPtr;
using ErrorCallback = generated::Log::ErrorCallback;
using ErrorRequestPtr = generated::Log::ErrorRequestPtr;
using InfoCallback = generated::Log::InfoCallback;
using InfoRequestPtr = generated::Log::InfoRequestPtr;
using TraceCallback = generated::Log::TraceCallback;
using TraceRequestPtr = generated::Log::TraceRequestPtr;
using WarnCallback = generated::Log::WarnCallback;
using WarnRequestPtr = generated::Log::WarnRequestPtr;

namespace {

enum class Level { kTrace, kDebug, kInfo, kWarn, kError };

template <typename Request, typename Callback>
void WriteLog(Level level, Request&& request, Callback&& callback) {
  if (request->message.size() > 16 * 1024) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  const std::string text = "[renderer] " + request->message;
  switch (level) {
    case Level::kTrace:
      VLOG(2) << text;
      break;
    case Level::kDebug:
      VLOG(1) << text;
      break;
    case Level::kInfo:
      LOG(INFO) << text;
      break;
    case Level::kWarn:
      LOG(WARNING) << text;
      break;
    case Level::kError:
      LOG(ERROR) << text;
      break;
  }
  std::move(callback).Run(rpc::Status::kOk);
}

}  // namespace

void LogImpl::Trace(const dispatch::Context& context,
                    TraceRequestPtr request,
                    TraceCallback callback) {
  if (!IsUnscopedAllowed(context, generated::Log::kTrace)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  WriteLog(Level::kTrace, std::move(request), std::move(callback));
}
void LogImpl::Debug(const dispatch::Context& context,
                    DebugRequestPtr request,
                    DebugCallback callback) {
  if (!IsUnscopedAllowed(context, generated::Log::kDebug)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  WriteLog(Level::kDebug, std::move(request), std::move(callback));
}
void LogImpl::Info(const dispatch::Context& context,
                   InfoRequestPtr request,
                   InfoCallback callback) {
  if (!IsUnscopedAllowed(context, generated::Log::kInfo)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  WriteLog(Level::kInfo, std::move(request), std::move(callback));
}
void LogImpl::Warn(const dispatch::Context& context,
                   WarnRequestPtr request,
                   WarnCallback callback) {
  if (!IsUnscopedAllowed(context, generated::Log::kWarn)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  WriteLog(Level::kWarn, std::move(request), std::move(callback));
}
void LogImpl::Error(const dispatch::Context& context,
                    ErrorRequestPtr request,
                    ErrorCallback callback) {
  if (!IsUnscopedAllowed(context, generated::Log::kError)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  WriteLog(Level::kError, std::move(request), std::move(callback));
}

}  // namespace tuffite::shell_api::log
