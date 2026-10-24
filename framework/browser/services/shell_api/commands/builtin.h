// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#pragma once
#include <vector>

#include "tuffite/components/dispatch/browser/rpc.h"
namespace tuffite::shell_api {
std::vector<rpc::Service> BuiltinServices();
}
