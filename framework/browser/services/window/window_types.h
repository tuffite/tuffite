// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_TYPES_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_TYPES_H_
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

#include "ui/gfx/geometry/rect.h"
#include "ui/gfx/geometry/size.h"
#include "url/gurl.h"
namespace tuffite::window {
struct State {
  std::u16string title;
  gfx::Rect bounds;
  bool focused = false;
  bool minimized = false;
  bool maximized = false;
  bool fullscreen = false;
};

enum class Operation {
  kCurrent,
  kList,
  kCreate,
  kFocus,
  kMinimize,
  kMaximize,
  kRestore,
  kSetFullscreen,
  kClose,
  kSetTitle,
  kGetBounds,
  kSetBounds
};
struct CreateOptions {
  std::optional<std::string> label;
  std::optional<GURL> url;
  std::optional<std::string> title;
  std::optional<gfx::Rect> bounds;
  std::optional<bool> focused;
};
struct Command {
  Operation operation;
  std::optional<int64_t> target;
  CreateOptions create;
  std::string title;
  gfx::Rect bounds;
  bool fullscreen = false;
};
struct Info {
  int64_t id;
  std::string label;
  std::string url;
  State state;
};
}  // namespace tuffite::window
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_TYPES_H_
