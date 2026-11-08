// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_MANIFEST_BOOTSTRAP_PROCESS_CONFIGURATION_H_
#define TUFFITE_COMPONENTS_MANIFEST_BOOTSTRAP_PROCESS_CONFIGURATION_H_

#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "base/component_export.h"
#include "base/memory/read_only_shared_memory_region.h"

namespace tuffite {
inline constexpr char kBootstrapHandleSwitch[] = "tuffite-bootstrap-handle";
inline constexpr uint32_t kBootstrapRendezvousKey = 0x74756662;  // 'tufb'
// First embedder descriptor, following Content's reserved 0..199 range.
inline constexpr int kBootstrapDescriptorKey = 200;

COMPONENT_EXPORT(TUFFITE_MANIFEST_BOOTSTRAP)
base::ReadOnlySharedMemoryRegion EncodeProcessConfiguration(
    const std::vector<std::string>& schemes);
COMPONENT_EXPORT(TUFFITE_MANIFEST_BOOTSTRAP)
std::optional<std::vector<std::string>> DecodeProcessConfiguration(
    const base::ReadOnlySharedMemoryRegion& region);

// Called before URL scheme reg·istration; subsequent initialization must match.
COMPONENT_EXPORT(TUFFITE_MANIFEST_BOOTSTRAP)
bool InitializeProcessConfiguration(std::vector<std::string> schemes);
COMPONENT_EXPORT(TUFFITE_MANIFEST_BOOTSTRAP)
bool InitializeProcessConfigurationFromHandle(std::string_view handle);

COMPONENT_EXPORT(TUFFITE_MANIFEST_BOOTSTRAP)
const std::vector<std::string>& ApplicationSchemes();
COMPONENT_EXPORT(TUFFITE_MANIFEST_BOOTSTRAP)
const base::ReadOnlySharedMemoryRegion& ProcessConfigurationRegion();

}  // namespace tuffite

#endif
