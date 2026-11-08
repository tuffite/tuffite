// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/manifest/browser/content/document_configuration_provider.h"

#include "base/functional/bind.h"
#include "base/test/launcher/unit_test_launcher.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/test/test_renderer_host.h"
#include "content/public/test/unittest_test_suite.h"
#include "content/test/content_test_suite.h"
#include "third_party/blink/public/mojom/frame/policy_container.mojom.h"
#include "tuffite/components/manifest/common/document_configuration.mojom.h"
#include "url/origin.h"

namespace tuffite {
namespace {
using DocumentConfigurationTest = content::RenderViewHostTestHarness;
TEST_F(DocumentConfigurationTest, UnknownContextAndOpaqueOriginsDenyExposure) {
  NavigateAndCommit(GURL("https://allowed.test"));
  auto container = blink::mojom::PolicyContainer::New();
  container->policies = blink::mojom::PolicyContainerPolicies::New();
  ConfigureDocument(*main_rfh(),
                    url::Origin::Create(GURL("https://allowed.test")),
                    container.get());
  EXPECT_FALSE(container->policies->tuffite_configuration);
  SetDocumentConfigurationProvider(
      *browser_context(),
      base::BindRepeating([](content::RenderFrameHost&, const url::Origin&) {
        return DocumentExposure{true, true};
      }));
  ConfigureDocument(*main_rfh(), url::Origin(), container.get());
  EXPECT_FALSE(container->policies->tuffite_configuration);
}

TEST_F(DocumentConfigurationTest, ReusedFrameDoesNotRetainPriorExposure) {
  NavigateAndCommit(GURL("https://allowed.test"));
  SetDocumentConfigurationProvider(
      *browser_context(), base::BindRepeating([](content::RenderFrameHost&,
                                                 const url::Origin& origin) {
        return DocumentExposure{origin.host() == "allowed.test",
                                origin.host() == "allowed.test"};
      }));
  auto container = blink::mojom::PolicyContainer::New();
  container->policies = blink::mojom::PolicyContainerPolicies::New();
  ConfigureDocument(*main_rfh(),
                    url::Origin::Create(GURL("https://allowed.test")),
                    container.get());
  ASSERT_TRUE(container->policies->tuffite_configuration);
  EXPECT_TRUE(container->policies->tuffite_configuration->expose_shell_api);
  auto cached = container->policies.Clone();
  ConfigureDocument(*main_rfh(),
                    url::Origin::Create(GURL("https://denied.test")),
                    container.get());
  EXPECT_FALSE(container->policies->tuffite_configuration);
  EXPECT_TRUE(cached->tuffite_configuration->enable_explorer);
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
