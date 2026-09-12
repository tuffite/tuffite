// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/log.h"

#include <string>
#include <utility>

#include "base/logging.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"

namespace tuffite::shell_api::log {

using generated::log::DebugCallback;
using generated::log::DebugRequest;
using generated::log::ErrorCallback;
using generated::log::ErrorRequest;
using generated::log::InfoCallback;
using generated::log::InfoRequest;
using generated::log::TraceCallback;
using generated::log::TraceRequest;
using generated::log::WarnCallback;
using generated::log::WarnRequest;

namespace {

enum class Level { kTrace, kDebug, kInfo, kWarn, kError };

template <typename Request, typename Callback>
void WriteLog(Level level, Request&& request, Callback&& callback) {
  if (request.message.size() > 16 * 1024) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  const std::string text = "[renderer] " + request.message;
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
  std::move(callback).Run(kStatusOk, {});
}

}  // namespace

void Trace(TraceRequest&& request, TraceCallback&& callback) {
  WriteLog(Level::kTrace, std::move(request), std::move(callback));
}
void Debug(DebugRequest&& request, DebugCallback&& callback) {
  WriteLog(Level::kDebug, std::move(request), std::move(callback));
}
void Info(InfoRequest&& request, InfoCallback&& callback) {
  WriteLog(Level::kInfo, std::move(request), std::move(callback));
}
void Warn(WarnRequest&& request, WarnCallback&& callback) {
  WriteLog(Level::kWarn, std::move(request), std::move(callback));
}
void Error(ErrorRequest&& request, ErrorCallback&& callback) {
  WriteLog(Level::kError, std::move(request), std::move(callback));
}

}  // namespace tuffite::shell_api::log
