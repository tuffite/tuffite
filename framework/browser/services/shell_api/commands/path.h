// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_PATH_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_PATH_H_

#include "tuffite/framework/shell_api/path.h"

namespace tuffite::shell_api::path {

void Directories(const ::tuffite::dispatch::Context& context,
                 generated::path::DirectoriesRequest&& request,
                 generated::path::DirectoriesCallback&& callback);
void Resolve(const ::tuffite::dispatch::Context& context,
             generated::path::ResolveRequest&& request,
             generated::path::ResolveCallback&& callback);

}  // namespace tuffite::shell_api::path

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_PATH_H_
