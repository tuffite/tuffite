// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_MANIFEST_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_MANIFEST_H_
#include <string_view>

#include "content/public/browser/render_frame_host.h"
#include "tuffite/components/dispatch/browser/context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/shell_api/common.mojom.h"
namespace tuffite::shell_api {
inline std::string_view BaseDirectoryName(generated::BaseDirectory value) {
  switch (value) {
    case generated::BaseDirectory::Resource:
      return "resource";
    case generated::BaseDirectory::Data:
      return "data";
    case generated::BaseDirectory::Cache:
      return "cache";
    case generated::BaseDirectory::Temp:
      return "temp";
  }
  return {};
}
// Shell API entry points query the application-scoped configuration service.
inline const ManifestService* ManifestFor(content::RenderFrameHost& frame) {
  return &BrowserContext::GetFrom(frame).manifest_service();
}
// APIs without resource scopes must reject scoped grants, rather than ignore
// them.
inline bool IsUnscopedAllowed(const dispatch::Context& context,
                              std::string_view method) {
  return context.is_application() ||
         ManifestFor(context.frame())
             ->IsShellApiScopeAllowed(context.frame().GetLastCommittedOrigin(),
                                      method, "", "");
}
}  // namespace tuffite::shell_api
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_MANIFEST_H_
