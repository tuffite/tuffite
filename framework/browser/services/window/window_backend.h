// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_BACKEND_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_BACKEND_H_
#include <memory>

#include "tuffite/framework/browser/services/window/window_types.h"
namespace content {
class BrowserContext;
}
namespace tuffite {
class Window;
}
namespace tuffite::window {
// Native actions are separate from capability checks and window identity.
class WindowBackend {
 public:
  virtual ~WindowBackend() = default;
  virtual Window* Create(content::BrowserContext&, const gfx::Size&) = 0;
  virtual State Read(Window&) const = 0;
  virtual bool Apply(Window&, const Command&) = 0;
  virtual void Close(Window&) = 0;
};
std::unique_ptr<WindowBackend> CreateWindowBackend();
}  // namespace tuffite::window
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_BACKEND_H_
