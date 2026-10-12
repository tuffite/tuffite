// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_DEVTOOLS_DEVTOOLS_FRONTEND_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_DEVTOOLS_DEVTOOLS_FRONTEND_H_

namespace content {
class RenderFrameHost;
class WebContents;
}  // namespace content

namespace tuffite {
namespace window {
class WindowService;
}

// Focuses the existing frontend for |inspected|, or opens and connects one.
// The controller owns itself for the lifetime of the frontend WebContents.
void ShowDevToolsFrontend(window::WindowService& windows,
                          content::WebContents* inspected,
                          content::RenderFrameHost* inspected_frame,
                          int x,
                          int y);

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_DEVTOOLS_DEVTOOLS_FRONTEND_H_
