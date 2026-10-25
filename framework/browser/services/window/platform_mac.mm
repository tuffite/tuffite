// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/platform.h"

#import <Cocoa/Cocoa.h>
#import <objc/runtime.h>

#include "base/strings/sys_string_conversions.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "ui/gfx/mac/coordinate_conversion.h"

// AppKit relays out the standard buttons when the window resizes or leaves
// fullscreen. Observe those frames so its native 32-point titlebar cannot
// overwrite the application's 36-point alignment after initial configuration.
// Defer until AppKit finishes positioning all three buttons in the current
// turn.
@interface TuffiteTitlebarLayout : NSObject {
  __weak NSWindow* _window;
  BOOL _updating;
  BOOL _scheduled;
}
- (instancetype)initWithWindow:(NSWindow*)window;
- (void)update:(NSNotification*)notification;
@end

@implementation TuffiteTitlebarLayout
- (instancetype)initWithWindow:(NSWindow*)window {
  if ((self = [super init])) {
    _window = window;
    NSNotificationCenter* center = NSNotificationCenter.defaultCenter;
    NSView* controls =
        [window standardWindowButton:NSWindowCloseButton].superview;
    for (NSView* view in @[
           controls, [window standardWindowButton:NSWindowCloseButton],
           [window standardWindowButton:NSWindowMiniaturizeButton],
           [window standardWindowButton:NSWindowZoomButton]
         ]) {
      view.postsFrameChangedNotifications = YES;
      [center addObserver:self
                 selector:@selector(update:)
                     name:NSViewFrameDidChangeNotification
                   object:view];
    }
    [center addObserver:self
               selector:@selector(update:)
                   name:NSWindowDidExitFullScreenNotification
                 object:window];
  }
  return self;
}

- (void)update:(NSNotification*)notification {
  if (notification) {
    if (_updating || _scheduled) {
      return;
    }
    _scheduled = YES;
    __weak TuffiteTitlebarLayout* weak_self = self;
    dispatch_async(dispatch_get_main_queue(), ^{
      TuffiteTitlebarLayout* strong_self = weak_self;
      if (strong_self) {
        strong_self->_scheduled = NO;
        [strong_self update:nil];
      }
    });
    return;
  }
  NSWindow* window = _window;
  if (_updating || !window ||
      (window.styleMask & NSWindowStyleMaskFullScreen)) {
    return;
  }
  _updating = YES;
  NSView* controls =
      [window standardWindowButton:NSWindowCloseButton].superview;
  NSRect frame = controls.frame;
  frame.origin.y += frame.size.height - 36.0;
  frame.size.height = 36.0;
  if (!NSEqualRects(frame, controls.frame)) {
    controls.frame = frame;
  }
  for (NSNumber* type in @[
         @(NSWindowCloseButton), @(NSWindowMiniaturizeButton),
         @(NSWindowZoomButton)
       ]) {
    NSButton* button =
        [window standardWindowButton:static_cast<NSWindowButton>(
                                         type.unsignedIntegerValue)];
    NSRect button_frame = button.frame;
    button_frame.origin.y = (36.0 - button_frame.size.height) / 2.0;
    if (!NSEqualRects(button_frame, button.frame)) {
      button.frame = button_frame;
    }
  }
  _updating = NO;
}

- (void)dealloc {
  [NSNotificationCenter.defaultCenter removeObserver:self];
}
@end

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
  static char layout_key;
  TuffiteTitlebarLayout* layout = objc_getAssociatedObject(native, &layout_key);
  if (!layout) {
    layout = [[TuffiteTitlebarLayout alloc] initWithWindow:native];
    objc_setAssociatedObject(native, &layout_key, layout,
                             OBJC_ASSOCIATION_RETAIN_NONATOMIC);
  }
  [layout update:nil];
  for (NSButton* button in @[ close, minimize, zoom ]) {
    button.hidden = NO;
    button.alphaValue = 1.0;
    button.enabled = YES;
  }
}

}  // namespace tuffite::window
