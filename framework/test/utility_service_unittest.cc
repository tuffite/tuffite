// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/services/utility/utility_service.h"

#include <map>

#include "base/functional/bind.h"
#include "base/functional/callback_helpers.h"
#include "base/memory/raw_ptr.h"
#include "base/run_loop.h"
#include "base/test/bind.h"
#include "base/test/scoped_run_loop_timeout.h"
#include "base/test/task_environment.h"
#include "base/test/test_future.h"
#include "mojo/public/cpp/bindings/receiver.h"
#include "testing/gtest/include/gtest/gtest.h"
#include "tuffite/framework/app/services/utility/utility_process_launcher.h"
#include "tuffite/framework/public/abi.h"
namespace tuffite {
namespace {
class FakeProcess final : public mojom::UtilityService {
 public:
  mojo::Receiver<mojom::UtilityService> receiver{this};
  std::map<uint64_t, InvokeCallback> pending;
  std::vector<uint8_t> startup_configuration;
  base::RepeatingClosure on_invoke;
  void WaitForRequests(size_t count) {
    if (pending.size() >= count) {
      return;
    }
    base::RunLoop loop;
    on_invoke = base::BindLambdaForTesting([&] {
      if (pending.size() >= count) {
        loop.Quit();
      }
    });
    loop.Run();
    on_invoke.Reset();
  }
  void Start(const std::string&,
             const std::vector<uint8_t>& configuration,
             StartCallback callback) override {
    startup_configuration = configuration;
    std::move(callback).Run(TUFFITE_STATUS_OK);
  }
  void Invoke(uint64_t id,
              const std::vector<uint8_t>&,
              InvokeCallback callback) override {
    pending.emplace(id, std::move(callback));
    if (on_invoke) {
      on_invoke.Run();
    }
  }
  void Cancel(uint64_t id) override {
    auto node = pending.extract(id);
    if (!node.empty()) {
      std::move(node.mapped()).Run(TUFFITE_STATUS_INVALID_STATE, {});
    }
  }
  void CreateSession(uint64_t,
                     const std::vector<uint8_t>&,
                     uint32_t,
                     uint32_t,
                     mojo::PendingRemote<mojom::UtilitySessionClient>,
                     mojo::PendingReceiver<mojom::UtilitySession>,
                     CreateSessionCallback callback) override {
    std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT);
  }
  void Crash() {
    pending.clear();
    receiver.reset();
  }
};
class FakeLauncher final : public UtilityProcessLauncher {
 public:
  FakeProcess process;
  size_t launches = 0;
  mojo::Remote<mojom::UtilityService> Launch(std::string_view) override {
    ++launches;
    mojo::Remote<mojom::UtilityService> remote;
    process.receiver.Bind(remote.BindNewPipeAndPassReceiver());
    return remote;
  }
};
class UtilityServiceTest : public testing::Test {
 protected:
  base::test::SingleThreadTaskEnvironment environment{
      base::test::TaskEnvironment::TimeSource::MOCK_TIME};
  base::test::ScopedRunLoopTimeout timeout{FROM_HERE, base::Seconds(10)};
  const base::UnguessableToken owner = base::UnguessableToken::Create();
  std::unique_ptr<UtilityService> service;
  raw_ptr<FakeLauncher> launcher = nullptr;
  void SetUp() override {
    auto backend = std::make_unique<FakeLauncher>();
    launcher = backend.get();
    service = CreateUtilityService(std::move(backend));
    service->SetUtilityRegistrationChecker(base::BindRepeating(
        [](std::string_view name) { return name == "demo"; }));
  }
  void TearDown() override {
    service->Shutdown();
    launcher = nullptr;
    service.reset();
  }
};
TEST_F(UtilityServiceTest, ServiceStartupConfigurationIsIndependentOfRequests) {
  service->SetUtilityStartupConfigurations({{"demo", {7, 8, 9}}});
  service->InvokeUtilityService(owner, 1, "demo", {1, 2}, base::DoNothing());
  launcher->process.WaitForRequests(1);
  ASSERT_EQ(launcher->launches, 1u);
  EXPECT_EQ(launcher->process.startup_configuration,
            (std::vector<uint8_t>{7, 8, 9}));
}
TEST_F(UtilityServiceTest, ShutdownFailsOnceAndNeverRestarts) {
  int replies = 0;
  int32_t status = 0;
  service->InvokeUtilityService(
      owner, 1, "demo", {},
      base::BindLambdaForTesting([&](int32_t s, std::vector<uint8_t>) {
        ++replies;
        status = s;
      }));
  launcher->process.WaitForRequests(1);
  ASSERT_EQ(launcher->launches, 1u);
  service->Shutdown();
  EXPECT_EQ(replies, 1);
  EXPECT_EQ(status, TUFFITE_STATUS_INVALID_STATE);
  environment.FastForwardBy(base::Minutes(2));
  EXPECT_EQ(launcher->launches, 1u);
  EXPECT_EQ(replies, 1);
  service->InvokeUtilityService(
      owner, 2, "demo", {},
      base::BindLambdaForTesting([&](int32_t s, std::vector<uint8_t>) {
        ++replies;
        status = s;
      }));
  EXPECT_EQ(status, TUFFITE_STATUS_INVALID_STATE);
  EXPECT_EQ(replies, 2);
  EXPECT_EQ(launcher->launches, 1u);
}
TEST_F(UtilityServiceTest, CrashFailsAdmittedWorkWithoutReplay) {
  base::test::TestFuture<void> first_reply;
  int replies = 0;
  int32_t status = 0;
  service->InvokeUtilityService(
      owner, 1, "demo", {},
      base::BindLambdaForTesting([&](int32_t s, std::vector<uint8_t>) {
        ++replies;
        status = s;
        first_reply.SetValue();
      }));
  launcher->process.WaitForRequests(1);
  ASSERT_EQ(launcher->process.pending.size(), 1u);
  launcher->process.Crash();
  ASSERT_TRUE(first_reply.Wait());
  EXPECT_EQ(replies, 1);
  EXPECT_EQ(status, TUFFITE_STATUS_INTERNAL);
  environment.FastForwardBy(base::Seconds(1));
  EXPECT_EQ(launcher->launches, 1u);
  service->InvokeUtilityService(
      owner, 2, "demo", {},
      base::BindLambdaForTesting(
          [&](int32_t, std::vector<uint8_t>) { ++replies; }));
  launcher->process.WaitForRequests(1);
  EXPECT_EQ(launcher->launches, 2u);
  EXPECT_FALSE(launcher->process.pending.contains(1));
  EXPECT_TRUE(launcher->process.pending.contains(2));
  // Complete callbacks while their captured test-local state is still alive.
  service->Shutdown();
  EXPECT_EQ(replies, 2);
}
TEST_F(UtilityServiceTest, SameRequestIdIsIndependentAcrossOwners) {
  const base::UnguessableToken second_owner = base::UnguessableToken::Create();
  base::test::TestFuture<void> first_reply;
  int first_replies = 0;
  int second_replies = 0;
  service->InvokeUtilityService(
      owner, 7, "demo", {},
      base::BindLambdaForTesting([&](int32_t, std::vector<uint8_t>) {
        ++first_replies;
        first_reply.SetValue();
      }));
  service->InvokeUtilityService(
      second_owner, 7, "demo", {},
      base::BindLambdaForTesting(
          [&](int32_t, std::vector<uint8_t>) { ++second_replies; }));
  launcher->process.WaitForRequests(2);
  ASSERT_EQ(launcher->process.pending.size(), 2u);

  service->CancelUtilityOwner(owner);
  ASSERT_TRUE(first_reply.Wait());
  EXPECT_EQ(first_replies, 1);
  EXPECT_EQ(second_replies, 0);
  EXPECT_EQ(launcher->process.pending.size(), 1u);

  service->Shutdown();
  EXPECT_EQ(first_replies, 1);
  EXPECT_EQ(second_replies, 1);
}

TEST_F(UtilityServiceTest, TimeoutCompletesOnceAndShutdownCancelsBackoff) {
  int replies = 0;
  service->InvokeUtilityService(
      owner, 1, "demo", {},
      base::BindLambdaForTesting(
          [&](int32_t, std::vector<uint8_t>) { ++replies; }));
  launcher->process.WaitForRequests(1);
  environment.FastForwardBy(base::Seconds(31));
  EXPECT_EQ(replies, 1);
  service->Shutdown();
  environment.FastForwardBy(base::Minutes(2));
  EXPECT_EQ(replies, 1);
  EXPECT_EQ(launcher->launches, 1u);
}
}  // namespace
}  // namespace tuffite
