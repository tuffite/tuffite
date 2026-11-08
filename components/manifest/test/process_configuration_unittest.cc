// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/manifest/bootstrap/process_configuration.h"

#include <algorithm>

#include "base/pickle.h"
#include "testing/gtest/include/gtest/gtest.h"

namespace tuffite {
TEST(ProcessConfigurationTest, RoundTripsLargeBusinessSchemeList) {
  std::vector<std::string> schemes;
  for (int i = 0; i < 4096; ++i) {
    schemes.push_back("business-" + std::to_string(i));
  }
  auto region = EncodeProcessConfiguration(schemes);
  ASSERT_TRUE(region.IsValid());
  auto decoded = DecodeProcessConfiguration(region);
  ASSERT_TRUE(decoded);
  EXPECT_EQ(schemes, *decoded);
}

TEST(ProcessConfigurationTest, RejectsInvalidOrOversizedConfiguration) {
  for (const auto* scheme :
       {"", "1app", "APP", "app://", "app name", "app,other"}) {
    EXPECT_FALSE(EncodeProcessConfiguration({scheme}).IsValid());
  }
  EXPECT_FALSE(EncodeProcessConfiguration(std::vector<std::string>(4097, "app"))
                   .IsValid());
  EXPECT_FALSE(EncodeProcessConfiguration({std::string(1025, 'a')}).IsValid());
  EXPECT_FALSE(EncodeProcessConfiguration({"app", "app"}).IsValid());
  EXPECT_FALSE(DecodeProcessConfiguration(base::ReadOnlySharedMemoryRegion()));
  auto oversized = base::ReadOnlySharedMemoryRegion::Create(1024 * 1024 + 1);
  EXPECT_FALSE(DecodeProcessConfiguration(oversized.region));
}

TEST(ProcessConfigurationTest, RejectsMalformedBinaryInput) {
  auto memory = base::ReadOnlySharedMemoryRegion::Create(32);
  ASSERT_TRUE(memory.IsValid());
  std::ranges::fill(memory.mapping.GetMemoryAsSpan<uint8_t>(), 0);
  EXPECT_FALSE(DecodeProcessConfiguration(memory.region));
  base::Pickle extra_header(8);
  extra_header.WriteUInt32(0x54554642);
  extra_header.WriteUInt32(1);
  extra_header.WriteUInt32(0);
  auto nonstandard =
      base::ReadOnlySharedMemoryRegion::Create(extra_header.size());
  ASSERT_TRUE(nonstandard.IsValid());
  nonstandard.mapping.GetMemoryAsSpan<uint8_t>().copy_from(
      extra_header.AsBytes());
  EXPECT_FALSE(DecodeProcessConfiguration(nonstandard.region));
  auto short_memory = base::ReadOnlySharedMemoryRegion::Create(1);
  ASSERT_TRUE(short_memory.IsValid());
  EXPECT_FALSE(DecodeProcessConfiguration(short_memory.region));
}

TEST(ProcessConfigurationTest, SupportsAnEmptySchemeList) {
  auto region = EncodeProcessConfiguration({});
  ASSERT_TRUE(region.IsValid());
  auto decoded = DecodeProcessConfiguration(region);
  ASSERT_TRUE(decoded);
  EXPECT_TRUE(decoded->empty());
}

}  // namespace tuffite
