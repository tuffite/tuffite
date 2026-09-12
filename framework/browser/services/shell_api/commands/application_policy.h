// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_APPLICATION_POLICY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_APPLICATION_POLICY_H_
#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/browser/browser_context.h"
namespace tuffite::shell_api {
// ShellAPI entry points borrow the policy injected into their caller's context.
inline const AppManifest* ManifestFor(content::RenderFrameHost& frame) {
  return &BrowserContext::GetFrom(frame).manifest();
}
}  // namespace tuffite::shell_api
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_APPLICATION_POLICY_H_
