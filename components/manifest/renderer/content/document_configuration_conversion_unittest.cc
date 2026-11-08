// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// content/test:test_support links this private renderer implementation.
#include "content/renderer/policy_container_util.h"  // nogncheck
#include "testing/gtest/include/gtest/gtest.h"
#include "third_party/blink/public/mojom/frame/policy_container.mojom.h"
#include "tuffite/components/manifest/common/document_configuration.mojom.h"

namespace tuffite {

TEST(DocumentConfigurationConversionTest,
     ExposureSurvivesContentBlinkBoundary) {
  auto policies = blink::mojom::PolicyContainerPolicies::New();
  policies->tuffite_configuration =
      mojom::DocumentConfiguration::New(true, false);
  auto web = content::ToWebPolicyContainerPolicies(policies);
  EXPECT_TRUE(web.tuffite_expose_shell_api);
  EXPECT_FALSE(web.tuffite_enable_explorer);
  auto restored = content::FromWebPolicyContainerPolicies(web);
  ASSERT_TRUE(restored->tuffite_configuration);
  EXPECT_TRUE(restored->tuffite_configuration->expose_shell_api);
  EXPECT_FALSE(restored->tuffite_configuration->enable_explorer);
}

TEST(DocumentConfigurationConversionTest, DefaultPolicyRemainsDenied) {
  auto policies = blink::mojom::PolicyContainerPolicies::New();
  auto web = content::ToWebPolicyContainerPolicies(policies);
  EXPECT_FALSE(web.tuffite_expose_shell_api);
  EXPECT_FALSE(web.tuffite_enable_explorer);
  auto restored = content::FromWebPolicyContainerPolicies(web);
  EXPECT_FALSE(restored->tuffite_configuration);
}

}  // namespace tuffite
