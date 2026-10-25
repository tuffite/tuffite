// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "base/files/file_util.h"
#include "base/files/scoped_temp_dir.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "testing/gtest/include/gtest/gtest.h"
#include "tuffite/framework/app/services/manifest/manifest_service_factory.h"
#include "tuffite/framework/browser/services/isolated/isolated_registry.h"
#include "url/url_util.h"

namespace tuffite {
namespace {
std::unique_ptr<ManifestService> LoadManifest(
    const base::FilePath& root,
    std::string* error,
    const app::StartupEnvironment& environment = {},
    const app::BrowserOptions& options = {}) {
  auto definition = app::ManifestLoader::Load(root, error);
  if (!definition) {
    return nullptr;
  }
  auto configuration = app::ConfigurationAssembler::Build(
      std::move(*definition), environment, options, error);
  return configuration
             ? ManifestServiceFactory::Create(std::move(*configuration))
             : nullptr;
}

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
              "method": "tuffite.Fs.ReadFile",
              "allow": [
                {
                  "path": "resource/allowed/*"
                }
              ]
            },
            {
              "method": "tuffite.Fs.Exists",
              "allow": [
                {
                  "path": "resource/allowed/*"
                }
              ]
            },
            {
              "method": "tuffite.Fs.WriteFile",
              "allow": [
                {
                  "path": "data/safe/*"
                }
              ]
            },
            {
              "method": "tuffite.Fs.CreateDir",
              "allow": [
                {
                  "path": "data/safe/*"
                }
              ]
            },
            {
              "method": "tuffite.Fs.RemoveFile",
              "allow": [
                {
                  "path": "data/safe/*"
                }
              ]
            },
            {
              "method": "tuffite.Net.Request",
              "allow": [
                {
                  "url": "https://example.com/api/*"
                }
              ]
            },
            {
              "method": "tuffite.Fs.ReadFile",
              "allow": [
                {
                  "path": "data/**"
                }
              ]
            },
            "tuffite.Window.Focus",
            "tuffite.Window.Close",
            {
              "method": "tuffite.Window.Close",
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
            "tuffite.Path.Directories"
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
              "method": "tuffite.Fs.ReadFile",
              "deny": [
                {
                  "path": "data/secrets/**"
                }
              ]
            },
            {
              "method": "tuffite.Window.Close",
              "deny": [
                {
                  "window": "others"
                }
              ]
            },
            {
              "method": "tuffite.Net.Request",
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

      "commandIds": {
        "tuffite.Path.Directories": "0000000000000000",
        "originlab.commands.allowed": "0000000000000001",
        "originlab.commands.denied": "0000000000000002",
        "tuffite.Fs.ReadFile": "0000000000000000",
        "tuffite.Fs.Exists": "0000000000000000",
        "tuffite.Fs.WriteFile": "0000000000000000",
        "tuffite.Fs.CreateDir": "0000000000000000",
        "tuffite.Fs.RemoveFile": "0000000000000000",
        "tuffite.Net.Request": "0000000000000000",
        "tuffite.Window.Focus": "0000000000000000",
        "tuffite.Window.Close": "0000000000000000"
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
    manifest_ = LoadManifest(root_.GetPath(), &error_);
    ASSERT_TRUE(manifest_) << error_;
  }
  url::Origin Origin(const char* value) {
    return url::Origin::Create(GURL(value));
  }

 protected:
  url::ScopedSchemeRegistryForTests registry_;
  base::ScopedTempDir root_;
  std::unique_ptr<ManifestService> manifest_;
  std::string error_;
};
TEST_F(OriginCapabilityTest, IsolatedPackagesAreImmutableAndOriginScoped) {
  const auto assets = root_.GetPath().AppendASCII("assets");
  ASSERT_TRUE(base::CreateDirectory(assets));
  const auto source = assets.AppendASCII("card.js");
  ASSERT_TRUE(base::WriteFile(source, "globalThis.fixture = 1;"));
  auto parsed = base::JSONReader::Read(kManifest, base::JSON_PARSE_RFC);
  ASSERT_TRUE(parsed);
  base::DictValue definition;
  definition.Set("entry", "card.js");
  base::ListValue origins;
  origins.Append("originlab://main");
  definition.Set("origins", std::move(origins));
  base::DictValue worlds;
  worlds.Set("io.example.visual-blocks", std::move(definition));
  parsed->GetDict().FindDict("app")->Set("isolatedWorlds", std::move(worlds));
  ASSERT_TRUE(base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"),
                              *base::WriteJson(*parsed)));
  auto manifest = LoadManifest(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  ASSERT_TRUE(base::WriteFile(source, "changed"));
  IsolatedRegistry registry;
  for (const auto& [name, snapshot] : manifest->IsolatedWorldDefinitions()) {
    registry.AddBundled(name, snapshot);
  }
  EXPECT_EQ(registry.Source("io.example.visual-blocks", Origin("originlab://main")),
            "globalThis.fixture = 1;");
  EXPECT_FALSE(registry.Source("io.example.visual-blocks", Origin("originlab://public")));
  EXPECT_FALSE(registry.Source("unknown", Origin("originlab://main")));
  parsed->GetDict()
      .FindDict("app")
      ->FindDict("isolatedWorlds")
      ->FindDict("io.example.visual-blocks")
      ->Set("entry", "../card.js");
  ASSERT_TRUE(base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"),
                              *base::WriteJson(*parsed)));
  EXPECT_FALSE(LoadManifest(root_.GetPath(), &error_));
}

TEST_F(OriginCapabilityTest, ProductNameDefaultsToApplicationIdentity) {
  EXPECT_EQ(manifest_->app_info().product_name, "com.example.originlab");
}

TEST_F(OriginCapabilityTest, MojomModulesDoNotDependOnTheAssetScheme) {
  auto text = std::string(kManifest);
  // Keep originlab:// URLs while the generated table contains two unrelated
  // modules. Method identity and interface routing come only from that table.
  const std::string marker =
      "\"originlab.commands.allowed\": \"0000000000000001\",";
  const auto at = text.find(marker);
  ASSERT_NE(at, std::string::npos);
  text.insert(at, "\"other.files.Fs.Read\": \"0000000000000011\",\n");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), text));
  auto manifest = LoadManifest(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  EXPECT_EQ(manifest->app_info().scheme, "originlab");
  EXPECT_EQ(manifest->ApplicationInterface("other.files.Fs.Read"),
            "other.files.Fs");
  EXPECT_TRUE(manifest->IsApplicationCommandRequest("other.files.Fs.Read", 17));
  EXPECT_TRUE(manifest->IsApplicationInterface("originlab.commands"));
  EXPECT_TRUE(manifest->IsApplicationInterface("other.files.Fs"));
  EXPECT_TRUE(
      manifest->ApplicationInterface("tuffite.Path.Directories").empty());
  EXPECT_TRUE(manifest->ApplicationInterface("other.files.Fs.Unknown").empty());
  EXPECT_FALSE(manifest->IsAllShellApiAllowed(Origin("originlab://main")));
}

TEST_F(OriginCapabilityTest, ProductNameIsIndependentFromIdentity) {
  auto text = std::string(kManifest);
  text.insert(1, "\"productName\":\"Origin Lab\",");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), text));
  auto manifest = LoadManifest(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  EXPECT_EQ(manifest->app_info().product_name, "Origin Lab");
  EXPECT_EQ(manifest->app_info().identifier, "com.example.originlab");
}

TEST_F(OriginCapabilityTest, InvalidProductNamesAreRejected) {
  for (const auto* value : {"7", "null", "\"\"", "\"bad\\nname\""}) {
    auto text = std::string(kManifest);
    text.insert(1, std::string("\"productName\":") + value + ",");
    ASSERT_TRUE(
        base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), text));
    EXPECT_FALSE(LoadManifest(root_.GetPath(), &error_)) << value;
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
  auto manifest = LoadManifest(root_.GetPath(), &error_);
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
  EXPECT_FALSE(LoadManifest(root_.GetPath(), &error_));
}
TEST_F(OriginCapabilityTest, UnionGrantsRemainOriginScoped) {
  auto artifact = Origin("originlab://demo.artifact/page");
  auto preview = Origin("originlab://demo.preview/page");
  EXPECT_NE(artifact, preview);
  EXPECT_TRUE(manifest_->IsShellApiCommandAllowed(artifact,
                                                  "tuffite.Path.Directories"));
  EXPECT_TRUE(
      manifest_->IsShellApiCommandAllowed(preview, "tuffite.Path.Directories"));
  EXPECT_TRUE(manifest_->IsShellApiCommandAllowed(
      artifact, "originlab.commands.allowed"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      preview, "originlab.commands.allowed"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      artifact, "originlab.commands.denied"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(Origin("originlab://other/"),
                                                   "tuffite.Path.Directories"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      Origin("data:text/html,hello"), "tuffite.Path.Directories"));
  EXPECT_FALSE(manifest_->IsExplorerOriginAllowed(artifact));
  EXPECT_TRUE(manifest_->IsExplorerOriginAllowed(Origin("originlab://main/")));
  EXPECT_TRUE(manifest_->assets().IsAppUrl(
      GURL("originlab://demo.preview/index.html")));
  EXPECT_FALSE(
      manifest_->assets().IsAppUrl(GURL("originlab://unknown/index.html")));
  // Chromium registers application schemes as host-only origins.
  EXPECT_TRUE(manifest_->assets().IsAppUrl(
      GURL("originlab://demo.preview:443/index.html")));
}
TEST_F(OriginCapabilityTest,
       AssetsAndFrameworkScopesAreIndependentOfBusinessGrants) {
  const auto app = Origin("originlab://main/");
  const auto public_origin = Origin("originlab://public/");
  EXPECT_TRUE(
      manifest_->assets().IsAppUrl(GURL("originlab://public/index.html")));
  EXPECT_FALSE(manifest_->IsShellApiOriginAllowed(public_origin));
  EXPECT_FALSE(manifest_->IsExplorerOriginAllowed(public_origin));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(app, "tuffite.Storage.Get"));
  EXPECT_TRUE(
      manifest_->IsShellApiCommandAllowed(app, "originlab.commands.allowed"));
  EXPECT_TRUE(manifest_->ResolveFileSystemPath(app, "resource", "allowed/a",
                                               false, "tuffite.Fs.ReadFile"));
  EXPECT_FALSE(manifest_->ResolveFileSystemPath(
      public_origin, "resource", "allowed/a", false, "tuffite.Fs.ReadFile"));
  EXPECT_FALSE(manifest_->ResolveFileSystemPath(app, "resource", "private/a",
                                                false, "tuffite.Fs.ReadFile"));
  EXPECT_TRUE(manifest_->IsNetworkUrlAllowed(
      app, GURL("https://example.com/api/items")));
  EXPECT_FALSE(manifest_->IsNetworkUrlAllowed(
      public_origin, GURL("https://example.com/api/items")));
  EXPECT_FALSE(
      manifest_->IsNetworkUrlAllowed(app, GURL("https://example.com/private")));
  EXPECT_EQ(
      manifest_->assets().ResolveAssetRoute(GURL("originlab://main/exact")),
      "/ui/index.html");
  EXPECT_EQ(
      manifest_->assets().ResolveAssetRoute(GURL("originlab://main/app/deep")),
      "/ui/index.html");
  EXPECT_EQ(manifest_->assets().ResolveAssetRoute(
                GURL("originlab://public/app/deep")),
            "/ui/app/deep");
  EXPECT_EQ(
      manifest_->assets().ResolveAssetRoute(GURL("originlab://entryonly/home")),
      "/ui/index.html");
  EXPECT_TRUE(manifest_->assets()
                  .ResolveAssetRoute(GURL("originlab://entryonly/missing"))
                  .empty());
  EXPECT_TRUE(manifest_->assets()
                  .ResolveAssetRoute(GURL("originlab://entryonly/"))
                  .empty());
}
TEST_F(OriginCapabilityTest, PackagedModulePathsReuseOriginAndRouteChecks) {
  auto path =
      manifest_->assets().ResolveAssetFile(GURL("originlab://main/exact"));
  ASSERT_TRUE(path);
  EXPECT_EQ(*path, manifest_->assets().root.AppendASCII("ui/index.html"));
  EXPECT_FALSE(
      manifest_->assets().ResolveAssetFile(GURL("https://example.com/exact")));
  EXPECT_FALSE(
      manifest_->assets().ResolveAssetFile(GURL("file:///tmp/exact.html")));
  EXPECT_FALSE(
      manifest_->assets().ResolveAssetFile(GURL("originlab://unknown/exact")));
  EXPECT_FALSE(manifest_->assets().ResolveAssetFile(
      GURL("originlab://entryonly/missing")));
  // A fixed-file route intentionally ignores the request's suffix.
  EXPECT_EQ(manifest_->assets().ResolveAssetFile(
                GURL("originlab://main/app/%2e%2e%2fsecret.mjs")),
            path);
  // A directory route must reject decoded parent components.
  EXPECT_FALSE(manifest_->assets().ResolveAssetFile(
      GURL("originlab://public/app/%2e%2e%2fsecret.mjs")));
}

TEST_F(OriginCapabilityTest, OwnerNavigationDoesNotInheritAssetAuthority) {
  const auto owner = Origin("originlab://main/");
  EXPECT_TRUE(
      manifest_->assets().IsAppUrl(GURL("originlab://public/index.html")));
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
  EXPECT_TRUE(manifest_->ResolveFileSystemPath(owner, "data", "safe/a", false,
                                               "tuffite.Fs.ReadFile"));
  EXPECT_FALSE(manifest_->ResolveFileSystemPath(owner, "data", "secrets/a",
                                                false, "tuffite.Fs.ReadFile"));
  EXPECT_FALSE(manifest_->ResolveFileSystemPath(owner, "data", "safe/a", false,
                                                "tuffite.Fs.Exists"));
  EXPECT_TRUE(manifest_->IsShellApiScopeAllowed(owner, "tuffite.Window.Focus",
                                                "current", "window"));
  EXPECT_FALSE(manifest_->IsShellApiScopeAllowed(owner, "tuffite.Window.Focus",
                                                 "others", "window"));
  EXPECT_FALSE(manifest_->IsShellApiScopeAllowed(owner, "tuffite.Window.Close",
                                                 "others", "window"));
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
  auto manifest = LoadManifest(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  const auto owner = Origin("originlab://main/");
  EXPECT_TRUE(
      manifest->IsShellApiCommandAllowed(owner, "tuffite.Path.Directories"));
  EXPECT_FALSE(manifest->IsShellApiCommandAllowed(owner, "tuffite.unknown"));
  EXPECT_FALSE(manifest->IsShellApiCommandAllowed(Origin("originlab://public/"),
                                                  "tuffite.Path.Directories"));
  EXPECT_TRUE(manifest->ResolveFileSystemPath(owner, "data", "safe/a", false,
                                              "tuffite.Fs.ReadFile"));
  EXPECT_FALSE(manifest->ResolveFileSystemPath(owner, "data", "secrets/a",
                                               false, "tuffite.Fs.ReadFile"));
  EXPECT_FALSE(manifest->ResolveFileSystemPath(owner, "data", "safe/a", false,
                                               "tuffite.Fs.Exists"));
  EXPECT_FALSE(manifest->IsNetworkUrlAllowed(
      owner, GURL("https://example.com/api/private/token")));
}

TEST_F(OriginCapabilityTest, NamespaceGroupScopesAreValidatedByTheApi) {
  auto contents = std::string(kManifest);
  auto pos = contents.find("\"tuffite.Window.Focus\"");
  ASSERT_NE(pos, std::string::npos);
  contents.replace(pos, std::string("\"tuffite.Window.Focus\"").size(),
                   R"({"method":"tuffite.Window.*","allow":[{"window":"*"}]})");
  pos = contents.find("\"tuffite.Window.Focus\":");
  ASSERT_NE(pos, std::string::npos);
  contents.insert(pos, R"("tuffite.Window.Current":"0000000000000000",)");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), contents));
  auto manifest = LoadManifest(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  const auto owner = Origin("originlab://main/");
  EXPECT_TRUE(
      manifest->IsShellApiCommandAllowed(owner, "tuffite.Window.Current"));
  EXPECT_FALSE(manifest->IsShellApiScopeAllowed(owner, "tuffite.Window.Current",
                                                "", ""));
  EXPECT_TRUE(manifest->IsShellApiScopeAllowed(owner, "tuffite.Window.Focus",
                                               "others", "window"));
  // Existing deny scopes retain precedence over the group grant.
  EXPECT_FALSE(manifest->IsShellApiScopeAllowed(owner, "tuffite.Window.Close",
                                                "others", "window"));
  pos = contents.find(R"({"window":"*"})");
  ASSERT_NE(pos, std::string::npos);
  contents.replace(pos, std::string(R"({"window":"*"})").size(),
                   R"({"path":"data/*"})");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), contents));
  manifest = LoadManifest(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  EXPECT_FALSE(manifest->IsShellApiScopeAllowed(owner, "tuffite.Window.Focus",
                                                "others", "window"));
  EXPECT_FALSE(manifest->IsShellApiScopeAllowed(owner, "tuffite.Window.Current",
                                                "", ""));
}

TEST_F(OriginCapabilityTest, InvalidScopesAndPatternsFailAtStartup) {
  auto invalid = std::string(kManifest);
  const auto pos = invalid.find("tuffite.Path.Directories");
  ASSERT_NE(pos, std::string::npos);
  invalid.replace(pos, std::string("tuffite.Path.Directories").size(),
                  "tuffite.Path.Read*");
  ASSERT_TRUE(
      base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), invalid));
  EXPECT_FALSE(LoadManifest(root_.GetPath(), &error_));
}

TEST_F(OriginCapabilityTest, InterfaceIdentityAndMethodPermissions) {
  EXPECT_EQ(manifest_->ApplicationInterface("originlab.commands.allowed"),
            "originlab.commands");
  EXPECT_EQ(manifest_->ApplicationInterface("originlab.commands.missing"), "");
  EXPECT_TRUE(manifest_->IsApplicationInterface("originlab.commands"));
  EXPECT_FALSE(manifest_->IsApplicationInterface("originlab"));
  EXPECT_FALSE(manifest_->IsApplicationInterface("echo"));
  EXPECT_TRUE(manifest_->IsShellApiCommandAllowed(
      Origin("originlab://demo.artifact"), "originlab.commands.allowed"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      Origin("originlab://demo.artifact"), "originlab.commands.denied"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      Origin("originlab://other"), "originlab.commands.allowed"));
}

TEST_F(OriginCapabilityTest, RejectsLegacyRouteAndServiceGrants) {
  auto config = base::JSONReader::ReadDict(kManifest, base::JSON_PARSE_RFC);
  ASSERT_TRUE(config);
  auto load = [&]() {
    const auto json = base::WriteJson(*config);
    EXPECT_TRUE(json);
    EXPECT_TRUE(
        base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"), *json));
    return LoadManifest(root_.GetPath(), &error_);
  };
  config->SetByDottedPath("app.shellApi.utilityRoutes", base::DictValue());
  EXPECT_FALSE(load());
  config->FindDictByDottedPath("app.shellApi")->Remove("utilityRoutes");
  auto& grants = *config->FindDictByDottedPath("app.security")
                      ->FindList("capabilities")
                      ->front()
                      .GetDict()
                      .FindList("shellApi");
  grants.front() = base::Value(base::DictValue()
                                   .Set("method", "originlab.commands.allowed")
                                   .Set("service", "echo"));
  EXPECT_FALSE(load());
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
  manifest_ = LoadManifest(
      root_.GetPath(), &error_,
      {.development_url = GURL("http://127.0.0.1:5183/index.html")});
  ASSERT_TRUE(manifest_) << error_;
  auto dev = Origin("http://127.0.0.1:5183/");
  EXPECT_TRUE(manifest_->IsExplorerOriginAllowed(dev));
  EXPECT_TRUE(
      manifest_->IsShellApiCommandAllowed(dev, "originlab.commands.allowed"));
  EXPECT_FALSE(manifest_->IsShellApiCommandAllowed(
      Origin("http://127.0.0.1:5184/"), "originlab.commands.allowed"));
  EXPECT_EQ(manifest_->ExecuteJavaScriptOrigins(dev),
            std::vector<std::string>({"*"}));
}
TEST_F(OriginCapabilityTest, RejectsInvalidDevelopmentUrlBeforePublication) {
  EXPECT_FALSE(
      LoadManifest(root_.GetPath(), &error_,
                   {.development_url = GURL("https://untrusted.example/")}));
  EXPECT_FALSE(error_.empty());
}
TEST(ManifestBootstrapTest, ReadsBeforeApplicationSchemeRegistration) {
  base::ScopedTempDir root;
  ASSERT_TRUE(root.CreateUniqueTempDir());
  ASSERT_TRUE(base::WriteFile(
      root.GetPath().AppendASCII("tuffite.json"),
      R"({"app":{"windows":[{"url":"bootstraponly://main/index.html"}],"network":{"disableHttp2":true}}})"));
  std::string error;
  EXPECT_EQ(app::ManifestLoader::ReadApplicationScheme(root.GetPath(), &error),
            std::make_optional(std::string("bootstraponly")));
  EXPECT_TRUE(app::ManifestLoader::ReadNetworkDisableHttp2(root.GetPath()));
  ASSERT_TRUE(base::WriteFile(
      root.GetPath().AppendASCII("tuffite.json"),
      R"({"app":{"windows":[{"url":"https://example.com/"}]}})"));
  EXPECT_FALSE(
      app::ManifestLoader::ReadApplicationScheme(root.GetPath(), &error));
  EXPECT_FALSE(app::ManifestLoader::ReadNetworkDisableHttp2(root.GetPath()));
}
TEST_F(OriginCapabilityTest, DevToolsDenialsAreOriginScopedAndOverrideAllows) {
  auto text = base::JSONReader::ReadDict(kManifest, base::JSON_PARSE_RFC);
  ASSERT_TRUE(text);
  auto* capabilities =
      text->FindDict("app")->FindDict("security")->FindList("capabilities");
  capabilities->Append(
      base::DictValue()
          .Set("identifier", "deny-inspection")
          .Set("origins", base::ListValue().Append("originlab://main"))
          .Set("devtools", false));
  capabilities->Append(
      base::DictValue()
          .Set("identifier", "allow-inspection")
          .Set("origins", base::ListValue().Append("originlab://main"))
          .Set("devtools", true));
  ASSERT_TRUE(base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"),
                              *base::WriteJson(*text)));
  auto manifest = LoadManifest(root_.GetPath(), &error_);
  ASSERT_TRUE(manifest) << error_;
  EXPECT_FALSE(manifest->IsDevToolsAllowed(Origin("originlab://main/main")));
  EXPECT_TRUE(manifest->IsDevToolsAllowed(Origin("https://example.com/")));
  auto development =
      LoadManifest(root_.GetPath(), &error_,
                   {.development_url = GURL("http://127.0.0.1:5173/main")});
  ASSERT_TRUE(development) << error_;
  EXPECT_FALSE(
      development->IsDevToolsAllowed(Origin("http://127.0.0.1:5173/")));
}

TEST_F(OriginCapabilityTest, BrowserSetupAddsDenialsAndRejectsInvalidPatterns) {
  std::string error;
  app::BrowserOptions options;
  options.security.devtools.denied_origins = {"originlab://main"};
  auto manifest = LoadManifest(root_.GetPath(), &error, {}, options);
  ASSERT_TRUE(manifest) << error;
  EXPECT_FALSE(manifest->IsDevToolsAllowed(Origin("originlab://main/main")));
  EXPECT_TRUE(manifest->IsDevToolsAllowed(Origin("https://example.com/")));
  auto development = LoadManifest(
      root_.GetPath(), &error,
      {.development_url = GURL("http://127.0.0.1:5173/")}, options);
  ASSERT_TRUE(development) << error;
  EXPECT_FALSE(
      development->IsDevToolsAllowed(Origin("http://127.0.0.1:5173/")));
  options.security.devtools.denied_origins = {"not an origin"};
  EXPECT_FALSE(LoadManifest(root_.GetPath(), &error, {}, options));
}

TEST_F(OriginCapabilityTest,
       EffectiveConfigurationOwnsInputsBeforeServiceCreation) {
  auto document = base::JSONReader::ReadDict(kManifest, base::JSON_PARSE_RFC);
  ASSERT_TRUE(document);
  document->SetByDottedPath("app.utilities.echo.configuration.mode", "local");
  ASSERT_TRUE(base::WriteFile(root_.GetPath().AppendASCII("tuffite.json"),
                              *base::WriteJson(*document)));
  auto definition = app::ManifestLoader::Load(root_.GetPath(), &error_);
  ASSERT_TRUE(definition) << error_;
  app::StartupEnvironment environment{.development_url =
                                          GURL("http://127.0.0.1:5173/main")};
  app::BrowserOptions options;
  options.security.devtools.denied_origins = {"originlab://main"};
  auto configuration = app::ConfigurationAssembler::Build(
      std::move(*definition), environment, options, &error_);
  ASSERT_TRUE(configuration) << error_;

  // Creating/querying the service uses only the frozen snapshot, not caller
  // options or the package file. Runtime bytes retain the package declaration.
  environment.development_url = GURL("http://127.0.0.1:6000/");
  options.security.devtools.denied_origins.clear();
  ASSERT_TRUE(base::DeleteFile(root_.GetPath().AppendASCII("tuffite.json")));
  auto service = ManifestServiceFactory::Create(std::move(*configuration));
  EXPECT_EQ(service->startup_windows().front().url,
            GURL("http://127.0.0.1:5173/main"));
  EXPECT_FALSE(service->IsDevToolsAllowed(Origin("originlab://main")));
  EXPECT_FALSE(service->IsDevToolsAllowed(Origin("http://127.0.0.1:5173")));
  EXPECT_TRUE(service->IsDevToolsAllowed(Origin("http://127.0.0.1:6000")));
  auto serialized = base::JSONReader::ReadDict(
      service->SerializeRuntimeConfiguration(), base::JSON_PARSE_RFC);
  ASSERT_TRUE(serialized);
  EXPECT_EQ(*serialized->FindDict("app")
                 ->FindList("windows")
                 ->front()
                 .GetDict()
                 .FindString("url"),
            "originlab://main/index.html");
  const auto utilities = service->UtilityConfigurations();
  ASSERT_TRUE(utilities.contains("echo"));
  const auto& bytes = utilities.at("echo");
  EXPECT_EQ(std::string(bytes.begin(), bytes.end()), R"({"mode":"local"})");
}

TEST_F(OriginCapabilityTest, HostRestrictionsDoNotIntroduceCapabilityGrants) {
  app::BrowserOptions options;
  options.security.devtools.denied_origins = {"https://*.example.com"};
  auto service = LoadManifest(root_.GetPath(), &error_, {}, options);
  ASSERT_TRUE(service) << error_;
  const auto remote = Origin("https://private.example.com");
  EXPECT_FALSE(service->IsDevToolsAllowed(remote));
  EXPECT_FALSE(service->IsExplorerOriginAllowed(remote));
  EXPECT_FALSE(service->IsShellApiOriginAllowed(remote));
  EXPECT_TRUE(service->IsShellApiOriginAllowed(Origin("originlab://main")));
}

}  // namespace
}  // namespace tuffite

namespace tuffite {
namespace {
base::DictValue RegistryRequest(const char* json) {
  return *base::JSONReader::ReadDict(json, base::JSON_PARSE_RFC);
}
TEST(IsolatedRegistryTest, GenerationAndOriginAuthorization) {
  IsolatedRegistry registry;
  auto request = RegistryRequest(R"({"op":"register","name":"io.example.card",
      "source":"original","origins":["https://main.example"]})");
  auto first = registry.Apply(request);
  ASSERT_TRUE(first);
  EXPECT_FALSE(registry.Apply(request));
  registry.SetDevelopmentAlias(
      url::Origin::Create(GURL("https://main.example")),
      url::Origin::Create(GURL("http://localhost:5173")));
  EXPECT_EQ(registry.Source("io.example.card",
                            url::Origin::Create(GURL("http://localhost:5173"))),
            "original");
  EXPECT_EQ(registry.Source("io.example.card",
                            url::Origin::Create(GURL("https://main.example"))),
            "original");
  EXPECT_FALSE(registry.Source(
      "io.example.card", url::Origin::Create(GURL("https://other.example"))));
  request.Set("op", "replace");
  request.Set("generation", *first);
  request.Set("source", "replacement");
  auto second = registry.Apply(request);
  ASSERT_TRUE(second);
  EXPECT_NE(first, second);
  EXPECT_FALSE(registry.Apply(request));
  auto remove =
      RegistryRequest(R"({"op":"unregister","name":"io.example.card"})");
  remove.Set("generation", *first);
  EXPECT_FALSE(registry.Apply(remove));
  EXPECT_EQ(registry.Source("io.example.card",
                            url::Origin::Create(GURL("https://main.example"))),
            "replacement");
  remove.Set("generation", *second);
  EXPECT_TRUE(registry.Apply(remove));
  EXPECT_FALSE(registry.Source(
      "io.example.card", url::Origin::Create(GURL("https://main.example"))));
  EXPECT_FALSE(registry.Apply(remove));
  request.Set("op", "register");
  auto third = registry.Apply(request);
  ASSERT_TRUE(third);
  EXPECT_NE(second, third);
}
TEST(IsolatedRegistryTest, ValidationQuotaAndBundledProtection) {
  IsolatedRegistry registry;
  registry.AddBundled("bundled", {{"https://main.example"}, "bundled"});
  auto request = RegistryRequest(R"({"op":"register","name":"dynamic",
      "source":"dynamic","origins":["*"]})");
  EXPECT_FALSE(registry.Apply(request));
  request.Set("origins", base::ListValue().Append("https://main.example"));
  request.Set("source", std::string(8 * 1024 * 1024 + 1, 'a'));
  EXPECT_FALSE(registry.Apply(request));
  request.Set("source", "valid");
  request.Set("name", "../escape");
  EXPECT_FALSE(registry.Apply(request));
  request.Set("name", "bundled");
  request.Set("op", "replace");
  request.Set("generation", "1");
  EXPECT_FALSE(registry.Apply(request));
  request.Set("op", "unregister");
  EXPECT_FALSE(registry.Apply(request));
  request.Set("op", "register");
  for (int i = 0; i < 31; ++i) {
    request.Set("name", "dynamic" + std::to_string(i));
    ASSERT_TRUE(registry.Apply(request));
  }
  request.Set("name", "overflow");
  EXPECT_FALSE(registry.Apply(request));
}
}  // namespace
}  // namespace tuffite

namespace tuffite {
TEST(IsolatedRegistryTest, AggregateLimitPreservesPreviousGeneration) {
  IsolatedRegistry registry;
  auto request = *base::JSONReader::ReadDict(
      R"({"op":"register","name":"card","source":"x","origins":["https://main.example"]})",
      base::JSON_PARSE_RFC);
  auto generation = registry.Apply(request);
  ASSERT_TRUE(generation);
  request.Set("source", std::string(8 * 1024 * 1024, 'a'));
  for (int i = 0; i < 3; ++i) {
    request.Set("name", "large" + std::to_string(i));
    ASSERT_TRUE(registry.Apply(request));
  }
  request.Set("name", "overflow");
  EXPECT_FALSE(registry.Apply(request));
  request.Set("name", "card");
  request.Set("op", "replace");
  request.Set("generation", *generation);
  request.Set("source", std::string(8 * 1024 * 1024 + 1, 'b'));
  EXPECT_FALSE(registry.Apply(request));
  EXPECT_EQ(registry.Source("card",
                            url::Origin::Create(GURL("https://main.example"))),
            "x");
  request.Set("op", "unregister");
  EXPECT_TRUE(registry.Apply(request));
}
}  // namespace tuffite
