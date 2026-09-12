// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/platform.h"

#import <Cocoa/Cocoa.h>

#include "base/strings/sys_string_conversions.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "ui/gfx/mac/coordinate_conversion.h"

namespace tuffite::window {
namespace {

NSWindow* GetWindow(Window* window) {
  return window ? window->widget()->GetNativeWindow().GetNativeNSWindow() : nil;
}

}  // namespace

State GetState(Window* window) {
  State state;
  NSWindow* native = GetWindow(window);
  if (native) {
    state.title = base::SysNSStringToUTF16(native.title);
    state.bounds = gfx::ScreenRectFromNSRect(native.frame);
    state.focused = native.keyWindow;
    state.minimized = native.miniaturized;
    state.maximized = native.zoomed;
    state.fullscreen = (native.styleMask & NSWindowStyleMaskFullScreen) != 0;
  }
  return state;
}

bool Focus(Window* window) {
  NSWindow* native = GetWindow(window);
  if (!native) {
    return false;
  }
  [native makeKeyAndOrderFront:nil];
  window->web_contents()->Focus();
  return true;
}

bool Minimize(Window* window) {
  NSWindow* native = GetWindow(window);
  if (!native) {
    return false;
  }
  [native miniaturize:nil];
  return true;
}

bool Maximize(Window* window) {
  NSWindow* native = GetWindow(window);
  if (!native) {
    return false;
  }
  if (!native.zoomed) {
    [native zoom:nil];
  }
  return true;
}

bool Restore(Window* window) {
  NSWindow* native = GetWindow(window);
  if (!native) {
    return false;
  }
  if (native.miniaturized) {
    [native deminiaturize:nil];
  } else if (native.zoomed) {
    [native zoom:nil];
  }
  return true;
}

bool SetFullscreen(Window* window, bool fullscreen) {
  NSWindow* native = GetWindow(window);
  if (!native) {
    return false;
  }
  const bool current = (native.styleMask & NSWindowStyleMaskFullScreen) != 0;
  if (current != fullscreen) {
    [native toggleFullScreen:nil];
  }
  return true;
}

bool SetTitle(Window* window, std::u16string_view title) {
  NSWindow* native = GetWindow(window);
  if (!native) {
    return false;
  }
  native.title = base::SysUTF16ToNSString(title);
  return true;
}

bool SetBounds(Window* window, const gfx::Rect& bounds) {
  NSWindow* native = GetWindow(window);
  if (!native) {
    return false;
  }
  [native setFrame:gfx::ScreenRectToNSRect(bounds) display:YES animate:NO];
  return true;
}

void ConfigureCustomTitlebar(Window* window) {
  NSWindow* native = GetWindow(window);
  if (!native) {
    return;
  }
  native.styleMask |= NSWindowStyleMaskTitled | NSWindowStyleMaskClosable |
                      NSWindowStyleMaskMiniaturizable |
                      NSWindowStyleMaskResizable |
                      NSWindowStyleMaskFullSizeContentView;
  native.titlebarAppearsTransparent = YES;
  native.titleVisibility = NSWindowTitleHidden;
  NSButton* close = [native standardWindowButton:NSWindowCloseButton];
  NSButton* minimize = [native standardWindowButton:NSWindowMiniaturizeButton];
  NSButton* zoom = [native standardWindowButton:NSWindowZoomButton];
  close.superview.hidden = NO;
  for (NSButton* button in @[ close, minimize, zoom ]) {
    button.hidden = NO;
    button.alphaValue = 1.0;
    button.enabled = YES;
  }
}

}  // namespace tuffite::window
