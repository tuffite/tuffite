// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <algorithm>

#include "base/files/file_enumerator.h"
#include "base/files/file_util.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_util.h"
#include "base/values.h"
#include "tuffite/framework/app/services/manifest/manifest_configuration.h"
#include "tuffite/framework/app/services/manifest/manifest_configuration_data.h"
#include "tuffite/framework/common/access.h"
#include "url/origin.h"

namespace tuffite::app {
ManifestDefinition::ManifestDefinition(std::unique_ptr<ManifestData> data)
    : data_(std::move(data)) {}
ManifestDefinition::ManifestDefinition(ManifestDefinition&&) noexcept = default;
ManifestDefinition& ManifestDefinition::operator=(
    ManifestDefinition&&) noexcept = default;
ManifestDefinition::~ManifestDefinition() = default;

namespace {
constexpr size_t kMaxManifestBytes = 1024 * 1024;
constexpr size_t kMaxNetworkResponseBytes = 5 * 1024 * 1024;
constexpr int kMaxNetworkTimeoutSeconds = 300;
constexpr int kMaxApplicationWindows = 64;
constexpr int kMaxWindowDimension = 16384;

bool IsSafeRelativePath(std::string_view value) {
  if (value.empty() || value.find('\0') != std::string_view::npos ||
      value.find('\\') != std::string_view::npos || value.front() == '/' ||
      value.find(':') != std::string_view::npos) {
    return false;
  }
  const base::FilePath path = base::FilePath::FromUTF8Unsafe(value);
  return !path.IsAbsolute() && !path.ReferencesParent();
}

bool IsCapabilityName(std::string_view value) {
  return !value.empty() && value.size() <= 128 &&
         std::ranges::all_of(value, [](char character) {
           return base::IsAsciiAlphaNumeric(character) || character == '-' ||
                  character == '_';
         });
}

bool IsWindowLabel(std::string_view value) {
  if (value.empty() || value.size() > 128) {
    return false;
  }
  return std::ranges::all_of(value, [](char character) {
    return base::IsAsciiAlphaNumeric(character) || character == '-' ||
           character == '_';
  });
}

bool IsApplicationScheme(std::string_view value) {
  constexpr std::string_view reserved[] = {"http",
                                           "https",
                                           "ws",
                                           "wss",
                                           "ftp",
                                           "file",
                                           "filesystem",
                                           "blob",
                                           "data",
                                           "javascript",
                                           "about",
                                           "chrome",
                                           "chrome-untrusted",
                                           "chrome-extension",
                                           "devtools",
                                           "view-source",
                                           "vbscript",
                                           "shell",
                                           "ms-msdt",
                                           "search-ms"};
  if (std::ranges::contains(reserved, value)) {
    return false;
  }
  if (value.empty() || !base::IsAsciiLower(value.front())) {
    return false;
  }
  return std::ranges::all_of(value, [](char character) {
    return base::IsAsciiLower(character) || base::IsAsciiDigit(character) ||
           character == '+' || character == '-' || character == '.';
  });
}

bool IsValidApplicationId(std::string_view value) {
  if (value.empty() || value.size() > 255 || value.front() == '.' ||
      value.back() == '.') {
    return false;
  }
  for (char character : value) {
    if (!(base::IsAsciiAlphaNumeric(character) || character == '.' ||
          character == '-')) {
      return false;
    }
  }
  return true;
}

std::optional<std::vector<std::string>> ParseApplicationSchemes(
    const base::DictValue& app,
    std::string* error) {
  const auto* values = app.FindList("schemes");
  std::vector<std::string> schemes;
  if (values && !values->empty() && values->size() <= 16) {
    for (const auto& value : *values) {
      const auto* scheme = value.GetIfString();
      if (!scheme || scheme->size() > 64 || !IsApplicationScheme(*scheme) ||
          std::ranges::contains(schemes, *scheme)) {
        break;
      }
      schemes.push_back(*scheme);
    }
    if (schemes.size() == values->size()) {
      return schemes;
    }
  }
  if (error) {
    *error =
        "app.schemes requires 1..16 unique lowercase non-reserved schemes (up "
        "to 64 bytes)";
  }
  return std::nullopt;
}
bool MergeConfiguration(std::string* serialized,
                        std::string_view runtime_configuration) {
  if (runtime_configuration.empty()) {
    return true;
  }
  auto root = base::JSONReader::ReadDict(*serialized, base::JSON_PARSE_RFC);
  auto runtime =
      base::JSONReader::ReadDict(runtime_configuration, base::JSON_PARSE_RFC);
  if (!root || !runtime || !root->FindDict("app") ||
      !runtime->FindDict("app") || !runtime->FindDict("browser")) {
    return false;
  }
  for (const auto* key :
       {"schemes", "assets", "windows", "network", "isolatedWorlds"}) {
    root->FindDict("app")->Remove(key);
  }
  for (const auto [key, value] : *runtime->FindDict("app")) {
    if (key == "identifier" || key == "shellApi") {
      return false;
    }
    root->FindDict("app")->Set(key, value.Clone());
  }
  root->Set("browser", runtime->FindDict("browser")->Clone());
  auto merged = base::WriteJson(*root);
  if (!merged || merged->size() > kMaxManifestBytes) {
    return false;
  }
  *serialized = std::move(*merged);
  return true;
}
}  // namespace
// static
std::optional<std::vector<std::string>> ManifestLoader::ReadApplicationSchemes(
    const base::FilePath& package_root,
    std::string* error,
    std::string_view runtime_configuration) {
  const base::FilePath manifest_path =
      package_root.Append(FILE_PATH_LITERAL("tuffite.json"));
  std::string serialized;
  if (!base::ReadFileToStringWithMaxSize(manifest_path, &serialized,
                                         kMaxManifestBytes)) {
    if (error) {
      *error = "unable to read tuffite.json or manifest exceeds 1 MiB";
    }
    return std::nullopt;
  }
  if (!MergeConfiguration(&serialized, runtime_configuration)) {
    return {};
  }
  auto parsed = base::JSONReader::Read(serialized, base::JSON_PARSE_RFC);
  const auto* app =
      parsed && parsed->is_dict() ? parsed->GetDict().FindDict("app") : nullptr;
  if (!app) {
    if (error) {
      *error = "app must be an object";
    }
    return std::nullopt;
  }
  return ParseApplicationSchemes(*app, error);
}

// static
bool ManifestLoader::ReadNetworkDisableHttp2(
    const base::FilePath& package_root,
    std::string_view runtime_configuration) {
  std::string serialized;
  if (!base::ReadFileToStringWithMaxSize(
          package_root.Append(FILE_PATH_LITERAL("tuffite.json")), &serialized,
          kMaxManifestBytes)) {
    return false;
  }
  if (!MergeConfiguration(&serialized, runtime_configuration)) {
    return {};
  }
  auto parsed = base::JSONReader::Read(serialized, base::JSON_PARSE_RFC);
  const base::DictValue* app =
      parsed && parsed->is_dict() ? parsed->GetDict().FindDict("app") : nullptr;
  const base::DictValue* network = app ? app->FindDict("network") : nullptr;
  return network && network->FindBool("disableHttp2").value_or(false);
}

// static
std::optional<ManifestDefinition> ManifestLoader::Load(
    const base::FilePath& package_root,
    std::string* error,
    std::string_view runtime_configuration) {
  auto fail =
      [error](std::string message) -> std::optional<ManifestDefinition> {
    if (error) {
      *error = std::move(message);
    }
    return std::nullopt;
  };

  const base::FilePath manifest_path =
      package_root.Append(FILE_PATH_LITERAL("tuffite.json"));
  std::string serialized;
  if (!base::ReadFileToStringWithMaxSize(manifest_path, &serialized,
                                         kMaxManifestBytes)) {
    return fail("unable to read tuffite.json or manifest exceeds 1 MiB");
  }
  if (!MergeConfiguration(&serialized, runtime_configuration)) {
    return fail("invalid application runtime configuration");
  }
  base::JSONReader::Result parsed =
      base::JSONReader::ReadAndReturnValueWithError(serialized,
                                                    base::JSON_PARSE_RFC);
  if (!parsed.has_value() || !parsed->is_dict()) {
    return fail(parsed.has_value() ? "tuffite.json root must be an object"
                                   : parsed.error().ToString());
  }
  const base::DictValue& root = parsed->GetDict();
  const base::DictValue* app = root.FindDict("app");
  const std::string* application_id =
      app ? app->FindString("identifier") : nullptr;
  const base::DictValue* shell_api = app ? app->FindDict("shellApi") : nullptr;
  const base::ListValue* windows = app ? app->FindList("windows") : nullptr;
  if (!application_id || !IsValidApplicationId(*application_id) || !shell_api ||
      !windows || windows->empty() ||
      windows->size() > kMaxApplicationWindows) {
    return fail("manifest contains a missing or invalid required field");
  }

  static constexpr std::string_view kAppKeys[] = {
      "identifier", "shellApi",       "assets", "windows",
      "network",    "isolatedWorlds", "schemes"};
  for (const auto [key, value] : *app) {
    if (!std::ranges::contains(kAppKeys, key)) {
      return fail("unknown app field: " + std::string(key));
    }
  }
  auto manifest = std::make_unique<ManifestData>();
  auto schemes = ParseApplicationSchemes(*app, error);
  if (!schemes) {
    return std::nullopt;
  }
  manifest->assets_.schemes = std::move(*schemes);
  manifest->package_root_ = package_root;
  manifest->assets_.root = package_root.AppendASCII("assets");
  const auto* browser = root.FindDict("browser");
  if (!browser || !browser->FindList("rules")) {
    return fail("browser requires a rules array");
  }
  for (const auto [key, value] : *browser) {
    if (key != "rules" && key != "devtools") {
      return fail("unknown browser field: " + std::string(key));
    }
  }
  if (const auto* devtools = browser->Find("devtools")) {
    if (!devtools->is_bool()) {
      return fail("browser.devtools must be a boolean");
    }
    manifest->devtools_enabled_ = devtools->GetBool();
  }
  manifest->application_id_ = *application_id;
  if (root.contains("productName") && !root.FindString("productName")) {
    return fail("productName must be a string");
  }
  manifest->product_name_ = root.FindString("productName")
                                ? *root.FindString("productName")
                                : *application_id;
  if (manifest->product_name_.empty() || manifest->product_name_.size() > 128 ||
      !base::IsStringUTF8(manifest->product_name_) ||
      manifest->product_name_.find_first_of("\r\n") != std::string::npos ||
      manifest->product_name_.find('\0') != std::string::npos) {
    return fail("productName is invalid");
  }
  if (const auto* value = app->Find("isolatedWorlds")) {
    const auto* worlds = value->GetIfDict();
    if (!worlds || worlds->size() > 32) {
      return fail("app.isolatedWorlds must contain at most 32 definitions");
    }
    size_t total_source_bytes = 0;
    for (const auto [name, definition] : *worlds) {
      if (!isolated::IsValidWorldName(name) || !definition.is_dict()) {
        return fail("invalid isolated world definition");
      }
      const auto& config = definition.GetDict();
      const auto* entry = config.FindString("entry");
      const auto* origins = config.FindList("origins");
      if ((config.size() != 2 && config.size() != 3) || !entry ||
          !IsSafeRelativePath(*entry) || !origins || origins->empty() ||
          origins->size() > 32) {
        return fail(
            "isolated worlds require a package-relative entry and origins");
      }
      IsolatedWorldDefinition world;
      for (const auto& origin : *origins) {
        std::string normalized;
        if (!origin.is_string() ||
            !access::NormalizeOrigin(origin.GetString(), &normalized)) {
          return fail("invalid isolated world origin");
        }
        world.origins.push_back(std::move(normalized));
      }
      const auto* isolated_directory = config.FindString("directory");
      if (config.contains("directory") &&
          (!isolated_directory || !IsSafeRelativePath(*isolated_directory))) {
        return fail("invalid isolated world directory");
      }
      auto entry_path = base::FilePath::FromUTF8Unsafe(*entry);
      const auto isolated_root =
          isolated_directory
              ? manifest->assets_.root.Append(
                    base::FilePath::FromUTF8Unsafe(*isolated_directory))
              : manifest->assets_.root.Append(entry_path.DirName());
      world.entry =
          isolated_directory ? *entry : entry_path.BaseName().AsUTF8Unsafe();
      if (!isolated::IsValidResourcePath(world.entry) ||
          !access::HasNoSymlinkComponents(manifest->assets_.root,
                                          isolated_root) ||
          !base::DirectoryExists(isolated_root)) {
        return fail("invalid isolated world package root");
      }
      base::FileEnumerator files(
          isolated_root, true,
          base::FileEnumerator::FILES | base::FileEnumerator::DIRECTORIES
#if BUILDFLAG(IS_POSIX) || BUILDFLAG(IS_FUCHSIA)
              | base::FileEnumerator::SHOW_SYM_LINKS
#endif
      );
      for (auto path = files.Next(); !path.empty(); path = files.Next()) {
        if (!access::HasNoSymlinkComponents(isolated_root, path)) {
          return fail("isolated world package contains a symlink");
        }
        if (base::DirectoryExists(path)) {
          continue;
        }
        base::FilePath relative;
        if (!isolated_root.AppendRelativePath(path, &relative)) {
          return fail("invalid isolated resource path");
        }
        auto resource_path = relative.AsUTF8Unsafe();
        std::replace(resource_path.begin(), resource_path.end(), '\\', '/');
        std::string contents;
        if (!isolated::IsValidResourcePath(resource_path) ||
            world.resources.size() >= 1024 ||
            !base::ReadFileToStringWithMaxSize(path, &contents,
                                               8 * 1024 * 1024)) {
          return fail("invalid or oversized isolated resource");
        }
        total_source_bytes += contents.size();
        if (total_source_bytes > 32 * 1024 * 1024) {
          return fail("isolated world packages exceed 32 MiB");
        }
        world.resources.emplace(resource_path, std::move(contents));
      }
      const auto source = world.resources.find(world.entry);
      if (source == world.resources.end() || source->second.empty() ||
          !base::IsStringUTF8(source->second)) {
        return fail("unable to read isolated world entry: " + *entry);
      }

      manifest->isolated_worlds_.emplace(name, std::move(world));
    }
  }
  manifest->serialized_ = std::move(serialized);
  std::vector<std::string> labels;
  for (const base::Value& value : *windows) {
    const base::DictValue* window = value.GetIfDict();
    const std::string* label = window ? window->FindString("label") : nullptr;
    const std::string* url = window ? window->FindString("url") : nullptr;
    const std::string* title = window ? window->FindString("title") : nullptr;
    const std::optional<int> width =
        window ? window->FindInt("width") : std::nullopt;
    const std::optional<int> height =
        window ? window->FindInt("height") : std::nullopt;
    const base::Value* decorations =
        window ? window->Find("decorations") : nullptr;
    const GURL window_url(url ? *url : "");
    if (!label || !IsWindowLabel(*label) ||
        std::ranges::find(labels, *label) != labels.end() || !url ||
        !window_url.is_valid() || !IsApplicationScheme(window_url.scheme()) ||
        window_url.SchemeIsHTTPOrHTTPS() || window_url.host().empty() ||
        window_url.has_username() || window_url.has_password() ||
        window_url.has_port() || !width || !height || *width < 1 ||
        *height < 1 || *width > kMaxWindowDimension ||
        *height > kMaxWindowDimension ||
        (decorations && !decorations->is_bool())) {
      return fail("app.windows contains an invalid or duplicate window");
    }
    labels.push_back(*label);
    app::AppWindow configured;
    configured.label = *label;
    configured.url = window_url;
    configured.title = title ? *title : std::string();
    configured.x = window->FindInt("x");
    configured.y = window->FindInt("y");
    if (configured.x.has_value() != configured.y.has_value()) {
      return fail("app.windows x and y must be declared together");
    }
    configured.width = *width;
    configured.height = *height;
    auto dimension = [&](const char* key, int fallback) {
      return window->FindInt(key).value_or(fallback);
    };
    configured.min_width = dimension("minWidth", 1);
    configured.min_height = dimension("minHeight", 1);
    configured.max_width = dimension("maxWidth", kMaxWindowDimension);
    configured.max_height = dimension("maxHeight", kMaxWindowDimension);
    if (configured.min_width < 1 || configured.min_height < 1 ||
        configured.max_width > kMaxWindowDimension ||
        configured.max_height > kMaxWindowDimension ||
        *width < configured.min_width || *width > configured.max_width ||
        *height < configured.min_height || *height > configured.max_height) {
      return fail("app.windows has inconsistent size constraints");
    }
    configured.focused = window->FindBool("focused").value_or(true);
    configured.decorations = window->FindBool("decorations").value_or(true);
    manifest->windows_.push_back(std::move(configured));
  }
  manifest->entry_url_ = manifest->windows_.front().url;
  manifest->app_host_ = manifest->entry_url_.host();
  if (!std::ranges::all_of(manifest->windows_, [&](const auto& window) {
        return manifest->assets_.HandlesScheme(window.url.scheme());
      })) {
    return fail("app.windows URLs must use a declared app.schemes entry");
  }
  const url::Origin entry_origin = url::Origin::Create(manifest->entry_url_);
  manifest->shell_api_origin_patterns_.push_back(entry_origin.Serialize());

  auto read_origins = [&](const base::DictValue* capability,
                          std::string_view key, std::string_view field,
                          bool url_patterns,
                          std::vector<std::string>* destination) {
    const base::ListValue* origins =
        capability ? capability->FindList(key) : nullptr;
    if (!origins) {
      return !capability || !capability->Find(key);
    }
    for (const base::Value& value : *origins) {
      const std::string* serialized_origin = value.GetIfString();
      std::string normalized;
      if (serialized_origin && url_patterns && *serialized_origin == "self") {
        destination->push_back("self");
        continue;
      }
      if (!serialized_origin ||
          !(url_patterns
                ? access::NormalizeUrl(*serialized_origin, &normalized)
                : access::NormalizeOrigin(*serialized_origin, &normalized))) {
        if (error) {
          *error = std::string(field) +
                   (url_patterns
                        ? " must contain *, legacy origins, or valid "
                          "HTTP(S) URL patterns"
                        : " must contain *, canonical nonopaque origins, or "
                          "origins with a leading *. host label");
        }
        return false;
      }
      if (std::ranges::find(*destination, normalized) == destination->end()) {
        destination->push_back(std::move(normalized));
      }
    }
    return true;
  };
  const auto* assets = app->FindList("assets");
  if (!assets) {
    return fail("app.assets must be an array");
  }
  for (const auto& entry : *assets) {
    const auto* rule = entry.GetIfDict();
    const auto* paths = rule ? rule->FindList("paths") : nullptr;
    const auto* file = rule ? rule->FindString("file") : nullptr;
    const auto* directory = rule ? rule->FindString("directory") : nullptr;
    const bool resolver = rule && rule->FindBool("resolver").value_or(false);
    const bool invalid_provider =
        rule && ((rule->Find("resolver") && !resolver) ||
                 (rule->Find("file") && !file) ||
                 (rule->Find("directory") && !directory));
    asset::AssetRoute route;
    if (invalid_provider || !rule || !paths || paths->empty() ||
        !rule->FindList("origins") || rule->FindList("origins")->empty() ||
        (int(file != nullptr) + int(directory != nullptr) + int(resolver) !=
         1) ||
        (!resolver && !IsSafeRelativePath(file ? *file : *directory)) ||
        !read_origins(rule, "origins", "app.assets.origins", false,
                      &route.origins)) {
      return fail("invalid app.assets route");
    }
    for (const auto& pattern : route.origins) {
      if (!std::ranges::any_of(manifest->assets_.schemes,
                               [&](const auto& scheme) {
                                 return pattern.starts_with(scheme + "://");
                               })) {
        return fail("assets must use a declared app.schemes entry");
      }
    }
    route.directory = directory != nullptr;
    route.resolver = resolver;
    route.source = resolver ? "" : file ? *file : *directory;
    for (const auto& value : *paths) {
      const auto* path = value.GetIfString();
      if (!path || path->empty() || path->front() != '/' ||
          path->find_first_of("?#%\\") != std::string::npos ||
          (path->find('*') != std::string::npos && !path->ends_with("/*")) ||
          (path->ends_with("/*") && path->find('*') != path->size() - 1) ||
          (path->size() > 1 && *path != "/*" &&
           !IsSafeRelativePath(path->substr(1, path->ends_with("/*")
                                                   ? path->size() - 2
                                                   : std::string::npos))) ||
          (route.directory && !path->ends_with("/*"))) {
        return fail("invalid asset path pattern");
      }
      route.path = *path;
      for (const auto& existing : manifest->assets_.routes) {
        if (existing.path != route.path) {
          continue;
        }
        for (const auto& left : existing.origins) {
          for (const auto& right : route.origins) {
            auto sample = [](std::string text) {
              const auto star = text.find("*.");
              if (star != std::string::npos) {
                text.replace(star, 2, "sample.");
              }
              return url::Origin::Create(GURL(text));
            };
            if (access::MatchesOrigin(sample(left), right) ||
                access::MatchesOrigin(sample(right), left)) {
              return fail("ambiguous app.assets route");
            }
          }
        }
      }
      manifest->assets_.routes.push_back(route);
    }
  }

  // This registry is generated from the public IDL by the package builder,
  // never supplied by renderer IPC. It binds a method name to its wire ID.
  if (const auto* ids = shell_api->FindDict("commandIds")) {
    for (const auto [name, value] : *ids) {
      const auto* text = value.GetIfString();
      uint64_t id = 0;
      if (!text || !base::HexStringToUInt64(*text, &id)) {
        return fail("shellApi.commandIds contains an invalid command ID");
      }
      manifest->shell_api_command_ids_.emplace_back(name, id);
    }
  }
  if (shell_api->contains("utilityRoutes")) {
    return fail(
        "utilityRoutes is no longer supported; register Mojom interfaces in "
        "code");
  }
  const auto* rules = browser->FindList("rules");
  if (!rules) {
    return fail("browser.rules must be an array");
  }
  // The new policy is default-deny, including the entry document.
  manifest->shell_api_origin_patterns_.clear();
  manifest->explorer_origin_patterns_.clear();
  for (const auto& rule_value : *rules) {
    const auto* rule = rule_value.GetIfDict();
    if (!rule) {
      return fail("browser rule must be an object");
    }
    const auto* identifier = rule->FindString("identifier");
    if ((rule->contains("identifier") &&
         (!identifier || !IsCapabilityName(*identifier))) ||
        !rule->FindList("origins") || rule->FindList("origins")->empty() ||
        (identifier &&
         std::ranges::any_of(manifest->origin_rules_, [&](const auto& c) {
           return c.identifier == *identifier;
         }))) {
      return fail(
          "browser rule requires origins and an optional unique identifier");
    }
    for (const auto [key, value] : *rule) {
      if (key != "identifier" && key != "description" && key != "origins" &&
          key != "shellApi" && key != "explorer" && key != "devtools" &&
          key != "csp") {
        return fail("unknown capability field: " + std::string(key));
      }
    }
    OriginRule capability;
    capability.identifier = identifier ? *identifier : std::string();
    if (const auto* devtools = rule->Find("devtools")) {
      if (!devtools->is_bool()) {
        return fail("capability devtools must be a boolean");
      }
      capability.devtools_allowed = devtools->GetBool();
    }
    if (!read_origins(rule, "origins", "capability origins", false,
                      &capability.origins)) {
      return std::nullopt;
    }
    if (const auto* csp = rule->Find("csp")) {
      if (!csp->is_string() || csp->GetString().empty() ||
          csp->GetString().size() > 4096 ||
          csp->GetString().find_first_of("\r\n") != std::string::npos ||
          csp->GetString().find('\0') != std::string::npos) {
        return fail("browser rule csp must be a nonempty header value");
      }
      capability.csp = csp->GetString();
      manifest->assets_.csp_rules.push_back(
          {capability.origins, capability.csp});
    }
    if (const auto* value = rule->Find("explorer")) {
      const auto* explorer = value->GetIfDict();
      if (!explorer) {
        return fail("explorer grant must be an object");
      }
      for (const auto [key, flag] : *explorer) {
        if (key == "behavior") {
          continue;
        }
        if (!flag.is_bool() ||
            (key != "create" && key != "executeJavaScript")) {
          return fail("invalid explorer grant");
        }
      }
      if (const auto* behavior_value = explorer->Find("behavior")) {
        const auto* behavior = behavior_value->GetIfDict();
        if (!behavior) {
          return fail("explorer.behavior must be an object");
        }
        for (const auto [name, configuration] : *behavior) {
          auto action = ParseExplorerAction(name == "externalProtocol"
                                                ? "external-protocol"
                                            : name == "newWindow" ? "new-window"
                                                                  : name);
          if (name == "external-protocol" || name == "new-window") {
            return fail("manifest behavior names must use camelCase");
          }
          const auto* object = configuration.GetIfDict();
          const auto* mode = object ? object->FindString("mode") : nullptr;
          auto parsed_mode = mode ? ParseExplorerMode(*mode) : std::nullopt;
          if (!action || !object || !parsed_mode) {
            return fail("invalid explorer behavior");
          }
          for (const auto [key, ignored] : *object) {
            if (key != "mode" && key != "timeoutMs") {
              return fail("unknown behavior parameter");
            }
          }
          ExplorerBehaviorEntry entry{.mode = *parsed_mode};
          if (const auto* timeout = object->Find("timeoutMs")) {
            if (!timeout->is_int() || timeout->GetInt() < 1 ||
                timeout->GetInt() > 300000 ||
                *parsed_mode != ExplorerMode::kDelegate ||
                *action == ExplorerAction::kNavigation) {
              return fail("invalid behavior timeoutMs");
            }
            entry.timeout_ms = timeout->GetInt();
          }
          capability.behavior.insert_or_assign(*action, entry);
        }
      }
      capability.create_explorer = explorer->FindBool("create").value_or(false);
      capability.execute_javascript =
          explorer->FindBool("executeJavaScript").value_or(false);
    }
    if (const auto* value = rule->Find("shellApi")) {
      const auto* entries = value->GetIfList();
      if (!entries) {
        return fail("shellApi grants must be an array");
      }
      for (const auto& entry : *entries) {
        const auto* object = entry.GetIfDict();
        const auto* pattern =
            object ? object->FindString("method") : entry.GetIfString();
        if (!pattern || pattern->empty()) {
          return fail("shellApi entry requires method");
        }
        if (object) {
          for (const auto [key, field_value] : *object) {
            if (key != "method" && key != "allow" && key != "deny") {
              return fail("unknown ShellAPI entry field");
            }
          }
        }
        const bool all_methods = *pattern == "*";
        const bool wildcard = all_methods || pattern->ends_with(".*");
        const auto prefix =
            wildcard ? pattern->substr(0, pattern->size() - 1) : *pattern;
        if (prefix.find('*') != std::string::npos) {
          return fail("only * and trailing .* method patterns are supported");
        }
        bool matched = false;
        for (const auto& [method, id] : manifest->shell_api_command_ids_) {
          if (wildcard ? !method.starts_with(prefix) : method != *pattern) {
            continue;
          }
          matched = true;
          ShellApiGrant grant;
          grant.method = method;

          for (const auto* key : {"allow", "deny"}) {
            const auto* scopes = object ? object->Find(key) : nullptr;
            if (!scopes) {
              continue;
            }
            if (!scopes->is_list()) {
              return fail(method + ": scope must be an array");
            }
            grant.has_allow |= std::string_view(key) == "allow";
            for (const auto& scope : scopes->GetList()) {
              const auto* fields = scope.GetIfDict();
              if (!fields || fields->size() != 1) {
                return fail(method + ": invalid scope object");
              }
              const auto scope_kind = fields->begin()->first;
              const auto* text = fields->begin()->second.GetIfString();
              if (!text ||
                  (scope_kind != "path" && scope_kind != "url" &&
                   scope_kind != "window") ||
                  (!grant.scope_kind.empty() &&
                   grant.scope_kind != scope_kind)) {
                return fail(method + ": invalid or mixed scope kind");
              }
              grant.scope_kind = scope_kind;
              std::string normalized;
              if (grant.scope_kind == "path") {
                if (!access::NormalizePath(*text, false, &normalized)) {
                  return fail(method + ": invalid path scope");
                }
              } else if (grant.scope_kind == "url") {
                if (*text != "*" && !text->starts_with("https://") &&
                    !text->starts_with("http://")) {
                  return fail(method + ": expected absolute HTTP(S) URL scope");
                }
                if (!access::NormalizeUrl(*text, &normalized)) {
                  return fail(method + ": invalid URL scope");
                }
              } else if (grant.scope_kind == "window") {
                if (*text != "current" && *text != "others" && *text != "*") {
                  return fail(method + ": invalid window scope");
                }
                normalized = *text;
              }
              (std::string_view(key) == "allow" ? grant.allow : grant.deny)
                  .push_back(std::move(normalized));
            }
          }
          capability.shell_api.push_back(std::move(grant));
        }
        if (!matched) {
          return fail("method pattern matches no registered method: " +
                      *pattern);
        }
      }
    }
    for (const auto& origin : capability.origins) {
      if (capability.create_explorer) {
        manifest->explorer_origin_patterns_.push_back(origin);
      }
      if (!capability.shell_api.empty()) {
        manifest->shell_api_origin_patterns_.push_back(origin);
      }
    }
    manifest->origin_rules_.push_back(std::move(capability));
  }
  const auto* network = app->FindDict("network");
  if (network) {
    const auto* disable = network->Find("disableHttp2");
    if (disable && !disable->is_bool()) {
      return fail("network.disableHttp2 must be a boolean");
    }
    manifest->network_disable_http2_ =
        network->FindBool("disableHttp2").value_or(false);
    if (auto bytes = network->FindInt("maxResponseBytes")) {
      if (*bytes <= 0 ||
          static_cast<size_t>(*bytes) > kMaxNetworkResponseBytes) {
        return fail("invalid network.maxResponseBytes");
      }
      manifest->network_max_response_bytes_ = *bytes;
    }
    if (auto seconds = network->FindInt("timeoutSeconds")) {
      if (*seconds <= 0 || *seconds > kMaxNetworkTimeoutSeconds) {
        return fail("invalid network.timeoutSeconds");
      }
      manifest->network_timeout_seconds_ = *seconds;
    }
  }
  return ManifestDefinition(std::move(manifest));
}
}  // namespace tuffite::app
