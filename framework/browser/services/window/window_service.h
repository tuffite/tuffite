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
namespace content {
class WebContents;
}
namespace tuffite {
class Window;
}
namespace tuffite::window {
enum class Status {
  kOk,
  kInvalidArgument,
  kAccessDenied,
  kUnavailable,
};

struct Result {
  Status status = Status::kUnavailable;
  std::vector<Info> windows;
  std::optional<gfx::Rect> bounds;
  std::optional<int64_t> close_after_reply;
};

class WindowService : public KeyedService {
 public:
  virtual Window* CreateStartupWindows(const gfx::Size& primary_size = {}) = 0;
  virtual Result Invoke(const Command&, content::WebContents*) = 0;
  virtual void Adopt(std::unique_ptr<Window>) = 0;
  virtual void DestroyWhenIdle(Window*) = 0;
  virtual void Register(Window*, std::string) = 0;
  virtual void Unregister(Window*) = 0;
  virtual void CloseById(int64_t) = 0;
  virtual base::WeakPtr<WindowService> GetWeakPtr() = 0;
};
}  // namespace tuffite::window

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_SERVICE_H_
