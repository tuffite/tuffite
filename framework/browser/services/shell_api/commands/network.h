// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_NETWORK_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_NETWORK_H_

#include "tuffite/framework/shell_api/network.mojom.h"

namespace tuffite::shell_api::net {

class NetImpl final : public generated::Net {
 public:
  void Request(const dispatch::Context& context,
               generated::Net::RequestRequestPtr request,
               generated::Net::RequestCallback callback) override;
};

}  // namespace tuffite::shell_api::net

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_NETWORK_H_
