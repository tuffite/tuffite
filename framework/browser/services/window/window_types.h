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
namespace tuffite {
enum class WebContentsRole;
}
namespace tuffite::window {
struct Constraints {
  gfx::Size minimum_size{1, 1};
  gfx::Size maximum_size{16384, 16384};
  bool Contains(const gfx::Rect& bounds) const {
    return bounds.width() >= minimum_size.width() &&
           bounds.height() >= minimum_size.height() &&
           bounds.width() <= maximum_size.width() &&
           bounds.height() <= maximum_size.height();
  }
};
struct WindowOptions {
  std::string label;
  GURL url;
  std::u16string title;
  gfx::Size size;
  std::optional<gfx::Point> position;
  bool focused = true;
  bool decorations = true;
  Constraints constraints;
  // No role means an unprivileged surface. Only trusted browser callers set it.
  std::optional<WebContentsRole> role;
};
struct State {
  std::u16string title;
  gfx::Rect bounds;
  bool focused = false;
  bool minimized = false;
  bool maximized = false;
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
