// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_ISOLATED_ISOLATED_ABI_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_ISOLATED_ISOLATED_ABI_H_

#include "tuffite/framework/public/abi.h"

namespace tuffite {
class AbiHostBridge;
tuffite_isolated MakeIsolatedAbi(AbiHostBridge& bridge);
}  // namespace tuffite
#endif
