// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "base/test/run_until.h"
#include "base/test/task_environment.h"
#include "base/test/test_future.h"
#include "testing/gtest/include/gtest/gtest.h"
#include "tuffite/buildflags.h"
#if !BUILDFLAG(TUFFITE_DISABLE_REMOTING)
#include "tuffite/framework/app/services/remoting/remoting_service.h"
#include "tuffite/framework/app/services/remoting/remoting_service_impl.h"
#endif

namespace tuffite::remoting {

TEST(RemotingServiceTest, UiCallerCanGatherIceAndRepeatedlyStopSessions) {
#if BUILDFLAG(TUFFITE_DISABLE_REMOTING)
  GTEST_SKIP() << "Remoting is disabled in this Framework build";
#else
  base::test::TaskEnvironment environment(
      base::test::TaskEnvironment::MainThreadType::UI);
  auto service = CreateRemotingService();
  for (int attempt = 0; attempt < 3; ++attempt) {
    StartOptions options;
    options.auth_key = "native-regression-test-key";
    // Socket allocation must use an IO pump even when the caller uses a UI
    // pump. The loopback endpoint avoids depending on an external STUN service.
    options.ice_servers.push_back({.urls = {"stun:127.0.0.1:9"}});
    base::test::TestFuture<SessionResult> start;
    service->Start(std::move(options), start.GetCallback());
    EXPECT_FALSE(start.IsReady());
    auto started = start.Take();
    ASSERT_EQ(started.status, Status::kOk);
    ASSERT_TRUE(base::test::RunUntil([&] {
      base::test::TestFuture<SignalsResult> received;
      service->TakeSignals(started.value.id, 0, received.GetCallback());
      auto signals = received.Take();
      return signals.status == Status::kOk && !signals.value.empty();
    }));
    base::test::TestFuture<StatsResult> stats;
    service->GetStats(started.value.id, stats.GetCallback());
    EXPECT_EQ(stats.Take().status, Status::kOk);
    base::test::TestFuture<EmptyResult> stopped;
    service->Stop(started.value.id, stopped.GetCallback());
    EXPECT_EQ(stopped.Take().status, Status::kOk);
    service->GetStats(started.value.id, stats.GetCallback());
    EXPECT_EQ(stats.Take().status, Status::kInvalidHandle);
  }
  // Teardown must also drain a live connection on its owning IO sequence.
  StartOptions options;
  options.auth_key = "pending-shutdown-test-key";
  options.ice_servers.push_back({.urls = {"stun:127.0.0.1:9"}});
  base::test::TestFuture<SessionResult> start;
  service->Start(std::move(options), start.GetCallback());
  ASSERT_EQ(start.Get().status, Status::kOk);
  service->Shutdown();
  service.reset();
  environment.RunUntilIdle();
#endif
}

}  // namespace tuffite::remoting
