// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_CAPABILITY_PATTERN_H_
#define TUFFITE_FRAMEWORK_COMMON_CAPABILITY_PATTERN_H_

#include <string>
#include <string_view>

#include "url/gurl.h"
#include "url/origin.h"

namespace tuffite::capability_pattern {

bool NormalizeOrigin(std::string_view value, std::string* normalized);
bool MatchesOrigin(const url::Origin& origin, std::string_view pattern);

// Accepts legacy host entries, `*`, and absolute HTTP(S) URL globs. URL globs
// may use `*` in a leading host label and in the path.
bool NormalizeUrl(std::string_view value, std::string* normalized);
bool MatchesUrl(const GURL& url, std::string_view pattern);
bool ExplicitlyAllowsPrivateUrl(const GURL& url, std::string_view pattern);

// File-system entries are `base`, `*`, or `base/relative/glob`. Chromium's
// base::MatchPattern supplies `*`; it may cross path separators.
bool NormalizePath(std::string_view value, bool write, std::string* normalized);
bool MatchesPath(std::string_view base,
                 std::string_view relative,
                 std::string_view pattern,
                 bool write);

}  // namespace tuffite::capability_pattern

#endif  // TUFFITE_FRAMEWORK_COMMON_CAPABILITY_PATTERN_H_
