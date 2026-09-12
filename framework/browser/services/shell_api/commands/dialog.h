// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_DIALOG_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_DIALOG_H_

#include "tuffite/framework/shell_api/dialog.h"

namespace tuffite::shell_api::dialog {

void OpenFile(const ::tuffite::dispatch::Context& context,
              generated::dialog::OpenFileRequest&& request,
              generated::dialog::OpenFileCallback&& callback);
void OpenDirectory(const ::tuffite::dispatch::Context& context,
                   generated::dialog::OpenDirectoryRequest&& request,
                   generated::dialog::OpenDirectoryCallback&& callback);

}  // namespace tuffite::shell_api::dialog

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_DIALOG_H_
