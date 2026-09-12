// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/main_delegate.h"

#import <AppKit/AppKit.h>

#include "base/check.h"
#include "base/mac/scoped_sending_event.h"
#include "base/message_loop/message_pump_apple.h"
#include "base/observer_list.h"
#include "content/public/browser/native_event_processor_mac.h"
#include "content/public/browser/native_event_processor_observer_mac.h"

@interface TuffiteApplication
    : NSApplication <CrAppProtocol, CrAppControlProtocol, NativeEventProcessor>
@end

@implementation TuffiteApplication {
  base::ObserverList<content::NativeEventProcessorObserver>::Unchecked
      _observers;
  BOOL _handlingSendEvent;
}

- (BOOL)isHandlingSendEvent {
  return _handlingSendEvent;
}

- (void)setHandlingSendEvent:(BOOL)handlingSendEvent {
  _handlingSendEvent = handlingSendEvent;
}

- (void)sendEvent:(NSEvent*)event {
  base::mac::ScopedSendingEvent sendingEventScoper;
  content::ScopedNotifyNativeEventProcessorObserver observerNotifier(
      &_observers, event);
  // A Tuffite window hosts renderer-backed text controls rather than native
  // NSTextViews. Route standard Command-key equivalents through the AppKit
  // edit menu so the current WebContents responder receives cut/copy/paste,
  // select-all, and undo/redo just like a regular macOS application.
  if (event.type == NSEventTypeKeyDown &&
      (event.modifierFlags & NSEventModifierFlagCommand) != 0 &&
      [NSApp.mainMenu performKeyEquivalent:event]) {
    return;
  }
  [super sendEvent:event];
}

- (void)addNativeEventProcessorObserver:
    (content::NativeEventProcessorObserver*)observer {
  _observers.AddObserver(observer);
}

- (void)removeNativeEventProcessorObserver:
    (content::NativeEventProcessorObserver*)observer {
  _observers.RemoveObserver(observer);
}

@end

namespace tuffite {
namespace {

NSMenuItem* CreateMenuItem(NSString* title,
                           SEL action,
                           NSString* key_equivalent) {
  return [[NSMenuItem alloc] initWithTitle:title
                                    action:action
                             keyEquivalent:key_equivalent];
}

NSMenu* BuildMainMenu() {
  NSMenu* main_menu = [[NSMenu alloc] initWithTitle:@""];

  NSMenu* app_menu = [[NSMenu alloc] initWithTitle:@""];
  NSString* application_name =
      [[NSRunningApplication currentApplication] localizedName];
  if (application_name.length == 0) {
    application_name = [[NSProcessInfo processInfo] processName];
  }
  [app_menu addItem:CreateMenuItem([NSString stringWithFormat:@"Hide %@",
                                                              application_name],
                                   @selector(hide:), @"h")];
  [app_menu addItem:CreateMenuItem([NSString stringWithFormat:@"Quit %@",
                                                              application_name],
                                   @selector(terminate:), @"q")];

  NSMenu* edit_menu = [[NSMenu alloc] initWithTitle:@"Edit"];
  [edit_menu addItem:CreateMenuItem(@"Undo", @selector(undo:), @"z")];
  [edit_menu addItem:CreateMenuItem(@"Redo", @selector(redo:), @"Z")];
  [edit_menu addItem:[NSMenuItem separatorItem]];
  [edit_menu addItem:CreateMenuItem(@"Cut", @selector(cut:), @"x")];
  [edit_menu addItem:CreateMenuItem(@"Copy", @selector(copy:), @"c")];
  [edit_menu addItem:CreateMenuItem(@"Paste", @selector(paste:), @"v")];
  [edit_menu
      addItem:CreateMenuItem(@"Select All", @selector(selectAll:), @"a")];

  NSMenu* window_menu = [[NSMenu alloc] initWithTitle:@"Window"];
  [window_menu addItem:CreateMenuItem(@"Minimize",
                                      @selector(performMiniaturize:), @"m")];
  [window_menu addItem:CreateMenuItem(@"Zoom", @selector(performZoom:), @"")];

  for (NSMenu* menu in @[ app_menu, edit_menu, window_menu ]) {
    NSMenuItem* item = [[NSMenuItem alloc] initWithTitle:@""
                                                  action:nil
                                           keyEquivalent:@""];
    item.submenu = menu;
    [main_menu addItem:item];
  }
  return main_menu;
}

}  // namespace

void MainDelegate::RegisterNSApplication() {
  CHECK_EQ(NSApp, nil);
  [TuffiteApplication sharedApplication];
  CHECK([NSApp isKindOfClass:[TuffiteApplication class]]);
  [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
  [NSApp setMainMenu:BuildMainMenu()];
}

}  // namespace tuffite
