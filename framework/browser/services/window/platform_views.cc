// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "build/build_config.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/browser/services/window/platform.h"
#include "tuffite/framework/browser/services/window/window.h"
#if BUILDFLAG(IS_WIN)
#include <dwmapi.h>

#include "ui/aura/window.h"
#include "ui/aura/window_tree_host.h"
#endif
#include "ui/views/widget/widget.h"
#include "ui/views/widget/widget_delegate.h"

namespace tuffite::window {
namespace {

views::Widget* GetWidget(Window* window) {
  return window ? window->widget() : nullptr;
}

}  // namespace

State GetState(Window* window) {
  State state;
  if (views::Widget* widget = GetWidget(window)) {
    if (widget->widget_delegate()) {
      state.title = widget->widget_delegate()->GetWindowTitle();
    }
    state.bounds = widget->GetWindowBoundsInScreen();
    state.focused = widget->IsActive();
    state.minimized = widget->IsMinimized();
    state.maximized = widget->IsMaximized();
    state.fullscreen = widget->IsFullscreen();
  }
  return state;
}

bool Focus(Window* window) {
  views::Widget* widget = GetWidget(window);
  if (!widget) {
    return false;
  }
  widget->Show();
  widget->Activate();
  window->web_contents()->Focus();
  return true;
}

bool Minimize(Window* window) {
  views::Widget* widget = GetWidget(window);
  if (!widget) {
    return false;
  }
  widget->Minimize();
  return true;
}

bool Maximize(Window* window) {
  views::Widget* widget = GetWidget(window);
  if (!widget) {
    return false;
  }
  widget->Maximize();
  return true;
}

bool Restore(Window* window) {
  views::Widget* widget = GetWidget(window);
  if (!widget) {
    return false;
  }
  widget->Restore();
  return true;
}

bool SetFullscreen(Window* window, bool fullscreen) {
  views::Widget* widget = GetWidget(window);
  if (!widget) {
    return false;
  }
  widget->SetFullscreen(fullscreen);
  return true;
}

bool SetTitle(Window* window, std::u16string_view title) {
  views::Widget* widget = GetWidget(window);
  if (!widget || !widget->widget_delegate()) {
    return false;
  }
  widget->widget_delegate()->SetTitle(std::u16string(title));
  widget->UpdateWindowTitle();
  return true;
}

bool SetBounds(Window* window, const gfx::Rect& bounds) {
  views::Widget* widget = GetWidget(window);
  if (!widget) {
    return false;
  }
  widget->SetBoundsConstrained(bounds);
  return true;
}

void ConfigureCustomTitlebar(Window* window) {
#if BUILDFLAG(IS_WIN)
  views::Widget* widget = GetWidget(window);
  if (!widget || !widget->GetNativeWindow() ||
      !widget->GetNativeWindow()->GetHost()) {
    return;
  }
  HWND hwnd = widget->GetNativeWindow()->GetHost()->GetAcceleratedWidget();
  if (!hwnd) {
    return;
  }
  LONG_PTR style = ::GetWindowLongPtrW(hwnd, GWL_STYLE);
  style |= WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
  style &= ~WS_CAPTION;
  ::SetWindowLongPtrW(hwnd, GWL_STYLE, style);
  ::SetWindowPos(hwnd, nullptr, 0, 0, 0, 0,
                 SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER |
                     SWP_NOACTIVATE);
  const DWM_WINDOW_CORNER_PREFERENCE preference = DWMWCP_ROUND;
  ::DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &preference,
                          sizeof(preference));
#endif
}

}  // namespace tuffite::window
