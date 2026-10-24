// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_WINDOW_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_WINDOW_H_

#include "tuffite/framework/shell_api/window.mojom.h"

namespace tuffite::shell_api::window {

class WindowImpl final : public generated::Window {
 public:
  void Current(const dispatch::Context& context,
               generated::Window::CurrentRequestPtr request,
               generated::Window::CurrentCallback callback) override;
  void List(const dispatch::Context& context,
            generated::Window::ListRequestPtr request,
            generated::Window::ListCallback callback) override;
  void Create(const dispatch::Context& context,
              generated::Window::CreateRequestPtr request,
              generated::Window::CreateCallback callback) override;
  void Focus(const dispatch::Context& context,
             generated::Window::FocusRequestPtr request,
             generated::Window::FocusCallback callback) override;
  void Minimize(const dispatch::Context& context,
                generated::Window::MinimizeRequestPtr request,
                generated::Window::MinimizeCallback callback) override;
  void Maximize(const dispatch::Context& context,
                generated::Window::MaximizeRequestPtr request,
                generated::Window::MaximizeCallback callback) override;
  void Restore(const dispatch::Context& context,
               generated::Window::RestoreRequestPtr request,
               generated::Window::RestoreCallback callback) override;
  void SetFullscreen(
      const dispatch::Context& context,
      generated::Window::SetFullscreenRequestPtr request,
      generated::Window::SetFullscreenCallback callback) override;
  void Close(const dispatch::Context& context,
             generated::Window::CloseRequestPtr request,
             generated::Window::CloseCallback callback) override;
  void SetTitle(const dispatch::Context& context,
                generated::Window::SetTitleRequestPtr request,
                generated::Window::SetTitleCallback callback) override;
  void GetBounds(const dispatch::Context& context,
                 generated::Window::GetBoundsRequestPtr request,
                 generated::Window::GetBoundsCallback callback) override;
  void SetBounds(const dispatch::Context& context,
                 generated::Window::SetBoundsRequestPtr request,
                 generated::Window::SetBoundsCallback callback) override;
};

}  // namespace tuffite::shell_api::window

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_WINDOW_H_
