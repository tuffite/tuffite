// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/common/app_manifest.h"

#include <algorithm>
#include <utility>

#include "base/base_paths.h"
#include "base/check.h"
#include "base/files/file_util.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/path_service.h"
#include "base/strings/escape.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_util.h"
#include "base/values.h"
#include "build/build_config.h"
#include "net/base/url_util.h"
#include "tuffite/framework/common/capability_pattern.h"
#include "tuffite/framework/common/constants.h"
#include "tuffite/framework/common/scope_registry.inc"

namespace tuffite {
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

// Rejects hosts that literally spell out addresses in RFC1918 private space,
// loopback, link-local, or IPv6 loopback / unique-local / link-local. This
// is a coarse-grained SSRF block for the common case where an attacker
// injects a URL like `http://127.0.0.1/` or `http://169.254.169.254/latest/`
// (cloud metadata) into an allowlisted host wildcard. Runtime DNS lookups
// that resolve to the same ranges still need to be blocked at the network
// service layer; see docs/networking.md.
bool IsLiteralPrivateOrLoopbackHost(std::string_view host_lower) {
  if (host_lower.empty()) {
    return true;
  }
  if (host_lower == "localhost" || host_lower.ends_with(".localhost") ||
      host_lower.ends_with(".localdomain") || host_lower == "ip6-localhost" ||
      host_lower == "ip6-loopback") {
    return true;
  }
  // Strip IPv6 brackets so we can pattern-match string literals.
  std::string_view raw = host_lower;
  if (raw.size() >= 2 && raw.front() == '[' && raw.back() == ']') {
    raw.remove_prefix(1);
    raw.remove_suffix(1);
  }
  if (raw == "::1" || raw == "0:0:0:0:0:0:0:1" || raw.starts_with("fc") ||
      raw.starts_with("fd") || raw.starts_with("fe80:") ||
      raw.starts_with("fe80::") || raw == "::" || raw == "0:0:0:0:0:0:0:0") {
    return true;
  }
  // IPv4 literal fast paths: `10.`, `127.`, `169.254.`, `192.168.`,
  // and 172.16.0.0/12.
  if (raw.starts_with("10.") || raw.starts_with("127.") ||
      raw.starts_with("169.254.") || raw.starts_with("192.168.") ||
      raw == "0.0.0.0") {
    return true;
  }
  if (raw.starts_with("172.")) {
    // Parse the second octet and enforce 16..=31.
    const size_t first_dot = raw.find('.');
    if (first_dot == std::string_view::npos || first_dot + 1 >= raw.size()) {
      return false;
    }
    const size_t second_dot = raw.find('.', first_dot + 1);
    const std::string_view second =
        raw.substr(first_dot + 1, (second_dot == std::string_view::npos)
                                      ? std::string_view::npos
                                      : second_dot - (first_dot + 1));
    int value = 0;
    for (char ch : second) {
      if (!base::IsAsciiDigit(ch)) {
        return false;
      }
      value = value * 10 + (ch - '0');
      if (value > 255) {
        return false;
      }
    }
    return value >= 16 && value <= 31;
  }
  return false;
}

}  // namespace

AppManifest::AppManifest() = default;
AppManifest::AppManifest(const AppManifest&) = default;
AppManifest& AppManifest::operator=(const AppManifest&) = default;
AppManifest::AppManifest(AppManifest&&) = default;
AppManifest& AppManifest::operator=(AppManifest&&) = default;
AppManifest::~AppManifest() = default;

bool AppManifest::ApplyDevelopmentUrl(const GURL& url, std::string* error) {
  if (!url.is_valid() || !url.SchemeIs("http") || !net::IsLocalhost(url) ||
      url.has_username() || url.has_password() || url.has_ref()) {
    if (error) {
      *error = "--tuffite-dev-url must be a credential-free loopback HTTP URL";
    }
    return false;
  }
  if (windows_.empty()) {
    if (error) {
      *error = "application manifest has no startup window";
    }
    return false;
  }
  const std::string origin = url::Origin::Create(url).Serialize();
  auto grant_origin = [&](std::vector<std::string>* patterns) {
    if (std::ranges::find(*patterns, origin) == patterns->end()) {
      patterns->push_back(origin);
    }
  };
  windows_.front().url = url;
  const auto packaged_origin = url::Origin::Create(entry_url_);
  for (auto& capability : origin_capabilities_) {
    if (std::ranges::any_of(capability.origins, [&](const auto& pattern) {
          return capability_pattern::MatchesOrigin(packaged_origin, pattern);
        })) {
      capability.origins.push_back(origin);
      if (!capability.shell_api.empty()) {
        grant_origin(&shell_api_origin_patterns_);
      }
      if (capability.create_explorer) {
        grant_origin(&explorer_origin_patterns_);
      }
    }
  }
  return true;
}

// static
std::optional<std::string> AppManifest::ReadApplicationScheme(
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
bool AppManifest::ReadNetworkDisableHttp2(const base::FilePath& package_root) {
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
std::optional<AppManifest> AppManifest::Load(const base::FilePath& package_root,
                                             std::string* error) {
  auto fail = [error](std::string message) -> std::optional<AppManifest> {
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
  const std::string* shell_api_namespace =
      shell_api ? shell_api->FindString("namespace") : nullptr;
  const base::ListValue* windows = app ? app->FindList("windows") : nullptr;
  if (!application_id || !IsValidApplicationId(*application_id) ||
      !shell_api_namespace || !IsIdentifier(*shell_api_namespace) ||
      *shell_api_namespace == "tuffite" || !windows || windows->empty() ||
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
  AppManifest manifest;
  manifest.package_root_ = package_root;
  manifest.asset_root_ = package_root.AppendASCII("assets");
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
    manifest.content_security_policy_ = *policy;
  }
  manifest.application_id_ = *application_id;
  if (root.contains("productName") && !root.FindString("productName")) {
    return fail("productName must be a string");
  }
  manifest.product_name_ = root.FindString("productName")
                               ? *root.FindString("productName")
                               : *application_id;
  if (manifest.product_name_.empty() || manifest.product_name_.size() > 128 ||
      !base::IsStringUTF8(manifest.product_name_) ||
      manifest.product_name_.find_first_of("\r\n") != std::string::npos ||
      manifest.product_name_.find('\0') != std::string::npos) {
    return fail("productName is invalid");
  }
  manifest.shell_api_namespace_ = *shell_api_namespace;
  if (const auto* utilities_value = app->Find("utilities")) {
    const auto* utilities = utilities_value->GetIfDict();
    if (!utilities) {
      return fail("app.utilities must be an object");
    }
    for (const auto [name, definition] : *utilities) {
      if (!IsIdentifier(name) || !definition.is_dict()) {
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
      }
    }
  }
  manifest.serialized_ = std::move(serialized);
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
    AppWindow configured;
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
    manifest.windows_.push_back(std::move(configured));
  }
  manifest.entry_url_ = manifest.windows_.front().url;
  manifest.app_scheme_ = manifest.entry_url_.scheme();
  if (manifest.app_scheme_ != manifest.shell_api_namespace_) {
    return fail("application URL scheme must match shellApi.namespace");
  }
  manifest.app_host_ = manifest.entry_url_.host();
  if (!std::ranges::all_of(manifest.windows_, [&](const AppWindow& window) {
        return window.url.scheme() == manifest.app_scheme_;
      })) {
    return fail("all app.windows URLs must use the application scheme");
  }
  const url::Origin entry_origin = url::Origin::Create(manifest.entry_url_);
  manifest.shell_api_origin_patterns_.push_back(entry_origin.Serialize());

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
          !(url_patterns ? capability_pattern::NormalizeUrl(*serialized_origin,
                                                            &normalized)
                         : capability_pattern::NormalizeOrigin(
                               *serialized_origin, &normalized))) {
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
    AssetRoute route;
    if (!rule || !paths || paths->empty() || !rule->FindList("origins") ||
        rule->FindList("origins")->empty() ||
        (file != nullptr) == (directory != nullptr) ||
        !IsSafeRelativePath(file ? *file : *directory) ||
        !read_origins(rule, "origins", "app.assets.origins", false,
                      &route.origins)) {
      return fail("invalid app.assets route");
    }
    for (const auto& pattern : route.origins) {
      if (!pattern.starts_with(manifest.app_scheme_ + "://")) {
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
      for (const auto& existing : manifest.asset_routes_) {
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
            if (capability_pattern::MatchesOrigin(sample(left), right) ||
                capability_pattern::MatchesOrigin(sample(right), left)) {
              return fail("ambiguous app.assets route");
            }
          }
        }
      }
      manifest.asset_routes_.push_back(route);
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
      manifest.shell_api_command_ids_.emplace_back(name, id);
    }
  }
  const auto* rules = security->FindList("capabilities");
  if (!rules) {
    return fail("app.security.capabilities must be an array");
  }
  // The new policy is default-deny, including the entry document.
  manifest.shell_api_origin_patterns_.clear();
  manifest.explorer_origin_patterns_.clear();
  const auto scope_registry =
      base::JSONReader::Read(kShellApiScopeRegistry, base::JSON_PARSE_RFC);
  CHECK(scope_registry && scope_registry->is_dict());
  for (const auto& rule_value : *rules) {
    const auto* rule = rule_value.GetIfDict();
    const auto* identifier = rule ? rule->FindString("identifier") : nullptr;
    if (!identifier || !IsCapabilityName(*identifier) ||
        !rule->FindList("origins") || rule->FindList("origins")->empty() ||
        std::ranges::any_of(manifest.origin_capabilities_, [&](const auto& c) {
          return c.identifier == *identifier;
        })) {
      return fail("capability requires unique identifier and origins");
    }
    for (const auto [key, value] : *rule) {
      if (key != "identifier" && key != "description" && key != "origins" &&
          key != "shellApi" && key != "explorer") {
        return fail("unknown capability field: " + std::string(key));
      }
    }
    OriginCapability capability;
    capability.identifier = *identifier;
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
        bool matched_scope = false;
        for (const auto& [method, id] : manifest.shell_api_command_ids_) {
          if (wildcard ? !method.starts_with(prefix) : method != *pattern) {
            continue;
          }
          matched = true;
          ShellApiGrant grant;
          grant.method = method;
          const auto* descriptor = scope_registry->GetDict().FindDict(method);
          if (descriptor) {
            matched_scope = true;
            grant.scope_kind = *descriptor->FindString("kind");
          }
          for (const auto* key : {"allow", "deny"}) {
            const auto* scopes = object ? object->Find(key) : nullptr;
            if (!scopes) {
              continue;
            }
            // Namespace groups apply resource scopes only to methods that
            // consume resources; unscoped methods retain a plain method grant.
            if (!descriptor && wildcard && !all_methods) {
              continue;
            }
            if (!descriptor || !scopes->is_list()) {
              return fail(method + " does not support this scope");
            }
            grant.has_allow |= std::string_view(key) == "allow";
            for (const auto& scope : scopes->GetList()) {
              const auto* fields = scope.GetIfDict();
              const auto* text =
                  fields ? fields->FindString(grant.scope_kind == "window"
                                                  ? "window"
                                                  : grant.scope_kind)
                         : nullptr;
              std::string normalized;
              if (!text || fields->size() != 1) {
                return fail(method + ": invalid scope object");
              }
              if (grant.scope_kind == "path") {
                if (!capability_pattern::NormalizePath(
                        *text, descriptor->FindBool("write").value_or(false),
                        &normalized)) {
                  return fail(method + ": invalid path scope");
                }
              } else if (grant.scope_kind == "url") {
                if (*text != "*" && !text->starts_with("https://") &&
                    !text->starts_with("http://")) {
                  return fail(method + ": expected absolute HTTP(S) URL scope");
                }
                if (!capability_pattern::NormalizeUrl(*text, &normalized)) {
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
        if (object && (object->Find("allow") || object->Find("deny")) &&
            !matched_scope) {
          return fail(*pattern + " does not support scopes");
        }
        if (!matched) {
          return fail("method pattern matches no registered method: " +
                      *pattern);
        }
      }
    }
    for (const auto& origin : capability.origins) {
      if (capability.create_explorer) {
        manifest.explorer_origin_patterns_.push_back(origin);
      }
      if (!capability.shell_api.empty()) {
        manifest.shell_api_origin_patterns_.push_back(origin);
      }
    }
    manifest.origin_capabilities_.push_back(std::move(capability));
  }
  const auto* network = app->FindDict("network");
  if (network) {
    const auto* disable = network->Find("disableHttp2");
    if (disable && !disable->is_bool()) {
      return fail("network.disableHttp2 must be a boolean");
    }
    manifest.network_disable_http2_ =
        network->FindBool("disableHttp2").value_or(false);
    if (auto bytes = network->FindInt("maxResponseBytes")) {
      if (*bytes <= 0 ||
          static_cast<size_t>(*bytes) > kMaxNetworkResponseBytes) {
        return fail("invalid network.maxResponseBytes");
      }
      manifest.network_max_response_bytes_ = *bytes;
    }
    if (auto seconds = network->FindInt("timeoutSeconds")) {
      if (*seconds <= 0 || *seconds > kMaxNetworkTimeoutSeconds) {
        return fail("invalid network.timeoutSeconds");
      }
      manifest.network_timeout_seconds_ = *seconds;
    }
  }
  return manifest;
}

bool AppManifest::IsShellApiOriginAllowed(const url::Origin& origin) const {
  return std::ranges::any_of(
      shell_api_origin_patterns_, [&](const std::string& pattern) {
        return capability_pattern::MatchesOrigin(origin, pattern);
      });
}

bool AppManifest::IsExplorerOriginAllowed(const url::Origin& origin) const {
  return std::ranges::any_of(
      explorer_origin_patterns_, [&](const std::string& pattern) {
        return capability_pattern::MatchesOrigin(origin, pattern);
      });
}

bool AppManifest::IsExplorerNavigationAllowed(const url::Origin& caller,
                                              const GURL& url) const {
  if (url.IsAboutBlank()) {
    return true;
  }
  if (!url.is_valid() || url.has_username() || url.has_password()) {
    return false;
  }
  return IsExplorerOriginAllowed(caller) &&
         (url.SchemeIsHTTPOrHTTPS() || url.SchemeIsFile() ||
          url.scheme() == app_scheme_);
}

bool AppManifest::IsAllShellApiAllowed(const url::Origin& origin) const {
  // Events are currently business-wide. Require every registered business
  // command before exposing that stream.
  bool found = false;
  for (const auto& [method, id] : shell_api_command_ids_) {
    if (method.starts_with(shell_api_namespace_ + ".")) {
      found = true;
      if (!IsShellApiCommandAllowed(origin, method)) {
        return false;
      }
    }
  }
  return found;
}

bool AppManifest::IsShellApiCommandAllowed(const url::Origin& origin,
                                           std::string_view command) const {
  return std::ranges::any_of(origin_capabilities_, [&](const auto& capability) {
    return std::ranges::any_of(capability.origins,
                               [&](const auto& pattern) {
                                 return capability_pattern::MatchesOrigin(
                                     origin, pattern);
                               }) &&
           std::ranges::any_of(capability.shell_api, [&](const auto& grant) {
             return grant.method == command;
           });
  });
}

bool AppManifest::IsShellApiScopeAllowed(const url::Origin& origin,
                                         std::string_view method,
                                         std::string_view resource) const {
  bool allowed = false;
  for (const auto& capability : origin_capabilities_) {
    if (!std::ranges::any_of(capability.origins, [&](const auto& pattern) {
          return capability_pattern::MatchesOrigin(origin, pattern);
        })) {
      continue;
    }
    for (const auto& grant : capability.shell_api) {
      if (grant.method != method) {
        continue;
      }
      auto matches = [&](const std::string& pattern, bool deny) {
        if (grant.scope_kind == "path") {
          const auto slash = resource.find('/');
          return slash != std::string_view::npos &&
                 capability_pattern::MatchesPath(
                     resource.substr(0, slash), resource.substr(slash + 1),
                     pattern,
                     method != "tuffite.fs.readFile" &&
                         method != "tuffite.fs.exists" &&
                         method != "tuffite.path.resolve");
        }
        if (grant.scope_kind == "url") {
          const GURL url(resource);
          if (!deny &&
              IsLiteralPrivateOrLoopbackHost(base::ToLowerASCII(url.host()))) {
            return capability_pattern::ExplicitlyAllowsPrivateUrl(url, pattern);
          }
          return capability_pattern::MatchesUrl(url, pattern);
        }
        return pattern == "*" || pattern == resource;
      };
      // Denials apply across all matching capabilities, even with an unscoped
      // grant.
      if (std::ranges::any_of(grant.deny, [&](const auto& pattern) {
            return matches(pattern, true);
          })) {
        return false;
      }
      allowed |= std::ranges::any_of(grant.allow, [&](const auto& pattern) {
        return matches(pattern, false);
      });
      if (!grant.has_allow) {
        allowed |= grant.scope_kind.empty() ||
                   (grant.scope_kind == "window" && resource == "current");
      }
    }
  }
  return allowed;
}

bool AppManifest::IsApplicationCommandRequest(std::string_view channel,
                                              uint64_t command_id) const {
  return std::ranges::any_of(shell_api_command_ids_, [&](const auto& item) {
    return item.first == channel && item.second == command_id;
  });
}

std::vector<std::string> AppManifest::ExecuteJavaScriptOrigins(
    const url::Origin& origin) const {
  return GrantsFor(origin).execute_javascript ? std::vector<std::string>{"*"}
                                              : std::vector<std::string>{};
}

bool AppManifest::IsWindowUrlAllowed(const url::Origin& caller,
                                     const GURL& url) const {
  return IsShellApiCommandAllowed(caller, "tuffite.window.create") &&
         url.is_valid() && !url.has_username() && !url.has_password() &&
         (url.IsAboutBlank() || url.SchemeIsHTTPOrHTTPS() ||
          url.scheme() == app_scheme_);
}

std::string AppManifest::SerializeShellApiOrigins() const {
  std::string serialized;
  for (const std::string& origin : shell_api_origin_patterns_) {
    if (!serialized.empty()) {
      serialized.push_back(',');
    }
    serialized.append(origin);
  }
  return serialized;
}

std::string AppManifest::SerializeExplorerOrigins() const {
  std::string serialized;
  for (const std::string& origin : explorer_origin_patterns_) {
    if (!serialized.empty()) {
      serialized.push_back(',');
    }
    serialized.append(origin);
  }
  return serialized;
}

OriginCapability AppManifest::GrantsFor(const url::Origin& origin) const {
  OriginCapability grants;
  for (const auto& cap : origin_capabilities_) {
    if (!std::ranges::any_of(cap.origins, [&](const auto& pattern) {
          return capability_pattern::MatchesOrigin(origin, pattern);
        })) {
      continue;
    }
    grants.create_explorer |= cap.create_explorer;
    grants.execute_javascript |= cap.execute_javascript;
    grants.shell_api.insert(grants.shell_api.end(), cap.shell_api.begin(),
                            cap.shell_api.end());
  }
  return grants;
}

std::map<std::string, std::vector<uint8_t>> AppManifest::UtilityConfigurations()
    const {
  std::map<std::string, std::vector<uint8_t>> configurations;
  auto document = base::JSONReader::ReadDict(serialized_, base::JSON_PARSE_RFC);
  const auto* utilities =
      document ? document->FindDictByDottedPath("app.utilities") : nullptr;
  if (utilities) {
    for (const auto [name, definition] : *utilities) {
      if (!definition.is_dict()) {
        continue;
      }
      const auto* value = definition.GetDict().Find("configuration");
      if (!value) {
        continue;
      }
      auto json = base::WriteJson(*value);
      if (json) {
        configurations.emplace(
            name, std::vector<uint8_t>(json->begin(), json->end()));
      }
    }
  }
  return configurations;
}

std::string AppManifest::ResolveAssetRoute(const GURL& url) const {
  if (url.path() == "/tuffite-runtime.js" ||
      url.path() == "/tuffite-shell-api.js") {
    return std::string(url.path());
  }
  const auto origin = url::Origin::Create(url);
  const AssetRoute* best = nullptr;
  for (const auto& route : asset_routes_) {
    if (!std::ranges::any_of(route.origins, [&](const auto& pattern) {
          return capability_pattern::MatchesOrigin(origin, pattern);
        })) {
      continue;
    }
    const bool glob = route.path.ends_with('*');
    const auto prefix =
        glob ? route.path.substr(0, route.path.size() - 1) : route.path;
    if (glob ? !url.path().starts_with(prefix) : url.path() != prefix) {
      continue;
    }
    if (!best || (!glob && best->path.ends_with('*')) ||
        (glob == best->path.ends_with('*') &&
         route.path.size() > best->path.size())) {
      best = &route;
    }
  }
  if (!best) {
    return {};
  }
  if (!best->directory) {
    return "/" + best->source;
  }
  return "/" + best->source + "/" +
         std::string(url.path().substr(best->path.size() - 1));
}

std::optional<base::FilePath> AppManifest::ResolveAssetFile(
    const GURL& url) const {
  if (!IsAppUrl(url)) {
    return std::nullopt;
  }
  const std::string escaped = ResolveAssetRoute(url);
  std::string decoded;
  if (escaped.empty() || escaped.front() != '/' ||
      !base::UnescapeBinaryURLComponentSafe(escaped.substr(1), true,
                                            &decoded) ||
      decoded.empty()) {
    return std::nullopt;
  }
  const auto relative = base::FilePath::FromUTF8Unsafe(decoded);
  if (relative.IsAbsolute() || relative.ReferencesParent()) {
    return std::nullopt;
  }
  return asset_root().Append(relative);
}

bool AppManifest::IsAppUrl(const GURL& url) const {
  return url.is_valid() && url.scheme() == app_scheme_ && !url.has_username() &&
         !url.has_password() && !url.has_port() &&
         std::ranges::any_of(asset_routes_, [&](const auto& route) {
           return std::ranges::any_of(route.origins, [&](const auto& pattern) {
             return capability_pattern::MatchesOrigin(url::Origin::Create(url),
                                                      pattern);
           });
         });
}

bool AppManifest::IsFileSystemPathAllowed(const url::Origin& caller,
                                          std::string_view base,
                                          std::string_view relative,
                                          bool write,
                                          std::string_view method) const {
  return (!write || base != "resource") &&
         IsShellApiScopeAllowed(
             caller, method, std::string(base) + "/" + std::string(relative));
}

namespace {

bool IsSafeFileSystemRelativePath(std::string_view value) {
  if (value.empty() || value.find('\0') != std::string_view::npos ||
      value.find(':') != std::string_view::npos) {
    return false;
  }
  const base::FilePath path = base::FilePath::FromUTF8Unsafe(value);
  return !path.IsAbsolute() && !path.ReferencesParent() &&
         (value == "." || path != base::FilePath(FILE_PATH_LITERAL(".")));
}

}  // namespace

std::optional<base::FilePath> AppManifest::ResolveBaseDirectory(
    std::string_view base) const {
  if (base != "resource" && base != "data" && base != "cache" &&
      base != "temp") {
    return std::nullopt;
  }
  if (base == "resource") {
    return asset_root_;
  }
  base::FilePath root;
  if (base == "temp") {
    if (!base::PathService::Get(base::DIR_TEMP, &root)) {
      return std::nullopt;
    }
  } else {
#if BUILDFLAG(IS_WIN)
    if (!base::PathService::Get(base::DIR_LOCAL_APP_DATA, &root)) {
      return std::nullopt;
    }
#elif BUILDFLAG(IS_MAC)
    if (base == "cache") {
      if (!base::PathService::Get(base::DIR_CACHE, &root)) {
        return std::nullopt;
      }
    } else {
      if (!base::PathService::Get(base::DIR_APP_DATA, &root)) {
        return std::nullopt;
      }
    }
#else
    if (base == "cache") {
      if (!base::PathService::Get(base::DIR_CACHE, &root)) {
        return std::nullopt;
      }
    } else {
      if (!base::PathService::Get(base::DIR_HOME, &root)) {
        return std::nullopt;
      }
    }
    if (base == "data") {
      root = root.AppendASCII(".local").AppendASCII("share");
    }
#endif
  }
  return root.AppendASCII("Tuffite")
      .AppendASCII(application_id_)
      .AppendASCII(std::string(base));
}

std::optional<base::FilePath> AppManifest::ResolveFileSystemPath(
    const url::Origin& caller,
    std::string_view base,
    std::string_view relative,
    bool write,
    std::string_view method) const {
  if (!IsFileSystemPathAllowed(caller, base, relative, write, method) ||
      !IsSafeFileSystemRelativePath(relative)) {
    return std::nullopt;
  }
  std::optional<base::FilePath> root = ResolveBaseDirectory(base);
  if (!root) {
    return std::nullopt;
  }
  return root->Append(base::FilePath::FromUTF8Unsafe(relative));
}

bool AppManifest::HasNoSymlinkComponents(const base::FilePath& root,
                                         const base::FilePath& path) {
  if (base::PathExists(root) && base::IsLink(root)) {
    return false;
  }
  base::FilePath current = root;
  base::FilePath relative;
  if (!root.AppendRelativePath(path, &relative)) {
    return false;
  }
  for (const auto& component : relative.GetComponents()) {
    current = current.Append(component);
    if (base::PathExists(current) && base::IsLink(current)) {
      return false;
    }
  }
  return true;
}

bool AppManifest::IsNetworkUrlAllowed(const url::Origin& caller,
                                      const GURL& url) const {
  return url.is_valid() && url.SchemeIsHTTPOrHTTPS() && !url.has_username() &&
         !url.has_password() &&
         IsShellApiScopeAllowed(caller, "tuffite.net.request", url.spec());
}

}  // namespace tuffite
