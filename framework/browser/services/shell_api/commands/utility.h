// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_UTILITY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_UTILITY_H_

#include "tuffite/components/dispatch/browser/context.h"
#include "tuffite/framework/shell_api/utility.h"

namespace tuffite::shell_api::utility {

void Invoke(const ::tuffite::dispatch::Context& context,
            generated::utility::InvokeRequest&& request,
            generated::utility::InvokeCallback&& callback);
void Cancel(const ::tuffite::dispatch::Context& context,
            generated::utility::CancelRequest&& request,
            generated::utility::CancelCallback&& callback);

}  // namespace tuffite::shell_api::utility

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_UTILITY_H_
