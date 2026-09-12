// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_H_
#include "base/memory/weak_ptr.h"
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/components/dispatch/browser/rpc_handler.h"
namespace tuffite {
class ShellApiService : public KeyedService, public RpcHandler {
 public:
  virtual base::WeakPtr<ShellApiService> GetWeakPtr() = 0;
  virtual void Broadcast(const std::string&, const std::vector<uint8_t>&) = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_H_
