// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_STORAGE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_STORAGE_H_

#include "tuffite/framework/shell_api/storage.h"

namespace tuffite::shell_api::storage {

void Get(const ::tuffite::dispatch::Context& context,
         generated::storage::GetRequest&& request,
         generated::storage::GetCallback&& callback);
void Set(const ::tuffite::dispatch::Context& context,
         generated::storage::SetRequest&& request,
         generated::storage::SetCallback&& callback);
void Remove(const ::tuffite::dispatch::Context& context,
            generated::storage::RemoveRequest&& request,
            generated::storage::RemoveCallback&& callback);
void Clear(const ::tuffite::dispatch::Context& context,
           generated::storage::ClearRequest&& request,
           generated::storage::ClearCallback&& callback);

}  // namespace tuffite::shell_api::storage

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_STORAGE_H_
