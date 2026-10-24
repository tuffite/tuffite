// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/manifest/manifest_service_impl.h"

#include <algorithm>
#include <utility>

#include "base/base_paths.h"
#include "base/path_service.h"
#include "base/strings/string_util.h"
#include "build/build_config.h"
#include "tuffite/framework/app/services/manifest/manifest_configuration_data.h"
#include "tuffite/framework/common/access.h"
namespace tuffite {
namespace {
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
ManifestServiceImpl::ManifestServiceImpl(
    app::EffectiveConfiguration configuration)
    : configuration_(std::move(configuration)) {}
ManifestServiceImpl::~ManifestServiceImpl() = default;
bool ManifestServiceImpl::IsShellApiOriginAllowed(
    const url::Origin& origin) const {
  return std::ranges::any_of(data().shell_api_origin_patterns_,
                             [&](const std::string& pattern) {
                               return access::MatchesOrigin(origin, pattern);
                             });
}

bool ManifestServiceImpl::IsDevToolsAllowed(const url::Origin& origin) const {
  return !std::ranges::any_of(
      data().origin_capabilities_, [&](const auto& capability) {
        return !capability.devtools_allowed &&
               std::ranges::any_of(
                   capability.origins, [&](const auto& pattern) {
                     return access::MatchesOrigin(origin, pattern);
                   });
      });
}

bool ManifestServiceImpl::IsExplorerOriginAllowed(
    const url::Origin& origin) const {
  return std::ranges::any_of(data().explorer_origin_patterns_,
                             [&](const std::string& pattern) {
                               return access::MatchesOrigin(origin, pattern);
                             });
}

bool ManifestServiceImpl::IsExplorerNavigationAllowed(const url::Origin& caller,
                                                      const GURL& url) const {
  if (url.IsAboutBlank()) {
    return true;
  }
  if (!url.is_valid() || url.has_username() || url.has_password()) {
    return false;
  }
  return IsExplorerOriginAllowed(caller) &&
         (url.SchemeIsHTTPOrHTTPS() || url.SchemeIsFile() ||
          url.scheme() == data().assets_.scheme);
}

bool ManifestServiceImpl::IsAllShellApiAllowed(
    const url::Origin& origin) const {
  // Events are currently business-wide. Require every registered business
  // command before exposing that stream.
  bool found = false;
  for (const auto& [method, id] : data().shell_api_command_ids_) {
    if (!method.starts_with("tuffite.")) {
      found = true;
      if (!IsShellApiCommandAllowed(origin, method)) {
        return false;
      }
    }
  }
  return found;
}

bool ManifestServiceImpl::IsShellApiCommandAllowed(
    const url::Origin& origin,
    std::string_view command) const {
  return std::ranges::any_of(
      data().origin_capabilities_, [&](const auto& capability) {
        return std::ranges::any_of(capability.origins,
                                   [&](const auto& pattern) {
                                     return access::MatchesOrigin(origin,
                                                                  pattern);
                                   }) &&
               std::ranges::any_of(
                   capability.shell_api,
                   [&](const auto& grant) { return grant.method == command; });
      });
}

bool ManifestServiceImpl::IsShellApiScopeAllowed(const url::Origin& origin,
                                                 std::string_view method,
                                                 std::string_view resource,
                                                 std::string_view kind,
                                                 bool write) const {
  bool allowed = false;
  for (const auto& capability : data().origin_capabilities_) {
    if (!std::ranges::any_of(capability.origins, [&](const auto& pattern) {
          return access::MatchesOrigin(origin, pattern);
        })) {
      continue;
    }
    for (const auto& grant : capability.shell_api) {
      if (grant.method != method) {
        continue;
      }
      if (!grant.scope_kind.empty() && grant.scope_kind != kind) {
        return false;
      }
      auto matches = [&](const std::string& pattern, bool deny) {
        if (kind == "path") {
          const auto slash = resource.find('/');
          return slash != std::string_view::npos &&
                 access::MatchesPath(resource.substr(0, slash),
                                     resource.substr(slash + 1), pattern,
                                     write);
        }
        if (kind == "url") {
          const GURL url(resource);
          if (!deny &&
              IsLiteralPrivateOrLoopbackHost(base::ToLowerASCII(url.host()))) {
            return access::ExplicitlyAllowsPrivateUrl(url, pattern);
          }
          return access::MatchesUrl(url, pattern);
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
        allowed |= kind.empty() || (kind == "window" && resource == "current");
      }
    }
  }
  return allowed;
}

std::string ManifestServiceImpl::ApplicationInterface(
    std::string_view channel) const {
  if (channel.starts_with("tuffite.") ||
      !std::ranges::any_of(
          data().shell_api_command_ids_,
          [&](const auto& item) { return item.first == channel; })) {
    return {};
  }
  return std::string(channel.substr(0, channel.rfind('.')));
}

bool ManifestServiceImpl::IsApplicationInterface(std::string_view name) const {
  return std::ranges::any_of(data().shell_api_command_ids_,
                             [&](const auto& item) {
                               return ApplicationInterface(item.first) == name;
                             });
}

bool ManifestServiceImpl::IsApplicationCommandRequest(
    std::string_view channel,
    uint64_t command_id) const {
  return std::ranges::any_of(
      data().shell_api_command_ids_, [&](const auto& item) {
        return item.first == channel && item.second == command_id;
      });
}

std::vector<std::string> ManifestServiceImpl::ExecuteJavaScriptOrigins(
    const url::Origin& origin) const {
  return GrantsFor(origin).execute_javascript ? std::vector<std::string>{"*"}
                                              : std::vector<std::string>{};
}

bool ManifestServiceImpl::IsWindowUrlAllowed(const url::Origin& caller,
                                             const GURL& url) const {
  return IsShellApiCommandAllowed(caller, "tuffite.Window.Create") &&
         url.is_valid() && !url.has_username() && !url.has_password() &&
         (url.IsAboutBlank() || url.SchemeIsHTTPOrHTTPS() ||
          url.scheme() == data().assets_.scheme);
}

std::string ManifestServiceImpl::SerializeShellApiOrigins() const {
  std::string serialized;
  for (const std::string& origin : data().shell_api_origin_patterns_) {
    if (!serialized.empty()) {
      serialized.push_back(',');
    }
    serialized.append(origin);
  }
  return serialized;
}

std::string ManifestServiceImpl::SerializeExplorerOrigins() const {
  std::string serialized;
  for (const std::string& origin : data().explorer_origin_patterns_) {
    if (!serialized.empty()) {
      serialized.push_back(',');
    }
    serialized.append(origin);
  }
  return serialized;
}

ManifestServiceImpl::OriginCapability ManifestServiceImpl::GrantsFor(
    const url::Origin& origin) const {
  OriginCapability grants;
  for (const auto& cap : data().origin_capabilities_) {
    if (!std::ranges::any_of(cap.origins, [&](const auto& pattern) {
          return access::MatchesOrigin(origin, pattern);
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

std::map<std::string, std::vector<uint8_t>>
ManifestServiceImpl::UtilityConfigurations() const {
  return data().utility_configurations_;
}

bool ManifestServiceImpl::IsFileSystemPathAllowed(
    const url::Origin& caller,
    std::string_view base,
    std::string_view relative,
    bool write,
    std::string_view method) const {
  return (!write || base != "resource") &&
         IsShellApiScopeAllowed(caller, method,
                                std::string(base) + "/" + std::string(relative),
                                "path", write);
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

std::optional<base::FilePath> ManifestServiceImpl::ResolveBaseDirectory(
    std::string_view base) const {
  if (base != "resource" && base != "data" && base != "cache" &&
      base != "temp") {
    return std::nullopt;
  }
  if (base == "resource") {
    return data().assets_.root;
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
      .AppendASCII(data().application_id_)
      .AppendASCII(std::string(base));
}

std::optional<base::FilePath> ManifestServiceImpl::ResolveFileSystemPath(
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

bool ManifestServiceImpl::IsNetworkUrlAllowed(const url::Origin& caller,
                                              const GURL& url) const {
  return url.is_valid() && url.SchemeIsHTTPOrHTTPS() && !url.has_username() &&
         !url.has_password() &&
         IsShellApiScopeAllowed(caller, "tuffite.Net.Request", url.spec(),
                                "url");
}

app::AppInfo ManifestServiceImpl::app_info() const {
  return {data().application_id_, data().product_name_, data().entry_url_,
          data().assets_.scheme};
}
const std::vector<app::AppWindow>& ManifestServiceImpl::startup_windows()
    const {
  return data().windows_;
}
app::NetworkLimits ManifestServiceImpl::network_limits() const {
  return {data().network_max_response_bytes_, data().network_timeout_seconds_};
}
asset::AssetConfiguration ManifestServiceImpl::assets() const {
  return data().assets_;
}
const std::string& ManifestServiceImpl::SerializeRuntimeConfiguration() const {
  return data().serialized_;
}
base::WeakPtr<const ManifestService> ManifestServiceImpl::GetWeakPtr() const {
  return weak_factory_.GetWeakPtr();
}
void ManifestServiceImpl::Shutdown() {
  weak_factory_.InvalidateWeakPtrs();
}

}  // namespace tuffite
