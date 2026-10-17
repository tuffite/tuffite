// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/common/access.h"

#include <algorithm>
#include <string>

#include "base/files/file_util.h"
#include "base/strings/pattern.h"
#include "base/strings/string_util.h"

namespace tuffite::access {
namespace {

bool IsBase(std::string_view value) {
  return value == "resource" || value == "data" || value == "cache" ||
         value == "temp";
}

bool IsSafePatternText(std::string_view value) {
  return !value.empty() && value.size() <= 2048 &&
         value.find('\0') == std::string_view::npos &&
         value.find(',') == std::string_view::npos &&
         value.find('?') == std::string_view::npos &&
         value.find('\\') == std::string_view::npos &&
         std::ranges::none_of(value, [](unsigned char character) {
           return character <= 0x20 || character == 0x7f;
         });
}

std::string_view HostWithoutPort(std::string_view authority) {
  if (authority.starts_with('[')) {
    const size_t closing = authority.find(']');
    return closing == std::string_view::npos ? std::string_view()
                                             : authority.substr(0, closing + 1);
  }
  const size_t colon = authority.rfind(':');
  return colon == std::string_view::npos ? authority
                                         : authority.substr(0, colon);
}

bool NormalizeAuthority(std::string_view scheme,
                        std::string_view authority,
                        std::string* normalized) {
  if (!normalized || authority.empty() ||
      authority.find('@') != std::string_view::npos) {
    return false;
  }
  const std::string_view host = HostWithoutPort(authority);
  if (host.empty() || host == "*") {
    return false;
  }
  const size_t wildcard = host.find('*');
  if (wildcard != std::string_view::npos &&
      !(wildcard == 0 && host.starts_with("*.") && host.size() > 2 &&
        host.find('*', 1) == std::string_view::npos)) {
    return false;
  }
  const std::string_view suffix = authority.substr(host.size());
  if (!suffix.empty() &&
      (suffix.size() == 1 || suffix.front() != ':' ||
       !std::ranges::all_of(suffix.substr(1), base::IsAsciiDigit<char>))) {
    return false;
  }
  const bool wildcard_host = host.starts_with("*.");
  const std::string candidate = wildcard_host
                                    ? "wildcard." + std::string(host.substr(2))
                                    : std::string(host);
  const GURL parsed(std::string(scheme) + "://" + candidate +
                    std::string(suffix) + "/");
  if (!parsed.is_valid() || url::Origin::Create(parsed).opaque() ||
      parsed.has_username() || parsed.has_password() || parsed.has_query() ||
      parsed.has_ref() || parsed.host().empty()) {
    return false;
  }
  std::string canonical_host = base::ToLowerASCII(parsed.host());
  if (wildcard_host) {
    constexpr std::string_view kPlaceholder = "wildcard.";
    if (!canonical_host.starts_with(kPlaceholder) ||
        canonical_host.size() == kPlaceholder.size()) {
      return false;
    }
    canonical_host = "*." + canonical_host.substr(kPlaceholder.size());
  }
  *normalized = std::move(canonical_host);
  if (parsed.has_port()) {
    normalized->append(":").append(parsed.port());
  }
  return true;
}

bool SplitAbsolutePattern(std::string_view value,
                          std::string_view* scheme,
                          std::string_view* authority,
                          std::string_view* path) {
  const size_t separator = value.find("://");
  if (separator == std::string_view::npos) {
    return false;
  }
  *scheme = value.substr(0, separator);
  const size_t authority_start = separator + 3;
  const size_t path_start = value.find('/', authority_start);
  *authority =
      value.substr(authority_start, path_start == std::string_view::npos
                                        ? std::string_view::npos
                                        : path_start - authority_start);
  *path = path_start == std::string_view::npos ? std::string_view("/")
                                               : value.substr(path_start);
  return true;
}

}  // namespace

bool NormalizeOrigin(std::string_view value, std::string* normalized) {
  if (!normalized || !IsSafePatternText(value)) {
    return false;
  }
  if (value == "*") {
    *normalized = "*";
    return true;
  }
  if (value.find('*') == std::string_view::npos) {
    const GURL url(value);
    const url::Origin origin = url::Origin::Create(url);
    if (!url.is_valid() || origin.opaque() || url != origin.GetURL()) {
      return false;
    }
    *normalized = origin.Serialize();
    return true;
  }
  std::string_view scheme;
  std::string_view authority;
  std::string_view path;
  if (!SplitAbsolutePattern(value, &scheme, &authority, &path) || path != "/" ||
      authority.find('@') != std::string_view::npos) {
    return false;
  }
  std::string normalized_authority;
  if (!NormalizeAuthority(scheme, authority, &normalized_authority)) {
    return false;
  }
  *normalized = std::string(scheme) + "://" + normalized_authority;
  return true;
}

bool MatchesOrigin(const url::Origin& origin, std::string_view pattern) {
  if (origin.opaque()) {
    return false;
  }
  if (pattern == "*") {
    return true;
  }
  std::string_view scheme, authority, path;
  if (!SplitAbsolutePattern(pattern, &scheme, &authority, &path) ||
      scheme != origin.scheme()) {
    return false;
  }
  const std::string_view host = HostWithoutPort(authority);
  // Parse a concrete hostname to canonicalize scheme-specific default ports.
  const std::string concrete = host.starts_with("*.")
                                   ? "wildcard." + std::string(host.substr(2))
                                   : std::string(host);
  const url::Origin expected = url::Origin::Create(
      GURL(std::string(scheme) + "://" + concrete +
           std::string(authority.substr(host.size())) + "/"));
  if (expected.opaque() || expected.port() != origin.port()) {
    return false;
  }
  if (!host.starts_with("*.")) {
    return expected.host() == origin.host();
  }
  const std::string suffix(host.substr(1));
  if (origin.host().size() <= suffix.size() ||
      !origin.host().ends_with(suffix)) {
    return false;
  }
  const std::string prefix =
      origin.host().substr(0, origin.host().size() - suffix.size());
  return !prefix.starts_with('.') && !prefix.ends_with('.') &&
         prefix.find("..") == std::string::npos;
}

bool NormalizeUrl(std::string_view value, std::string* normalized) {
  if (!normalized || !IsSafePatternText(value)) {
    return false;
  }
  if (value == "*") {
    *normalized = "*";
    return true;
  }
  if (value.find("://") == std::string_view::npos) {
    if (value.find('/') != std::string_view::npos ||
        value.find(':') != std::string_view::npos) {
      return false;
    }
    return NormalizeAuthority("https", value, normalized);
  }
  std::string_view scheme;
  std::string_view authority;
  std::string_view path;
  if (!SplitAbsolutePattern(value, &scheme, &authority, &path) ||
      authority.empty() || authority.find('@') != std::string_view::npos ||
      path.find('#') != std::string_view::npos ||
      path.find("..") != std::string_view::npos) {
    return false;
  }
  std::string normalized_authority;
  if (!NormalizeAuthority(scheme, authority, &normalized_authority)) {
    return false;
  }
  *normalized =
      std::string(scheme) + "://" + normalized_authority + std::string(path);
  if (path == "/") {
    normalized->push_back('*');
  }
  return true;
}

bool MatchesUrl(const GURL& url, std::string_view pattern) {
  if (!url.is_valid()) {
    return false;
  }
  if (pattern == "*") {
    // The navigation wildcard follows a normal browser address bar: remote
    // HTTP(S) pages and explicitly entered local files are navigable, while
    // executable/inline schemes such as javascript: and data: stay closed.
    return !url::Origin::Create(url).opaque() || url.SchemeIsFile();
  }
  if (url::Origin::Create(url).opaque()) {
    return false;
  }
  if (pattern.find("://") == std::string_view::npos) {
    return url.SchemeIsHTTPOrHTTPS() &&
           base::MatchPattern(base::ToLowerASCII(url.host()),
                              std::string(pattern));
  }
  std::string_view scheme;
  std::string_view authority;
  std::string_view path;
  if (!SplitAbsolutePattern(pattern, &scheme, &authority, &path)) {
    return false;
  }
  const std::string host_pattern(HostWithoutPort(authority));
  std::string normalized_authority;
  if (!NormalizeAuthority(scheme, authority, &normalized_authority)) {
    return false;
  }
  const std::string_view normalized_host =
      HostWithoutPort(normalized_authority);
  const std::string concrete_host =
      normalized_host.starts_with("*.")
          ? "wildcard." + std::string(normalized_host.substr(2))
          : std::string(normalized_host);
  const GURL normalized_pattern(
      std::string(scheme) + "://" + concrete_host +
      std::string(normalized_authority.substr(normalized_host.size())) + "/");
  return normalized_pattern.is_valid() && url.scheme() == scheme &&
         url.EffectiveIntPort() == normalized_pattern.EffectiveIntPort() &&
         base::MatchPattern(base::ToLowerASCII(url.host()), host_pattern) &&
         base::MatchPattern(url.path(), std::string(path));
}

bool ExplicitlyAllowsPrivateUrl(const GURL& url, std::string_view pattern) {
  if (pattern == "*") {
    return false;
  }
  if (pattern.find("://") == std::string_view::npos) {
    return pattern.find('*') == std::string_view::npos &&
           base::EqualsCaseInsensitiveASCII(pattern, url.host());
  }
  std::string_view scheme;
  std::string_view authority;
  std::string_view path;
  return SplitAbsolutePattern(pattern, &scheme, &authority, &path) &&
         HostWithoutPort(authority).find('*') == std::string_view::npos &&
         MatchesUrl(url, pattern);
}

bool NormalizePath(std::string_view value,
                   bool write,
                   std::string* normalized) {
  if (!normalized || !IsSafePatternText(value) ||
      value.find('\\') != std::string_view::npos ||
      value.find(':') != std::string_view::npos ||
      value.find('?') != std::string_view::npos || value.front() == '/' ||
      value.find("..") != std::string_view::npos) {
    return false;
  }
  if (value == "*") {
    *normalized = "*";
    return true;
  }
  const size_t separator = value.find('/');
  const std::string_view base = value.substr(0, separator);
  if (!IsBase(base) || (write && base == "resource")) {
    return false;
  }
  if (separator != std::string_view::npos && separator + 1 == value.size()) {
    return false;
  }
  *normalized = std::string(value);
  return true;
}

bool MatchesPath(std::string_view base,
                 std::string_view relative,
                 std::string_view pattern,
                 bool write) {
  if (!IsBase(base) || (write && base == "resource")) {
    return false;
  }
  if (pattern == "*") {
    return true;
  }
  const size_t separator = pattern.find('/');
  if (pattern.substr(0, separator) != base) {
    return false;
  }
  if (separator == std::string_view::npos) {
    return true;
  }
  std::string normalized_relative(relative);
  std::ranges::replace(normalized_relative, '\\', '/');
  return base::MatchPattern(normalized_relative,
                            std::string(pattern.substr(separator + 1)));
}

bool HasNoSymlinkComponents(const base::FilePath& root,
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

}  // namespace tuffite::access
