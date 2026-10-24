// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <algorithm>

#include "net/base/url_util.h"
#include "tuffite/framework/app/services/manifest/manifest_configuration.h"
#include "tuffite/framework/app/services/manifest/manifest_configuration_data.h"
#include "tuffite/framework/common/access.h"
#include "url/origin.h"

namespace tuffite::app {
EffectiveConfiguration::EffectiveConfiguration(
    std::unique_ptr<ManifestData> data)
    : data_(std::move(data)) {}
EffectiveConfiguration::EffectiveConfiguration(
    EffectiveConfiguration&&) noexcept = default;
EffectiveConfiguration& EffectiveConfiguration::operator=(
    EffectiveConfiguration&&) noexcept = default;
EffectiveConfiguration::~EffectiveConfiguration() = default;

namespace {
bool ApplyDevelopmentUrl(ManifestData& data,
                         const GURL& url,
                         std::string* error) {
  if (!url.is_valid() || !url.SchemeIs("http") || !net::IsLocalhost(url) ||
      url.has_username() || url.has_password() || url.has_ref()) {
    if (error) {
      *error = "--tuffite-dev-url must be a credential-free loopback HTTP URL";
    }
    return false;
  }
  if (data.windows_.empty()) {
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
  data.windows_.front().url = url;
  const auto packaged_origin = url::Origin::Create(data.entry_url_);
  for (auto& capability : data.origin_capabilities_) {
    if (std::ranges::any_of(capability.origins, [&](const auto& pattern) {
          return access::MatchesOrigin(packaged_origin, pattern);
        })) {
      capability.origins.push_back(origin);
      if (!capability.shell_api.empty()) {
        grant_origin(&data.shell_api_origin_patterns_);
      }
      if (capability.create_explorer) {
        grant_origin(&data.explorer_origin_patterns_);
      }
    }
  }
  return true;
}

}  // namespace

std::optional<EffectiveConfiguration> ConfigurationAssembler::Build(
    ManifestDefinition definition,
    const StartupEnvironment& environment,
    const BrowserOptions& options,
    std::string* error) {
  if (!definition.data_) {
    if (error) {
      *error = "manifest definition has already been consumed";
    }
    return std::nullopt;
  }
  auto data = std::move(definition.data_);
  OriginCapability denial;
  denial.devtools_allowed = false;
  for (const auto& origin : options.security.devtools.denied_origins) {
    std::string normalized;
    if (!access::NormalizeOrigin(origin, &normalized)) {
      if (error) {
        *error =
            "Browser::setup DevTools origins must be canonical origin patterns";
      }
      return std::nullopt;
    }
    if (!std::ranges::contains(denial.origins, normalized)) {
      denial.origins.push_back(std::move(normalized));
    }
  }
  if (!denial.origins.empty()) {
    data->origin_capabilities_.push_back(std::move(denial));
  }
  // Apply the alias after all policy sources have been merged: denials inherit
  // too.
  if (environment.development_url &&
      !ApplyDevelopmentUrl(*data, *environment.development_url, error)) {
    return std::nullopt;
  }
  return EffectiveConfiguration(std::move(data));
}
}  // namespace tuffite::app
