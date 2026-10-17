// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_ACCESS_H_
#define TUFFITE_FRAMEWORK_COMMON_ACCESS_H_

#include <string>
#include <string_view>

#include "base/files/file_path.h"
#include "url/gurl.h"
#include "url/origin.h"

namespace tuffite::access {

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

// Filesystem query: invoke on a sequence that permits blocking I/O.
// Checks root and its existing, lexically root-relative path components for
// symlinks.
bool HasNoSymlinkComponents(const base::FilePath& root,
                            const base::FilePath& path);

}  // namespace tuffite::access

#endif  // TUFFITE_FRAMEWORK_COMMON_ACCESS_H_
