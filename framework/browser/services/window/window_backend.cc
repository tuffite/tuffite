// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/window/window_backend.h"

#include "base/strings/utf_string_conversions.h"
#include "tuffite/framework/browser/services/window/platform.h"
#include "tuffite/framework/browser/services/window/window.h"
namespace tuffite::window {
namespace {
class NativeWindowBackend final : public WindowBackend {
 public:
  Window* Create(content::BrowserContext& context,
                 const gfx::Size& size) override {
    return Window::Create(&context, GURL(), size);
  }
  State Read(Window& window) const override { return GetState(&window); }
  void Close(Window& window) override { window.Close(); }
  bool Apply(Window& window, const Command& command) override {
    switch (command.operation) {
      case Operation::kFocus:
        return Focus(&window);
      case Operation::kMinimize:
        return Minimize(&window);
      case Operation::kMaximize:
        return Maximize(&window);
      case Operation::kRestore:
        return Restore(&window);
      case Operation::kSetFullscreen:
        return SetFullscreen(&window, command.fullscreen);
      case Operation::kSetTitle:
        return SetTitle(&window, base::UTF8ToUTF16(command.title));
      case Operation::kSetBounds:
        return SetBounds(&window, command.bounds);
      default:
        return false;
    }
  }
};
}  // namespace
std::unique_ptr<WindowBackend> CreateWindowBackend() {
  return std::make_unique<NativeWindowBackend>();
}
}  // namespace tuffite::window
