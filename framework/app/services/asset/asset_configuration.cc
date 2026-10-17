// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/asset/asset_configuration.h"

#include <algorithm>

#include "base/strings/escape.h"
#include "tuffite/framework/common/access.h"
#include "url/origin.h"
namespace tuffite::asset {
std::string AssetConfiguration::ResolveAssetRoute(const GURL& url) const {
  if (url.path() == "/tuffite-runtime.js" ||
      url.path() == "/tuffite-shell-api.js") {
    return std::string(url.path());
  }
  const auto origin = url::Origin::Create(url);
  const AssetRoute* best = nullptr;
  for (const auto& route : routes) {
    if (!std::ranges::any_of(route.origins, [&](const auto& pattern) {
          return access::MatchesOrigin(origin, pattern);
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

std::optional<base::FilePath> AssetConfiguration::ResolveAssetFile(
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
  return root.Append(relative);
}

bool AssetConfiguration::IsAppUrl(const GURL& url) const {
  return url.is_valid() && url.scheme() == scheme && !url.has_username() &&
         !url.has_password() && !url.has_port() &&
         std::ranges::any_of(routes, [&](const auto& route) {
           return std::ranges::any_of(route.origins, [&](const auto& pattern) {
             return access::MatchesOrigin(url::Origin::Create(url), pattern);
           });
         });
}

}  // namespace tuffite::asset
