// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/request_registry.h"

#include <memory>

#include "base/functional/bind.h"
#include "base/test/bind.h"
#include "testing/gtest/include/gtest/gtest.h"
#include "tuffite/components/dispatch/common/status.h"
namespace tuffite::rpc {
TEST(RequestRegistryTest, SameIdInDifferentConnectionsIsIndependent) {
  RequestRegistry registry;
  auto a = base::UnguessableToken::Create();
  auto b = base::UnguessableToken::Create();
  int replies = 0;
  auto reply = [&replies](int32_t, const std::vector<uint8_t>&) { ++replies; };
  auto first = registry.Admit({a, 1}, base::BindLambdaForTesting(reply), {});
  auto second = registry.Admit({b, 1}, base::BindLambdaForTesting(reply), {});
  ASSERT_TRUE(first);
  ASSERT_TRUE(second);
  EXPECT_EQ(registry.size(), 2u);
  registry.CancelConnection(a);
  EXPECT_EQ(registry.size(), 1u);
  std::move(*second).Run(0, {});
  EXPECT_EQ(replies, 2);
}
TEST(RequestRegistryTest, CancelWinsAndLateCompletionIsIgnored) {
  RequestRegistry registry;
  auto c = base::UnguessableToken::Create();
  int replies = 0;
  bool cancelled = false;
  int32_t status = 0;
  auto completion = registry.Admit(
      {c, 1},
      base::BindLambdaForTesting([&](int32_t s, const std::vector<uint8_t>&) {
        ++replies;
        status = s;
      }),
      base::BindLambdaForTesting([&] { cancelled = true; }));
  ASSERT_TRUE(completion);
  EXPECT_TRUE(registry.Cancel({c, 1}));
  std::move(*completion).Run(0, {});
  EXPECT_TRUE(cancelled);
  EXPECT_EQ(replies, 1);
  EXPECT_EQ(status, static_cast<int32_t>(Status::kCancelled));
}
TEST(RequestRegistryTest, ShutdownRejectsNewWorkAndInvalidatesCompletion) {
  RequestRegistry registry;
  auto c = base::UnguessableToken::Create();
  int replies = 0;
  auto reply = [&](int32_t, const std::vector<uint8_t>&) { ++replies; };
  auto completion =
      registry.Admit({c, 1}, base::BindLambdaForTesting(reply), {});
  registry.Shutdown();
  ASSERT_TRUE(completion);
  std::move(*completion).Run(0, {});
  EXPECT_EQ(replies, 1);
  EXPECT_FALSE(registry.Admit({c, 2}, base::BindLambdaForTesting(reply), {}));
  EXPECT_EQ(replies, 2);
}
TEST(RequestRegistryTest, OwnerDestructionDuringCancelDoesNotUseFreedRegistry) {
  auto registry = std::make_unique<RequestRegistry>();
  auto connection = base::UnguessableToken::Create();
  int replies = 0;
  auto first = registry->Admit(
      {connection, 1},
      base::BindLambdaForTesting([&](int32_t, const std::vector<uint8_t>&) {
        ++replies;
        registry.reset();
      }),
      {});
  auto second = registry->Admit(
      {connection, 2},
      base::BindLambdaForTesting(
          [&](int32_t, const std::vector<uint8_t>&) { ++replies; }),
      {});
  ASSERT_TRUE(first);
  ASSERT_TRUE(second);
  registry->CancelConnection(connection);
  EXPECT_FALSE(registry);
  EXPECT_EQ(replies, 2);
  std::move(*first).Run(0, {});
  std::move(*second).Run(0, {});
  EXPECT_EQ(replies, 2);
}
TEST(RequestRegistryTest, DuplicateAndZeroIdDoNotReplaceAdmittedWork) {
  RequestRegistry registry;
  auto connection = base::UnguessableToken::Create();
  int replies = 0;
  std::vector<int32_t> statuses;
  auto reply = [&](int32_t status, const std::vector<uint8_t>&) {
    ++replies;
    statuses.push_back(status);
  };
  auto accepted =
      registry.Admit({connection, 1}, base::BindLambdaForTesting(reply), {});
  ASSERT_TRUE(accepted);
  EXPECT_FALSE(
      registry.Admit({connection, 1}, base::BindLambdaForTesting(reply), {}));
  EXPECT_FALSE(
      registry.Admit({connection, 0}, base::BindLambdaForTesting(reply), {}));
  EXPECT_EQ(statuses, std::vector<int32_t>(
                          {static_cast<int32_t>(Status::kInvalidArgument),
                           static_cast<int32_t>(Status::kInvalidArgument)}));
  EXPECT_EQ(registry.size(), 1u);
  std::move(*accepted).Run(0, {});
  EXPECT_EQ(replies, 3);
}
TEST(RequestRegistryTest, LateCompletionCannotCompleteReusedId) {
  RequestRegistry registry;
  RequestKey key{base::UnguessableToken::Create(), 1};
  int old_replies = 0;
  int new_replies = 0;
  auto old = registry.Admit(
      key,
      base::BindLambdaForTesting(
          [&](int32_t, const std::vector<uint8_t>&) { ++old_replies; }),
      {});
  ASSERT_TRUE(old);
  EXPECT_TRUE(registry.Cancel(key));
  auto current = registry.Admit(
      key,
      base::BindLambdaForTesting(
          [&](int32_t status, const std::vector<uint8_t>& bytes) {
            ++new_replies;
            EXPECT_EQ(status, 0);
            EXPECT_EQ(bytes, std::vector<uint8_t>({2}));
          }),
      {});
  ASSERT_TRUE(current);
  std::move(*old).Run(0, {1});
  EXPECT_EQ(old_replies, 1);
  EXPECT_EQ(new_replies, 0);
  EXPECT_EQ(registry.size(), 1u);
  std::move(*current).Run(0, {2});
  EXPECT_EQ(new_replies, 1);
  EXPECT_EQ(registry.size(), 0u);
}
}  // namespace tuffite::rpc
