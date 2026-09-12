// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_LOG_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_LOG_H_

#include "tuffite/framework/shell_api/log.h"

namespace tuffite::shell_api::log {

void Trace(generated::log::TraceRequest&& request,
           generated::log::TraceCallback&& callback);
void Debug(generated::log::DebugRequest&& request,
           generated::log::DebugCallback&& callback);
void Info(generated::log::InfoRequest&& request,
          generated::log::InfoCallback&& callback);
void Warn(generated::log::WarnRequest&& request,
          generated::log::WarnCallback&& callback);
void Error(generated::log::ErrorRequest&& request,
           generated::log::ErrorCallback&& callback);

}  // namespace tuffite::shell_api::log

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_LOG_H_
