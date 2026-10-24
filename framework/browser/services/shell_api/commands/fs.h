// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_FS_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_FS_H_

#include "tuffite/framework/shell_api/fs.mojom.h"

namespace tuffite::shell_api::fs {

class FsImpl final : public generated::Fs {
 public:
  void ReadFile(const dispatch::Context& context,
                generated::Fs::ReadFileRequestPtr request,
                generated::Fs::ReadFileCallback callback) override;
  void WriteFile(const dispatch::Context& context,
                 generated::Fs::WriteFileRequestPtr request,
                 generated::Fs::WriteFileCallback callback) override;
  void Exists(const dispatch::Context& context,
              generated::Fs::ExistsRequestPtr request,
              generated::Fs::ExistsCallback callback) override;
  void CreateDir(const dispatch::Context& context,
                 generated::Fs::CreateDirRequestPtr request,
                 generated::Fs::CreateDirCallback callback) override;
  void RemoveFile(const dispatch::Context& context,
                  generated::Fs::RemoveFileRequestPtr request,
                  generated::Fs::RemoveFileCallback callback) override;
};

}  // namespace tuffite::shell_api::fs

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_FS_H_
