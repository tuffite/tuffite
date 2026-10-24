// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_PATH_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_PATH_H_

#include "tuffite/framework/shell_api/path.mojom.h"

namespace tuffite::shell_api::path {

class PathImpl final : public generated::Path {
 public:
  void Directories(const dispatch::Context& context,
                   generated::Path::DirectoriesRequestPtr request,
                   generated::Path::DirectoriesCallback callback) override;
  void Resolve(const dispatch::Context& context,
               generated::Path::ResolveRequestPtr request,
               generated::Path::ResolveCallback callback) override;
};

}  // namespace tuffite::shell_api::path

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_PATH_H_
