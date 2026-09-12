// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_FS_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_FS_H_

#include "tuffite/framework/shell_api/fs.h"

namespace tuffite::shell_api::fs {

void ReadFile(const ::tuffite::dispatch::Context& context,
              generated::fs::ReadFileRequest&& request,
              generated::fs::ReadFileCallback&& callback);
void WriteFile(const ::tuffite::dispatch::Context& context,
               generated::fs::WriteFileRequest&& request,
               generated::fs::WriteFileCallback&& callback);
void Exists(const ::tuffite::dispatch::Context& context,
            generated::fs::ExistsRequest&& request,
            generated::fs::ExistsCallback&& callback);
void CreateDir(const ::tuffite::dispatch::Context& context,
               generated::fs::CreateDirRequest&& request,
               generated::fs::CreateDirCallback&& callback);
void RemoveFile(const ::tuffite::dispatch::Context& context,
                generated::fs::RemoveFileRequest&& request,
                generated::fs::RemoveFileCallback&& callback);

}  // namespace tuffite::shell_api::fs

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_FS_H_
