// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_PROCESS_LAUNCHER_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_PROCESS_LAUNCHER_H_
#include <memory>
#include <string_view>

#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/framework/common/utility/utility_service.mojom.h"
namespace tuffite {
// The supervisor owns scheduling/restart policy; this port only creates a pipe.
// Tests can supply an in-process Mojo receiver without launching a child.
class UtilityProcessLauncher {
 public:
  virtual ~UtilityProcessLauncher() = default;
  virtual mojo::Remote<mojom::UtilityService> Launch(std::string_view name) = 0;
};
std::unique_ptr<UtilityProcessLauncher> CreateUtilityProcessLauncher();
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_PROCESS_LAUNCHER_H_
