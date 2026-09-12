// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_HOST_RUNTIME_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_HOST_RUNTIME_H_
#include "tuffite/framework/public/abi.h"
namespace tuffite {
class AppManifest;
class UtilityService;
// Configure service-owned host module resolution after the ABI runtime starts.
void ConfigureUtilityHostRuntime(UtilityService& service,
                                 const AppManifest& manifest,
                                 const tuffite_application& application,
                                 tuffite_handle_t runtime);
}  // namespace tuffite
#endif
