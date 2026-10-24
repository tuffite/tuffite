// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_LOG_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_LOG_H_

#include "tuffite/framework/shell_api/log.mojom.h"

namespace tuffite::shell_api::log {

class LogImpl final : public generated::Log {
 public:
  void Trace(const dispatch::Context& context,
             generated::Log::TraceRequestPtr request,
             generated::Log::TraceCallback callback) override;
  void Debug(const dispatch::Context& context,
             generated::Log::DebugRequestPtr request,
             generated::Log::DebugCallback callback) override;
  void Info(const dispatch::Context& context,
            generated::Log::InfoRequestPtr request,
            generated::Log::InfoCallback callback) override;
  void Warn(const dispatch::Context& context,
            generated::Log::WarnRequestPtr request,
            generated::Log::WarnCallback callback) override;
  void Error(const dispatch::Context& context,
             generated::Log::ErrorRequestPtr request,
             generated::Log::ErrorCallback callback) override;
};

}  // namespace tuffite::shell_api::log

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_LOG_H_
