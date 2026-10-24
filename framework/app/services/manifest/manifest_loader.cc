// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <algorithm>

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

bool IsIdentifier(std::string_view value) {
  if (value.empty() || value.size() > 128 ||
      !(base::IsAsciiAlpha(value.front()) || value.front() == '_' ||
        value.front() == '$')) {
    return false;
  }
  for (char character : value.substr(1)) {
    if (!(base::IsAsciiAlphaNumeric(character) || character == '_' ||
          character == '$')) {
      return false;
    }
  }
  return true;
}

bool IsCapabilityName(std::string_view value) {
  return !value.empty() && value.size() <= 128 &&
         std::ranges::all_of(value, [](char character) {
           return base::IsAsciiAlphaNumeric(character) || character == '-' ||
                  character == '_';
         });
}

bool IsUtilityName(std::string_view value) {
  if (value.size() > 128) {
    return false;
  }
  while (true) {
    const size_t separator = value.find_first_of(".:");
    if (!IsIdentifier(value.substr(0, separator))) {
      return false;
    }
    if (separator == std::string_view::npos) {
      return true;
    }
    if (value[separator] == ':') {
      if (value.substr(separator, 2) != "::") {
        return false;
      }
      value.remove_prefix(separator + 2);
    } else {
      value.remove_prefix(separator + 1);
    }
  }
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

}  // namespace
// static
std::optional<std::string> ManifestLoader::ReadApplicationScheme(
    const base::FilePath& package_root,
    std::string* error) {
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
  auto parsed = base::JSONReader::Read(serialized, base::JSON_PARSE_RFC);
  const base::ListValue* windows =
      parsed && parsed->is_dict() && parsed->GetDict().FindDict("app")
          ? parsed->GetDict().FindDict("app")->FindList("windows")
          : nullptr;
  const base::DictValue* first =
      windows && !windows->empty() ? (*windows)[0].GetIfDict() : nullptr;
  const std::string* url = first ? first->FindString("url") : nullptr;
  if (!url) {
    if (error) {
      *error = "app.windows[0].url is required";
    }
    return std::nullopt;
  }
  const size_t separator = url->find("://");
  const std::string scheme = separator == std::string::npos
                                 ? std::string()
                                 : url->substr(0, separator);
  if (!IsApplicationScheme(scheme) || scheme == "http" || scheme == "https") {
    if (error) {
      *error = "app.windows[0].url has an invalid application scheme";
    }
    return std::nullopt;
  }
  return scheme;
}

// static
bool ManifestLoader::ReadNetworkDisableHttp2(
    const base::FilePath& package_root) {
  std::string serialized;
  if (!base::ReadFileToStringWithMaxSize(
          package_root.Append(FILE_PATH_LITERAL("tuffite.json")), &serialized,
          kMaxManifestBytes)) {
    return false;
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
    std::string* error) {
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
      "identifier", "security", "shellApi", "assets",
      "windows",    "network",  "utilities"};
  for (const auto [key, value] : *app) {
    if (!std::ranges::contains(kAppKeys, key)) {
      return fail("unknown app field: " + std::string(key));
    }
  }
  auto manifest = std::make_unique<ManifestData>();
  manifest->package_root_ = package_root;
  manifest->assets_.root = package_root.AppendASCII("assets");
  const base::DictValue* security = app ? app->FindDict("security") : nullptr;
  if (!security) {
    return fail("app.security must be an object");
  }
  if (const std::string* policy =
          security ? security->FindString("csp") : nullptr) {
    if (policy->empty() || policy->size() > 4096 ||
        policy->find_first_of("\r\n") != std::string::npos) {
      return fail("app.security.csp is invalid");
    }
    manifest->assets_.content_security_policy = *policy;
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
  if (const auto* utilities_value = app->Find("utilities")) {
    const auto* utilities = utilities_value->GetIfDict();
    if (!utilities) {
      return fail("app.utilities must be an object");
    }
    for (const auto [name, definition] : *utilities) {
      if (!IsUtilityName(name) || !definition.is_dict()) {
        return fail(
            "app.utilities requires valid names and object definitions");
      }
      for (const auto [key, value] : definition.GetDict()) {
        if (key != "configuration") {
          return fail("unknown app.utilities option");
        }
        auto json = base::WriteJson(value);
        if (!json || json->size() > 64 * 1024) {
          return fail("utility startup configuration exceeds 64 KiB");
        }
        manifest->utility_configurations_.emplace(
            name, std::vector<uint8_t>(json->begin(), json->end()));
      }
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
  manifest->assets_.scheme = manifest->entry_url_.scheme();
  manifest->app_host_ = manifest->entry_url_.host();
  if (!std::ranges::all_of(
          manifest->windows_, [&](const app::AppWindow& window) {
            return window.url.scheme() == manifest->assets_.scheme;
          })) {
    return fail("all app.windows URLs must use the application scheme");
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
    asset::AssetRoute route;
    if (!rule || !paths || paths->empty() || !rule->FindList("origins") ||
        rule->FindList("origins")->empty() ||
        (file != nullptr) == (directory != nullptr) ||
        !IsSafeRelativePath(file ? *file : *directory) ||
        !read_origins(rule, "origins", "app.assets.origins", false,
                      &route.origins)) {
      return fail("invalid app.assets route");
    }
    for (const auto& pattern : route.origins) {
      if (!pattern.starts_with(manifest->assets_.scheme + "://")) {
        return fail("assets must use the application scheme");
      }
    }
    route.directory = directory != nullptr;
    route.source = file ? *file : *directory;
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
  const auto* rules = security->FindList("capabilities");
  if (!rules) {
    return fail("app.security.capabilities must be an array");
  }
  // The new policy is default-deny, including the entry document.
  manifest->shell_api_origin_patterns_.clear();
  manifest->explorer_origin_patterns_.clear();
  for (const auto& rule_value : *rules) {
    const auto* rule = rule_value.GetIfDict();
    const auto* identifier = rule ? rule->FindString("identifier") : nullptr;
    if (!identifier || !IsCapabilityName(*identifier) ||
        !rule->FindList("origins") || rule->FindList("origins")->empty() ||
        std::ranges::any_of(manifest->origin_capabilities_, [&](const auto& c) {
          return c.identifier == *identifier;
        })) {
      return fail("capability requires unique identifier and origins");
    }
    for (const auto [key, value] : *rule) {
      if (key != "identifier" && key != "description" && key != "origins" &&
          key != "shellApi" && key != "explorer" && key != "devtools") {
        return fail("unknown capability field: " + std::string(key));
      }
    }
    OriginCapability capability;
    capability.identifier = *identifier;
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
    if (const auto* value = rule->Find("explorer")) {
      const auto* explorer = value->GetIfDict();
      if (!explorer) {
        return fail("explorer grant must be an object");
      }
      for (const auto [key, flag] : *explorer) {
        if (!flag.is_bool() ||
            (key != "create" && key != "executeJavaScript")) {
          return fail("invalid explorer grant");
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
    manifest->origin_capabilities_.push_back(std::move(capability));
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
