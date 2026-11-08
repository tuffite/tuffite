// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/manifest/bootstrap/process_configuration.h"

#include <algorithm>
#include <set>
#include <utility>

#include "base/check.h"
#include "base/containers/span.h"
#include "base/memory/shared_memory_switch.h"
#include "base/no_destructor.h"
#include "base/pickle.h"
#include "base/strings/string_util.h"

namespace tuffite {
namespace {
constexpr uint32_t kMagic = 0x54554642;
constexpr uint32_t kVersion = 1;
constexpr size_t kMaxBytes = 1024 * 1024;
constexpr size_t kMaxSchemes = 4096;
constexpr size_t kMaxSchemeLength = 1024;
struct Configuration {
  bool initialized = false;
  std::vector<std::string> schemes;
  base::ReadOnlySharedMemoryRegion region;
};
Configuration& State() {
  static base::NoDestructor<Configuration> state;
  return *state;
}

bool ValidSchemes(const std::vector<std::string>& schemes) {
  if (schemes.size() > kMaxSchemes) {
    return false;
  }
  size_t bytes = 0;
  std::set<std::string_view> seen;
  for (const auto& scheme : schemes) {
    if (scheme.empty() || scheme.size() > kMaxSchemeLength ||
        !base::IsAsciiLower(scheme.front()) ||
        !std::ranges::all_of(scheme, [](char c) {
          return base::IsAsciiLower(c) || base::IsAsciiDigit(c) || c == '+' ||
                 c == '-' || c == '.';
        })) {
      return false;
    }
    if (!seen.emplace(scheme).second) {
      return false;
    }
    bytes += scheme.size() + 8;
    if (bytes > kMaxBytes - 32) {
      return false;
    }
  }
  return true;
}

}  // namespace
base::ReadOnlySharedMemoryRegion EncodeProcessConfiguration(
    const std::vector<std::string>& schemes) {
  if (!ValidSchemes(schemes)) {
    return {};
  }
  base::Pickle data;
  data.WriteUInt32(kMagic);
  data.WriteUInt32(kVersion);
  data.WriteUInt32(static_cast<uint32_t>(schemes.size()));
  for (const auto& scheme : schemes) {
    data.WriteString(scheme);
  }
  if (data.size() > kMaxBytes) {
    return {};
  }
  auto memory = base::ReadOnlySharedMemoryRegion::Create(data.size());
  if (!memory.IsValid()) {
    return {};
  }
  memory.mapping.GetMemoryAsSpan<uint8_t>().copy_from(data.AsBytes());
  return std::move(memory.region);
}

std::optional<std::vector<std::string>> DecodeProcessConfiguration(
    const base::ReadOnlySharedMemoryRegion& region) {
  if (!region.IsValid() || region.GetSize() > kMaxBytes) {
    return std::nullopt;
  }
  auto mapping = region.Map();
  if (!mapping.IsValid()) {
    return std::nullopt;
  }
  auto bytes = mapping.GetMemoryAsSpan<const uint8_t>();
  auto data = base::Pickle::WithUnownedBuffer(bytes);
  if (bytes.size() < sizeof(base::Pickle::Header) ||
      data.size() != bytes.size() ||
      data.payload_size() != bytes.size() - sizeof(base::Pickle::Header)) {
    return std::nullopt;
  }
  base::PickleIterator reader(data);
  uint32_t magic, version, count;
  if (!reader.ReadUInt32(&magic) || magic != kMagic ||
      !reader.ReadUInt32(&version) || version != kVersion ||
      !reader.ReadUInt32(&count) || count > kMaxSchemes) {
    return std::nullopt;
  }
  std::vector<std::string> schemes;
  for (uint32_t i = 0; i < count; ++i) {
    std::string scheme;
    if (!reader.ReadString(&scheme)) {
      return std::nullopt;
    }
    schemes.push_back(std::move(scheme));
  }
  if (!reader.ReachedEnd() || !ValidSchemes(schemes)) {
    return std::nullopt;
  }
  return schemes;
}

bool InitializeProcessConfiguration(std::vector<std::string> schemes) {
  auto& state = State();
  if (state.initialized) {
    return state.schemes == schemes;
  }
  auto region = EncodeProcessConfiguration(schemes);
  if (!region.IsValid()) {
    return false;
  }
  state.schemes = std::move(schemes);
  state.region = std::move(region);
  state.initialized = true;
  return true;
}

bool InitializeProcessConfigurationFromHandle(std::string_view handle) {
  auto region = base::shared_memory::ReadOnlySharedMemoryRegionFrom(handle);
  if (!region.has_value()) {
    return false;
  }
  auto schemes = DecodeProcessConfiguration(*region);
  if (!schemes) {
    return false;
  }
  auto& state = State();
  if (state.initialized) {
    return state.schemes == *schemes;
  }
  state.schemes = std::move(*schemes);
  // Children keep only decoded immutable data; the browser owns the launch
  // region.
  state.initialized = true;
  return true;
}

const std::vector<std::string>& ApplicationSchemes() {
  CHECK(State().initialized);
  return State().schemes;
}

const base::ReadOnlySharedMemoryRegion& ProcessConfigurationRegion() {
  return State().region;
}

}  // namespace tuffite
