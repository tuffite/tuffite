// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_DEVTOOLS_DEVTOOLS_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_DEVTOOLS_DEVTOOLS_SERVICE_H_
#include <memory>

#include "base/functional/callback.h"
#include "components/keyed_service/core/keyed_service.h"
#include "content/public/browser/devtools_manager_delegate.h"
namespace content {
class BrowserContext;
class RenderFrameHost;
class WebContents;
}  // namespace content
namespace tuffite {
// Process-level adapter resolves the live remote-debugging owner dynamically.
std::unique_ptr<content::DevToolsManagerDelegate>
CreateDevToolsManagerDelegate();

class DevToolsService : public KeyedService {
 public:
  using FrontendLauncher = base::RepeatingCallback<
      void(content::WebContents*, content::RenderFrameHost*, int, int)>;
  virtual void SetFrontendLauncher(FrontendLauncher launcher) = 0;
  virtual void StartIfRequested(content::BrowserContext* context) = 0;
  virtual void InspectElement(content::WebContents* contents,
                              content::RenderFrameHost* frame,
                              int x,
                              int y) = 0;
};
}  // namespace tuffite
#endif
