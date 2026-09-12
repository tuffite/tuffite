// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/common/app_manifest.h"

#include "base/files/file_util.h"
#include "base/files/scoped_temp_dir.h"
#include "testing/gtest/include/gtest/gtest.h"
#include "url/url_util.h"

namespace tuffite {
namespace {
constexpr char kManifest[] = R"({
  "app": {
    "identifier": "com.example.originlab",
    "security": {
      "capabilities": [
        {
          "identifier": "app",
          "origins": [
            "originlab://main"
          ],
          "explorer": {
            "create": true,
            "executeJavaScript": true
          },
          "shellApi": [
            "originlab.*",
            {
              "method": "tuffite.fs.readFile",
              "allow": [
                {
                  "path": "resource/allowed/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.exists",
              "allow": [
                {
                  "path": "resource/allowed/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.writeFile",
              "allow": [
                {
                  "path": "data/safe/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.createDir",
              "allow": [
                {
                  "path": "data/safe/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.removeFile",
              "allow": [
                {
                  "path": "data/safe/*"
                }
              ]
            },
            {
              "method": "tuffite.net.request",
              "allow": [
                {
                  "url": "https://example.com/api/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.readFile",
              "allow": [
                {
                  "path": "data/**"
                }
              ]
            },
            "tuffite.window.focus",
            "tuffite.window.close",
            {
              "method": "tuffite.window.close",
              "allow": [
                {
                  "window": "others"
                }
              ]
            }
          ]
        },
        {
          "identifier": "readonly",
          "origins": [
            "originlab://*.artifact",
            "originlab://*.preview"
          ],
          "shellApi": [
            "tuffite.path.directories"
          ]
        },
        {
          "identifier": "artifact",
          "origins": [
            "originlab://demo.artifact"
          ],
          "shellApi": [
            "originlab.commands.allowed"
          ]
        },
        {
          "identifier": "no-grant",
          "origins": [
            "originlab://*.artifact"
          ],
          "shellApi": []
        },
        {
          "identifier": "denials",
          "origins": [
            "originlab://main"
          ],
          "shellApi": [
            {
              "method": "tuffite.fs.readFile",
              "deny": [
                {
                  "path": "data/secrets/**"
                }
              ]
            },
            {
              "method": "tuffite.window.close",
              "deny": [
                {
                  "window": "others"
                }
              ]
            },
            {
              "method": "tuffite.net.request",
              "deny": [
                {
                  "url": "https://example.com/api/private/**"
                }
              ]
            }
          ]
        }
      ]
    },
    "windows": [
      {
        "label": "main",
        "url": "originlab://main/index.html",
        "width": 800,
        "height": 600
      }
    ],
    "shellApi": {
      "namespace": "originlab",
      "commandIds": {
        "tuffite.path.directories": "0000000000000000",
        "originlab.commands.allowed": "0000000000000001",
        "originlab.commands.denied": "0000000000000002",
        "tuffite.fs.readFile": "0000000000000000",
        "tuffite.fs.exists": "0000000000000000",
        "tuffite.fs.writeFile": "0000000000000000",
        "tuffite.fs.createDir": "0000000000000000",
        "tuffite.fs.removeFile": "0000000000000000",
        "tuffite.net.request": "0000000000000000",
        "tuffite.window.focus": "0000000000000000",
        "tuffite.window.close": "0000000000000000"
      }
    },
    "network": {},
    "assets": [
      {
        "origins": [
          "originlab://main",
          "originlab://*.artifact",
          "originlab://*.preview",
          "originlab://public"
        ],
        "paths": [
          "/*"
        ],
        "directory": "ui"
      },
      {
        "origins": [
          "originlab://main"
        ],
        "paths": [
          "/exact",
          "/app/*"
        ],
        "file": "ui/index.html"
      },
      {
        "origins": [
          "originlab://entryonly"
        ],
        "paths": [
          "/home"
        ],
        "file": "ui/index.html"
      }
    ]
  }
})";
class OriginCapabilityTest : public testing::Test {
 public:
  void SetUp() override {
    url::AddStandardScheme("originlab", url::SCHEME_WITH_HOST);
    ASSERT_TRUE(root_.CreateUniqueTempDir());
    ASSERT_TRUE(base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"),
                                kManifest));
    manifest_ = AppManifest::Load(root_.GetPath(), &error_);
    ASSERT_TRUE(manifest_) << error_;
  }
  url::Origin Origin(const char* value) {
    return url::Origin::Create(GURL(value));
  }

 protected:
  url::ScopedSchemeRegistryForTests registry_;
  base::ScopedTempDir root_;
  std::optional<AppManifest> manifest_;
  std::string error_;
};
TEST_F(OriginCapabilityTest, ProductNameDefaultsToApplicationIdentity) {
  EXPECT_EQ(manifest_->product_name(), "com.example.originlab");
}

TEST_F(OriginCapabilityTest, ProductNameIsIndependentFromIdentity) {
  auto text = std::string(kManifest);
  text.insert(1, "\"productName\":\"Origin Lab\",");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), text));
  auto manifest = AppManifest::Load(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  EXPECT_EQ(manifest->product_name(), "Origin Lab");
  EXPECT_EQ(manifest->application_id(), "com.example.originlab");
}

TEST_F(OriginCapabilityTest, InvalidProductNamesAreRejected) {
  for (const auto* value : {"7", "null", "\"\"", "\"bad\\nname\""}) {
    auto text = std::string(kManifest);
    text.insert(1, std::string("\"productName\":") + value + ",");
    ASSERT_TRUE(
        base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), text));
    EXPECT_FALSE(AppManifest::Load(root_.GetPath(), &error_)) << value;
  }
}

TEST_F(OriginCapabilityTest,
       UtilityStartupConfigurationIsServiceScopedAndBounded) {
  std::string configured(kManifest);
  configured.replace(configured.find("\"identifier\""), 0,
                     "\"utilities\": {\"indexer\": {\"configuration\": "
                     "{\"mode\": 7}}, \"parser\": {}},");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), configured));
  auto manifest = AppManifest::Load(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  auto configurations = manifest->UtilityConfigurations();
  ASSERT_EQ(configurations.size(), 1u);
  EXPECT_EQ(std::string(configurations.at("indexer").begin(),
                        configurations.at("indexer").end()),
            "{\"mode\":7}");
  configured.replace(configured.find("{\"mode\": 7}"), 11,
                     "\"" + std::string(64 * 1024, 'x') + "\"");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), configured));
  EXPECT_FALSE(AppManifest::Load(root_.GetPath(), &error_));
}
TEST_F(OriginCapabilityTest, UnionGrantsRemainOriginScoped) {
  auto artifact = Origin("originlab://demo.artifact/page");
  auto preview = Origin("originlab://demo.preview/page");
  EXPECT_NE(artifact, preview);
  EXPECT_TRUE(manifest_->IsShellApiCommandAllowed(artifact,
                                                  "tuffite.path.directories"));
  EXPECT_TRUE(
      manifest_->IsShellApiCommandAllowed(preview, "tuffite.path.directories"));
  EXPECT_TRUE(manifest_->IsShellApiCommandAllowed(
      artifact, "originlab.commands.allowed"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      preview, "originlab.commands.allowed"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      artifact, "originlab.commands.denied"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(Origin("originlab://other/"),
                                                   "tuffite.path.directories"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      Origin("data:text/html,hello"), "tuffite.path.directories"));
  EXPECT_FALSE(manifest_->IsExplorerOriginAllowed(artifact));
  EXPECT_TRUE(manifest_->IsExplorerOriginAllowed(Origin("originlab://main/")));
  EXPECT_TRUE(manifest_->IsAppUrl(GURL("originlab://demo.preview/index.html")));
  EXPECT_FALSE(manifest_->IsAppUrl(GURL("originlab://unknown/index.html")));
  // Chromium registers application schemes as host-only origins.
  EXPECT_TRUE(
      manifest_->IsAppUrl(GURL("originlab://demo.preview:443/index.html")));
}
TEST_F(OriginCapabilityTest,
       AssetsAndFrameworkScopesAreIndependentOfBusinessGrants) {
  const auto app = Origin("originlab://main/");
  const auto public_origin = Origin("originlab://public/");
  EXPECT_TRUE(manifest_->IsAppUrl(GURL("originlab://public/index.html")));
  EXPECT_FALSE(manifest_->IsShellApiOriginAllowed(public_origin));
  EXPECT_FALSE(manifest_->IsExplorerOriginAllowed(public_origin));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(app, "tuffite.storage.get"));
  EXPECT_TRUE(
      manifest_->IsShellApiCommandAllowed(app, "originlab.commands.allowed"));
  EXPECT_TRUE(
      manifest_->IsFileSystemPathAllowed(app, "resource", "allowed/a", false));
  EXPECT_FALSE(manifest_->IsFileSystemPathAllowed(public_origin, "resource",
                                                  "allowed/a", false));
  EXPECT_FALSE(
      manifest_->IsFileSystemPathAllowed(app, "resource", "private/a", false));
  EXPECT_TRUE(manifest_->IsNetworkUrlAllowed(
      app, GURL("https://example.com/api/items")));
  EXPECT_FALSE(manifest_->IsNetworkUrlAllowed(
      public_origin, GURL("https://example.com/api/items")));
  EXPECT_FALSE(
      manifest_->IsNetworkUrlAllowed(app, GURL("https://example.com/private")));
  EXPECT_EQ(manifest_->ResolveAssetRoute(GURL("originlab://main/exact")),
            "/ui/index.html");
  EXPECT_EQ(manifest_->ResolveAssetRoute(GURL("originlab://main/app/deep")),
            "/ui/index.html");
  EXPECT_EQ(manifest_->ResolveAssetRoute(GURL("originlab://public/app/deep")),
            "/ui/app/deep");
  EXPECT_EQ(manifest_->ResolveAssetRoute(GURL("originlab://entryonly/home")),
            "/ui/index.html");
  EXPECT_TRUE(
      manifest_->ResolveAssetRoute(GURL("originlab://entryonly/missing"))
          .empty());
  EXPECT_TRUE(
      manifest_->ResolveAssetRoute(GURL("originlab://entryonly/")).empty());
}
TEST_F(OriginCapabilityTest, PackagedModulePathsReuseOriginAndRouteChecks) {
  auto path = manifest_->ResolveAssetFile(GURL("originlab://main/exact"));
  ASSERT_TRUE(path);
  EXPECT_EQ(*path, manifest_->asset_root().AppendASCII("ui/index.html"));
  EXPECT_FALSE(manifest_->ResolveAssetFile(GURL("https://example.com/exact")));
  EXPECT_FALSE(manifest_->ResolveAssetFile(GURL("file:///tmp/exact.html")));
  EXPECT_FALSE(manifest_->ResolveAssetFile(GURL("originlab://unknown/exact")));
  EXPECT_FALSE(
      manifest_->ResolveAssetFile(GURL("originlab://entryonly/missing")));
  // A fixed-file route intentionally ignores the request's suffix.
  EXPECT_EQ(manifest_->ResolveAssetFile(
                GURL("originlab://main/app/%2e%2e%2fsecret.mjs")),
            path);
  // A directory route must reject decoded parent components.
  EXPECT_FALSE(manifest_->ResolveAssetFile(
      GURL("originlab://public/app/%2e%2e%2fsecret.mjs")));
}

TEST_F(OriginCapabilityTest, OwnerNavigationDoesNotInheritAssetAuthority) {
  const auto owner = Origin("originlab://main/");
  EXPECT_TRUE(manifest_->IsAppUrl(GURL("originlab://public/index.html")));
  EXPECT_TRUE(manifest_->IsExplorerNavigationAllowed(
      owner, GURL("originlab://main/index.html")));
  EXPECT_TRUE(manifest_->IsExplorerNavigationAllowed(
      owner, GURL("originlab://public/index.html")));
  EXPECT_TRUE(manifest_->IsExplorerNavigationAllowed(
      owner, GURL("https://example.com/")));
}

TEST_F(OriginCapabilityTest,
       ResourceDenialsOverrideUnionWithoutLeakingMethodScopes) {
  const auto owner = Origin("originlab://main/");
  EXPECT_TRUE(
      manifest_->IsFileSystemPathAllowed(owner, "data", "safe/a", false));
  EXPECT_FALSE(
      manifest_->IsFileSystemPathAllowed(owner, "data", "secrets/a", false));
  EXPECT_FALSE(manifest_->IsFileSystemPathAllowed(owner, "data", "safe/a",
                                                  false, "tuffite.fs.exists"));
  EXPECT_TRUE(manifest_->IsShellApiScopeAllowed(owner, "tuffite.window.focus",
                                                "current"));
  EXPECT_FALSE(manifest_->IsShellApiScopeAllowed(owner, "tuffite.window.focus",
                                                 "others"));
  EXPECT_FALSE(manifest_->IsShellApiScopeAllowed(owner, "tuffite.window.close",
                                                 "others"));
  EXPECT_FALSE(manifest_->IsNetworkUrlAllowed(
      owner, GURL("https://example.com/api/private/token")));
}
TEST_F(OriginCapabilityTest,
       AllMethodsWildcardPreservesOriginAndResourceScopes) {
  auto contents = std::string(kManifest);
  const auto pos = contents.find("originlab.*");
  ASSERT_NE(pos, std::string::npos);
  contents.replace(pos, std::string("originlab.*").size(), "*");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), contents));
  auto manifest = AppManifest::Load(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  const auto owner = Origin("originlab://main/");
  EXPECT_TRUE(
      manifest->IsShellApiCommandAllowed(owner, "tuffite.path.directories"));
  EXPECT_FALSE(manifest->IsShellApiCommandAllowed(owner, "tuffite.unknown"));
  EXPECT_FALSE(manifest->IsShellApiCommandAllowed(Origin("originlab://public/"),
                                                  "tuffite.path.directories"));
  EXPECT_TRUE(
      manifest->IsFileSystemPathAllowed(owner, "data", "safe/a", false));
  EXPECT_FALSE(
      manifest->IsFileSystemPathAllowed(owner, "data", "secrets/a", false));
  EXPECT_FALSE(manifest->IsFileSystemPathAllowed(owner, "data", "safe/a", false,
                                                 "tuffite.fs.exists"));
  EXPECT_FALSE(manifest->IsNetworkUrlAllowed(
      owner, GURL("https://example.com/api/private/token")));
}

TEST_F(OriginCapabilityTest, NamespaceGroupScopesApplyOnlyToResourceMethods) {
  auto contents = std::string(kManifest);
  auto pos = contents.find("\"tuffite.window.focus\"");
  ASSERT_NE(pos, std::string::npos);
  contents.replace(pos, std::string("\"tuffite.window.focus\"").size(),
                   R"({"method":"tuffite.window.*","allow":[{"window":"*"}]})");
  pos = contents.find("\"tuffite.window.focus\":");
  ASSERT_NE(pos, std::string::npos);
  contents.insert(pos, R"("tuffite.window.current":"0000000000000000",)");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), contents));
  auto manifest = AppManifest::Load(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  const auto owner = Origin("originlab://main/");
  EXPECT_TRUE(
      manifest->IsShellApiCommandAllowed(owner, "tuffite.window.current"));
  EXPECT_TRUE(manifest->IsShellApiScopeAllowed(owner, "tuffite.window.focus",
                                               "others"));
  // Existing deny scopes retain precedence over the group grant.
  EXPECT_FALSE(manifest->IsShellApiScopeAllowed(owner, "tuffite.window.close",
                                                "others"));
  pos = contents.find(R"({"window":"*"})");
  ASSERT_NE(pos, std::string::npos);
  contents.replace(pos, std::string(R"({"window":"*"})").size(),
                   R"({"path":"data/*"})");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), contents));
  EXPECT_FALSE(AppManifest::Load(root_.GetPath(), &error_));
}

TEST_F(OriginCapabilityTest, InvalidScopesAndPatternsFailAtStartup) {
  auto invalid = std::string(kManifest);
  const auto pos = invalid.find("tuffite.path.directories");
  ASSERT_NE(pos, std::string::npos);
  invalid.replace(pos, std::string("tuffite.path.directories").size(),
                  "tuffite.path.read*");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), invalid));
  EXPECT_FALSE(AppManifest::Load(root_.GetPath(), &error_));
}

TEST_F(OriginCapabilityTest, WireCommandCannotBeDisguisedAsAllowedChannel) {
  EXPECT_TRUE(
      manifest_->IsApplicationCommandRequest("originlab.commands.allowed", 1));
  EXPECT_FALSE(
      manifest_->IsApplicationCommandRequest("originlab.commands.allowed", 2));
  EXPECT_FALSE(
      manifest_->IsApplicationCommandRequest("originlab.commands.unknown", 1));
}
TEST_F(OriginCapabilityTest, DevelopmentAliasesOnlyPackagedEntryGrants) {
  ASSERT_TRUE(manifest_->ApplyDevelopmentUrl(
      GURL("http://127.0.0.1:5183/index.html"), &error_));
  auto dev = Origin("http://127.0.0.1:5183/");
  EXPECT_TRUE(manifest_->IsExplorerOriginAllowed(dev));
  EXPECT_TRUE(
      manifest_->IsShellApiCommandAllowed(dev, "originlab.commands.allowed"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      Origin("http://127.0.0.1:5184/"), "originlab.commands.allowed"));
  EXPECT_EQ(manifest_->ExecuteJavaScriptOrigins(dev),
            std::vector<std::string>({"*"}));
}
}  // namespace
}  // namespace tuffite
