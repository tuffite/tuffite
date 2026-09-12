// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_DEVTOOLS_DEVTOOLS_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_DEVTOOLS_DEVTOOLS_SERVICE_IMPL_H_
#include <cstdint>

#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "content/public/browser/global_routing_id.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"

namespace tuffite {
class DevToolsServiceImpl final : public DevToolsService {
 public:
  explicit DevToolsServiceImpl(content::BrowserContext& context);
  ~DevToolsServiceImpl() override;
  void SetFrontendLauncher(FrontendLauncher launcher) override;
  void StartIfRequested(content::BrowserContext* context) override;
  void InspectElement(content::WebContents* contents,
                      content::RenderFrameHost* frame,
                      int x,
                      int y) override;
  void Shutdown() override;
  content::BrowserContext* browser_context() const;

 private:
  bool Start(content::BrowserContext* context, uint16_t port);
  void ShowFrontendWhenReady(base::WeakPtr<content::WebContents> inspected,
                             content::GlobalRenderFrameHostId frame_id,
                             int x,
                             int y,
                             int attempts);
  raw_ptr<content::BrowserContext> browser_context_ = nullptr;
  bool stopped_ = false;
  FrontendLauncher frontend_launcher_;
  base::WeakPtrFactory<DevToolsServiceImpl> weak_factory_{this};
};
}  // namespace tuffite
#endif
