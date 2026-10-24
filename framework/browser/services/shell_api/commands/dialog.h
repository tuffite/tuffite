// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_DIALOG_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_DIALOG_H_

#include "tuffite/framework/shell_api/dialog.mojom.h"

namespace tuffite::shell_api::dialog {

class DialogImpl final : public generated::Dialog {
 public:
  void OpenFile(const dispatch::Context& context,
                generated::Dialog::OpenFileRequestPtr request,
                generated::Dialog::OpenFileCallback callback) override;
  void OpenDirectory(
      const dispatch::Context& context,
      generated::Dialog::OpenDirectoryRequestPtr request,
      generated::Dialog::OpenDirectoryCallback callback) override;
};

}  // namespace tuffite::shell_api::dialog

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_DIALOG_H_
