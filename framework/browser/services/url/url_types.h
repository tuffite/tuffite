// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_TYPES_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_TYPES_H_

#include <optional>
#include <string>

#include "base/files/file_path.h"
#include "base/functional/callback.h"
#include "url/gurl.h"

namespace network {
struct ResourceRequest;
}

namespace tuffite {

struct URLResolution {
  std::string kind;
  base::FilePath file;
  int status = 200;
  std::string mime_type;
  std::string contents;
};
using URLResolver = base::RepeatingCallback<void(
    const network::ResourceRequest&,
    base::OnceCallback<void(std::optional<URLResolution>)>)>;

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_URL_URL_TYPES_H_
