// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_lifecycle.h"

#include "base/functional/bind.h"
#include "base/test/launcher/unit_test_launcher.h"
#include "base/test/scoped_feature_list.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/site_instance.h"
#include "content/public/browser/storage_partition_config.h"
#include "content/public/browser/web_contents.h"
#include "content/public/common/content_features.h"
#include "content/public/test/test_browser_context.h"
#include "content/public/test/test_renderer_host.h"
#include "content/public/test/unittest_test_suite.h"
#include "content/test/content_test_suite.h"
#include "tuffite/components/explorer/browser/content/explorer_isolation.h"
#include "tuffite/components/explorer/browser/explorer_browser_support.h"
#include "tuffite/components/explorer/browser/explorer_policy.h"
#include "url/origin.h"

namespace tuffite {
namespace {
class ExplorerLifecycleTest : public content::RenderViewHostTestHarness {
 public:
  ExplorerLifecycleTest()
      : RenderViewHostTestHarness(
            base::test::TaskEnvironment::TimeSource::MOCK_TIME) {}
  void SetUp() override {
    RenderViewHostTestHarness::SetUp();
    NavigateAndCommit(GURL("https://example.test/page"));
    web_contents()->WasShown();
    ExplorerLifecycle::CreateForWebContents(web_contents());
    lifecycle()->SetAttached(true);
  }
  void SetPolicy(std::string_view text) {
    auto overrides = ParseExplorerPolicy(text);
    ASSERT_TRUE(overrides);
    ExplorerPolicySettings settings;
    settings.Apply(*overrides);
    lifecycle()->SetPolicy(settings);
  }
  ExplorerLifecycle* lifecycle() {
    return ExplorerLifecycle::FromWebContents(web_contents());
  }
};

TEST_F(ExplorerLifecycleTest, FreezesOnlyAfterHiddenIdleAndThawsOnShow) {
  task_environment()->FastForwardBy(base::Minutes(1));
  EXPECT_FALSE(lifecycle()->frozen());
  web_contents()->WasHidden();
  task_environment()->FastForwardBy(base::Seconds(29));
  EXPECT_FALSE(lifecycle()->frozen());
  task_environment()->FastForwardBy(base::Seconds(1));
  EXPECT_TRUE(lifecycle()->frozen());
  web_contents()->WasShown();
  EXPECT_FALSE(lifecycle()->frozen());
}

TEST_F(ExplorerLifecycleTest, KeepAliveAndCaptureProtectHiddenPages) {
  web_contents()->WasHidden();
  SetPolicy("freeze: never; discard: never;");
  task_environment()->FastForwardBy(base::Minutes(20));
  EXPECT_FALSE(lifecycle()->frozen());
  EXPECT_FALSE(lifecycle()->discarded());
  SetPolicy("discard: never;");
  auto capture = web_contents()->IncrementCapturerCount({}, false, true, true);
  task_environment()->FastForwardBy(base::Minutes(1));
  EXPECT_FALSE(lifecycle()->frozen());
  capture.RunAndReset();
  task_environment()->FastForwardBy(base::Minutes(1));
  EXPECT_TRUE(lifecycle()->frozen());
  SetPolicy("freeze: never; discard: never;");
  EXPECT_FALSE(lifecycle()->frozen());
}

TEST_F(ExplorerLifecycleTest, ActiveDialogThawsAndRestartsIdleInterval) {
  web_contents()->WasHidden();
  task_environment()->FastForwardBy(base::Seconds(30));
  ASSERT_TRUE(lifecycle()->frozen());
  lifecycle()->SetDialogPending(true);
  EXPECT_FALSE(lifecycle()->frozen());
  task_environment()->FastForwardBy(base::Minutes(1));
  EXPECT_FALSE(lifecycle()->frozen());
  lifecycle()->SetDialogPending(false);
  task_environment()->FastForwardBy(base::Seconds(30));
  EXPECT_TRUE(lifecycle()->frozen());
}

TEST_F(ExplorerLifecycleTest, VisibleOrDisabledDiscardIsRejected) {
  base::test::ScopedFeatureList feature_list;
  feature_list.InitAndDisableFeature(features::kWebContentsDiscard);
  EXPECT_FALSE(lifecycle()->Discard());
  web_contents()->WasHidden();
  task_environment()->FastForwardBy(base::Minutes(20));
  EXPECT_TRUE(lifecycle()->frozen());
  EXPECT_FALSE(lifecycle()->discarded());
  EXPECT_FALSE(lifecycle()->Discard());
}

TEST_F(ExplorerLifecycleTest, AutoDiscardsButFreezeOnlyPreservesHiddenPages) {
  base::test::ScopedFeatureList feature_list;
  feature_list.InitAndEnableFeature(features::kWebContentsDiscard);
  SetPolicy("discard: never;");
  web_contents()->WasHidden();
  task_environment()->FastForwardBy(base::Minutes(20));
  EXPECT_TRUE(lifecycle()->frozen());
  EXPECT_FALSE(lifecycle()->discarded());
  SetPolicy("");
  task_environment()->FastForwardBy(base::Minutes(15));
  EXPECT_TRUE(lifecycle()->discarded());
  EXPECT_FALSE(lifecycle()->frozen());
}

TEST_F(ExplorerLifecycleTest, DetachedHandlePreservesItsDocument) {
  base::test::ScopedFeatureList feature_list;
  feature_list.InitAndEnableFeature(features::kWebContentsDiscard);
  lifecycle()->SetAttached(false);
  web_contents()->WasHidden();
  task_environment()->FastForwardBy(base::Minutes(20));
  EXPECT_TRUE(lifecycle()->frozen());
  EXPECT_FALSE(lifecycle()->discarded());
  EXPECT_FALSE(lifecycle()->Discard());
}

TEST_F(ExplorerLifecycleTest, IndependentTimersAndRuntimeUpdates) {
  SetPolicy("freeze: 2s; discard: never;");
  web_contents()->WasHidden();
  task_environment()->FastForwardBy(base::Seconds(1));
  EXPECT_FALSE(lifecycle()->frozen());
  task_environment()->FastForwardBy(base::Seconds(1));
  EXPECT_TRUE(lifecycle()->frozen());
  SetPolicy("freeze: never; discard: never;");
  EXPECT_FALSE(lifecycle()->frozen());
  task_environment()->FastForwardBy(base::Minutes(20));
  EXPECT_FALSE(lifecycle()->frozen());
  EXPECT_FALSE(lifecycle()->discarded());
  SetPolicy("freeze: 1s; discard: never;");
  task_environment()->FastForwardBy(base::Seconds(1));
  EXPECT_TRUE(lifecycle()->frozen());
  SetPolicy("freeze: 2s; discard: never;");
  EXPECT_FALSE(lifecycle()->frozen());
  task_environment()->FastForwardBy(base::Seconds(2));
  EXPECT_TRUE(lifecycle()->frozen());
}

TEST_F(ExplorerLifecycleTest, ExplicitDiscardIgnoresAutomaticNever) {
  base::test::ScopedFeatureList features;
  features.InitAndEnableFeature(features::kWebContentsDiscard);
  SetPolicy("freeze: never; discard: never;");
  EXPECT_FALSE(lifecycle()->Discard());
  web_contents()->WasHidden();
  EXPECT_TRUE(lifecycle()->Discard());
}

TEST_F(ExplorerLifecycleTest,
       ProcessDomainsRespectOwnerGrantsAndOpaqueOrigins) {
  const auto owner = url::Origin::Create(GURL("https://app.test"));
  ExplorerPolicy policy;
  const auto shared =
      ResolveExplorerIsolationId(*browser_context(), owner, policy);
  EXPECT_EQ(shared,
            ResolveExplorerIsolationId(*browser_context(), owner, policy));
  auto site = CreateExplorerSiteInstance(
      browser_context(), GURL("https://example.test"),
      content::StoragePartitionConfig::CreateDefault(browser_context()),
      shared);
  EXPECT_TRUE(ShouldUseExplorerProcessPerSite(site->GetSecurityPrincipal()));
  policy.settings.freeze = {ExplorerDelay::Mode::kNever};
  policy.settings.discard = {ExplorerDelay::Mode::kAfter, 2000};
  policy.settings.actions[static_cast<size_t>(ExplorerAction::kContextMenu)]
      .mode = ExplorerMode::kBrowser;
  policy.navigation = ExplorerNavigationPolicy::kBlock;
  policy.dialogs = ExplorerDialogPolicy::kBrowser;
  EXPECT_EQ(shared,
            ResolveExplorerIsolationId(*browser_context(), owner, policy));
  policy.allow_shell_api = true;
  EXPECT_NE(shared,
            ResolveExplorerIsolationId(*browser_context(), owner, policy));
  policy.allow_shell_api = false;
  EXPECT_NE(shared,
            ResolveExplorerIsolationId(
                *browser_context(),
                url::Origin::Create(GURL("https://other.test")), policy));
  content::TestBrowserContext other_context;
  EXPECT_NE(shared, ResolveExplorerIsolationId(other_context, owner, policy));
  const url::Origin opaque_owner;
  const auto opaque_domain =
      ResolveExplorerIsolationId(*browser_context(), opaque_owner, policy);
  EXPECT_NE(opaque_domain, ResolveExplorerIsolationId(*browser_context(),
                                                      opaque_owner, policy));
  auto isolated = CreateExplorerSiteInstance(
      browser_context(), GURL("https://example.test"),
      content::StoragePartitionConfig::CreateDefault(browser_context()),
      opaque_domain);
  EXPECT_FALSE(
      ShouldUseExplorerProcessPerSite(isolated->GetSecurityPrincipal()));
}
}  // namespace
}  // namespace tuffite

int main(int argc, char** argv) {
  content::UnitTestTestSuite suite(
      new content::ContentTestSuite(argc, argv),
      base::BindRepeating(
          content::UnitTestTestSuite::CreateTestContentClients));
  return base::LaunchUnitTests(argc, argv,
                               base::BindOnce(&content::UnitTestTestSuite::Run,
                                              base::Unretained(&suite)));
}
