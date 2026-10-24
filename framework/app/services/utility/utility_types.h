// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_TYPES_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_TYPES_H_
#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

#include "base/functional/callback.h"
namespace tuffite {
using UtilityResponseCallback =
    base::OnceCallback<void(int32_t, std::vector<uint8_t>)>;
using UtilitySessionEventCallback =
    base::RepeatingCallback<bool(uint64_t, uint32_t, std::vector<uint8_t>)>;
using UtilitySessionClosedCallback =
    base::OnceCallback<void(int32_t, uint64_t, std::vector<uint8_t>)>;
using UtilityModuleResolver = base::RepeatingCallback<
    void(std::string, std::string, UtilityResponseCallback)>;
using UtilityRegistrationChecker =
    base::RepeatingCallback<bool(std::string_view)>;
using UtilityInterfaceResolver =
    base::RepeatingCallback<std::string(std::string_view)>;

}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_TYPES_H_
