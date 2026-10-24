// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_UTILITY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_UTILITY_H_

#include "tuffite/components/dispatch/browser/context.h"
#include "tuffite/framework/shell_api/utility.mojom.h"

namespace tuffite::shell_api::utility {

class UtilityImpl final : public generated::Utility {
 public:
  void Invoke(const dispatch::Context& context,
              generated::Utility::InvokeRequestPtr request,
              generated::Utility::InvokeCallback callback) override;
  void Cancel(const dispatch::Context& context,
              generated::Utility::CancelRequestPtr request,
              generated::Utility::CancelCallback callback) override;
};

}  // namespace tuffite::shell_api::utility

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_UTILITY_H_
