// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_PLATFORM_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_PLATFORM_H_

#include <string>
#include <string_view>

#include "tuffite/framework/browser/services/window/window_types.h"

namespace tuffite {
class Window;
}

namespace tuffite::window {

State GetState(Window* window);
bool Focus(Window* window);
bool Minimize(Window* window);
bool Maximize(Window* window);
bool Restore(Window* window);
bool SetFullscreen(Window* window, bool fullscreen);
bool SetTitle(Window* window, std::u16string_view title);
bool SetBounds(Window* window, const gfx::Rect& bounds);
void ConfigureCustomTitlebar(Window* window);

}  // namespace tuffite::window

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_PLATFORM_H_
