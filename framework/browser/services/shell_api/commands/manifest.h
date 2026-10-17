// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_MANIFEST_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_MANIFEST_H_
#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/browser_context.h"
namespace tuffite::shell_api {
// Shell API entry points query the application-scoped configuration service.
inline const ManifestService* ManifestFor(content::RenderFrameHost& frame) {
  return &BrowserContext::GetFrom(frame).manifest_service();
}
}  // namespace tuffite::shell_api
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_MANIFEST_H_
