// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_COMMON_EXTERNAL_PROTOCOL_H_
#define TUFFITE_COMPONENTS_EXPLORER_COMMON_EXTERNAL_PROTOCOL_H_

#include <algorithm>
#include <span>
#include <string>

#include "url/gurl.h"

namespace tuffite {

// External URLs are candidates for a browser-owned confirmation, never an
// instruction to execute script, load a file, or enter an internal browser UI.
inline bool IsExternalProtocol(
    const GURL& url,
    std::span<const std::string> application_schemes) {
  constexpr const char* kInternal[] = {"http",
                                       "https",
                                       "ws",
                                       "wss",
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
  return !std::ranges::contains(application_schemes, url.scheme()) &&
         url.is_valid() && !url.has_username() && !url.has_password() &&
         url.spec().size() <= 8192 &&
         std::ranges::find(kInternal, url.scheme()) == std::end(kInternal);
}

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_COMMON_EXTERNAL_PROTOCOL_H_
