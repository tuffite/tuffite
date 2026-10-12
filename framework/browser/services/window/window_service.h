// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_SERVICE_H_
#include <cstdint>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "base/memory/weak_ptr.h"
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/framework/browser/services/window/window_types.h"
namespace tuffite {
class Window;
}
namespace tuffite::window {
class WindowService : public KeyedService {
 public:
  virtual Window* Create(const gfx::Size& size = {},
                         bool decorations = true,
                         const Constraints& constraints = {}) = 0;
  // Primary navigation starts on the next task so callers can install
  // observers. Secondary windows open after its first successful navigation.
  virtual Window* CreateStartupWindows(std::vector<WindowOptions> windows,
                                       const gfx::Size& primary_size = {}) = 0;
  virtual Window* FindById(int64_t) = 0;
  virtual std::optional<Info> GetInfo(Window*) = 0;
  virtual std::vector<Info> List() = 0;
  virtual bool Focus(Window*) = 0;
  virtual bool Minimize(Window*) = 0;
  virtual bool Maximize(Window*) = 0;
  virtual bool Restore(Window*) = 0;
  virtual bool SetFullscreen(Window*, bool) = 0;
  virtual bool SetTitle(Window*, std::u16string_view) = 0;
  virtual bool SetBounds(Window*, const gfx::Rect&) = 0;
  virtual bool Close(Window*) = 0;
  virtual void Adopt(std::unique_ptr<Window>) = 0;
  virtual void DestroyWhenIdle(Window*) = 0;
  virtual void Register(Window*, std::string) = 0;
  virtual void Unregister(Window*) = 0;
  virtual void CloseById(int64_t) = 0;
  virtual base::WeakPtr<WindowService> GetWeakPtr() = 0;
};
}  // namespace tuffite::window

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_SERVICE_H_
