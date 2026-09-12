// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SERVICE_H_

#include <memory>

#include "components/keyed_service/core/keyed_service.h"

namespace content {
struct ContextMenuParams;
class RenderFrameHost;
class WebContents;
}  // namespace content
namespace tuffite {
// One session per WebContents, owned by its view delegate. Application services
// must outlive the session, as they outlive the owning BrowserContext.
class ContextMenuSession {
 public:
  virtual ~ContextMenuSession() = default;
  virtual void Show(content::RenderFrameHost& frame,
                    const content::ContextMenuParams& params) = 0;
  virtual void Dismiss() = 0;
};
class ContextMenuService : public KeyedService {
 public:
  virtual std::unique_ptr<ContextMenuSession> CreateSession(
      content::WebContents* contents) = 0;
};
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SERVICE_H_
