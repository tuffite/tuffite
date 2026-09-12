// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_partition.h"

#include "testing/gtest/include/gtest/gtest.h"

namespace tuffite {

TEST(ExplorerPartitionTest, ParsesEverySupportedMode) {
  auto shared = ExplorerPartition::Parse("shared");
  ASSERT_TRUE(shared);
  EXPECT_TRUE(shared->shared());
  EXPECT_EQ(shared->mode_name(), "shared");

  auto memory = ExplorerPartition::Parse("memory:worker_1");
  ASSERT_TRUE(memory);
  EXPECT_TRUE(memory->in_memory());
  EXPECT_FALSE(memory->inherit);
  EXPECT_EQ(memory->name, "worker_1");

  auto inherited = ExplorerPartition::Parse("inherit+persist:profile.main");
  ASSERT_TRUE(inherited);
  EXPECT_FALSE(inherited->in_memory());
  EXPECT_TRUE(inherited->inherit);
  EXPECT_EQ(inherited->mode_name(), "inherit+persist");
}

TEST(ExplorerPartitionTest, RejectsMalformedOrUnknownPartitions) {
  EXPECT_FALSE(ExplorerPartition::Parse(""));
  EXPECT_FALSE(ExplorerPartition::Parse("memory:"));
  EXPECT_FALSE(ExplorerPartition::Parse("persist:bad/name"));
  EXPECT_FALSE(ExplorerPartition::Parse("inherit:profile"));
  EXPECT_FALSE(
      ExplorerPartition::Parse(std::string("memory:") + std::string(129, 'a')));
}
}  // namespace tuffite
