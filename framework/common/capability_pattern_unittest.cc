// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/common/capability_pattern.h"

#include <string>

#include "testing/gtest/include/gtest/gtest.h"
#include "url/url_util.h"

namespace tuffite::capability_pattern {
namespace {

TEST(CapabilityPatternTest, MatchesOriginWildcards) {
  std::string pattern;
  EXPECT_TRUE(NormalizeOrigin("*", &pattern));
  EXPECT_TRUE(
      MatchesOrigin(url::Origin::Create(GURL("https://any.test/")), pattern));

  EXPECT_TRUE(NormalizeOrigin("https://*.example.com/", &pattern));
  EXPECT_EQ(pattern, "https://*.example.com");
  EXPECT_TRUE(MatchesOrigin(
      url::Origin::Create(GURL("https://docs.example.com/")), pattern));
  EXPECT_FALSE(MatchesOrigin(url::Origin::Create(GURL("https://example.com/")),
                             pattern));
  EXPECT_FALSE(NormalizeOrigin("https://*example.com/", &pattern));
  EXPECT_FALSE(NormalizeOrigin("https://*/", &pattern));
  EXPECT_TRUE(NormalizeOrigin("https://*.EXAMPLE.com:443/", &pattern));
  EXPECT_EQ(pattern, "https://*.example.com");
}

TEST(CapabilityPatternTest, CustomOriginWildcardsRespectSchemeHostAndPort) {
  url::ScopedSchemeRegistryForTests registry;
  url::AddStandardScheme("playground", url::SCHEME_WITH_HOST);
  std::string pattern;
  ASSERT_TRUE(NormalizeOrigin("playground://*.artifact", &pattern));
  auto origin = [](const char* url) { return url::Origin::Create(GURL(url)); };
  EXPECT_TRUE(MatchesOrigin(origin("playground://one.artifact/page"), pattern));
  EXPECT_TRUE(
      MatchesOrigin(origin("playground://one.two.artifact/page"), pattern));
  EXPECT_FALSE(MatchesOrigin(origin("playground://artifact/"), pattern));
  EXPECT_FALSE(MatchesOrigin(origin("playground://evilartifact/"), pattern));
  EXPECT_FALSE(MatchesOrigin(origin("https://one.artifact/"), pattern));
  // Host-only schemes do not use a port in their origin.
  EXPECT_TRUE(MatchesOrigin(origin("playground://one.artifact:443/"), pattern));
  EXPECT_FALSE(MatchesOrigin(origin("data:text/html,hello"), "*"));
  EXPECT_FALSE(NormalizeOrigin("playground://*.artifact/path", &pattern));
  EXPECT_FALSE(NormalizeOrigin("playground://user@*.artifact", &pattern));
  EXPECT_FALSE(NormalizeOrigin("playground://*.artifact?query", &pattern));
  ASSERT_TRUE(NormalizeOrigin("https://*.artifact", &pattern));
  EXPECT_TRUE(MatchesOrigin(origin("https://one.artifact:443/"), pattern));
  EXPECT_FALSE(MatchesOrigin(origin("https://one.artifact:444/"), pattern));
}

TEST(CapabilityPatternTest, CustomNavigationScopesRemainSchemeAndPathScoped) {
  url::ScopedSchemeRegistryForTests registry;
  url::AddStandardScheme("playground", url::SCHEME_WITH_HOST);
  std::string pattern;
  ASSERT_TRUE(NormalizeUrl("playground://*.artifact/view/*", &pattern));
  EXPECT_TRUE(MatchesUrl(GURL("playground://one.artifact/view/page"), pattern));
  EXPECT_FALSE(
      MatchesUrl(GURL("playground://one.artifact/private/page"), pattern));
  EXPECT_FALSE(MatchesUrl(GURL("https://one.artifact/view/page"), pattern));
  EXPECT_FALSE(MatchesUrl(GURL("playground://artifact/view/page"), pattern));
}

TEST(CapabilityPatternTest, MatchesUrlAndKeepsPrivateWildcardClosed) {
  std::string pattern;
  EXPECT_TRUE(NormalizeUrl("https://*.example.com/api/*", &pattern));
  EXPECT_TRUE(
      MatchesUrl(GURL("https://docs.example.com/api/items?id=1"), pattern));
  EXPECT_FALSE(
      MatchesUrl(GURL("https://docs.example.com/private/items"), pattern));
  EXPECT_FALSE(MatchesUrl(
      GURL("https://evil.test/https://docs.example.com/api/items"), pattern));

  EXPECT_TRUE(NormalizeUrl("*", &pattern));
  EXPECT_TRUE(MatchesUrl(GURL("https://public.example/"), pattern));
  EXPECT_TRUE(MatchesUrl(GURL("file:///C:/project/report.html"), pattern));
  EXPECT_FALSE(MatchesUrl(GURL("data:text/html,inline"), pattern));
  EXPECT_FALSE(MatchesUrl(GURL("javascript:alert(1)"), pattern));
  EXPECT_FALSE(ExplicitlyAllowsPrivateUrl(GURL("http://127.0.0.1/x"), pattern));
  EXPECT_TRUE(NormalizeUrl("http://127.0.0.1/api/*", &pattern));
  EXPECT_TRUE(
      ExplicitlyAllowsPrivateUrl(GURL("http://127.0.0.1/api/x"), pattern));
  EXPECT_TRUE(NormalizeUrl("127.0.0.1", &pattern));
  EXPECT_TRUE(
      ExplicitlyAllowsPrivateUrl(GURL("http://127.0.0.1:9250/"), pattern));

  EXPECT_TRUE(NormalizeUrl("https://*.EXAMPLE.com:443/api/*", &pattern));
  EXPECT_EQ(pattern, "https://*.example.com/api/*");
  EXPECT_FALSE(NormalizeUrl("https://*/api/*", &pattern));
  EXPECT_FALSE(NormalizeUrl("https://example.com:/api/*", &pattern));
  EXPECT_FALSE(NormalizeUrl("https://example.com/api/?", &pattern));
  EXPECT_FALSE(NormalizeUrl("example.com?unexpected", &pattern));
}

TEST(CapabilityPatternTest, MatchesPathsWithoutEscapingBase) {
  std::string pattern;
  EXPECT_TRUE(NormalizePath("data/reports/*.md", true, &pattern));
  EXPECT_TRUE(MatchesPath("data", "reports/final.md", pattern, true));
  EXPECT_TRUE(MatchesPath("data", "reports/archive/final.md", pattern, true));

  EXPECT_TRUE(NormalizePath("cache/*", true, &pattern));
  EXPECT_TRUE(MatchesPath("cache", "a/b/c.bin", pattern, true));
  EXPECT_FALSE(NormalizePath("resource/**", true, &pattern));
  EXPECT_FALSE(NormalizePath("data/../secret", false, &pattern));
}

}  // namespace
}  // namespace tuffite::capability_pattern
