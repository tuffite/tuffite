// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_PROCESS_LAUNCH_H_
#define TUFFITE_FRAMEWORK_APP_PROCESS_LAUNCH_H_

#include <string>
#include <vector>

#include "tuffite/framework/public/abi.h"

namespace tuffite {
// Synchronously prepares arguments for every native process role. Legacy
// browser hooks remain browser-only. Failure preserves the original vector.
bool PrepareProcessLaunch(const tuffite_application& application,
                          std::vector<std::string>* args);
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_PROCESS_LAUNCH_H_
