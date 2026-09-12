// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_WINDOW_API_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_WINDOW_API_H_

#include "tuffite/framework/shell_api/window.h"

namespace tuffite::shell_api::window {

void Current(const ::tuffite::dispatch::Context& context,
             generated::window::CurrentRequest&& request,
             generated::window::CurrentCallback&& callback);
void List(const ::tuffite::dispatch::Context& context,
          generated::window::ListRequest&& request,
          generated::window::ListCallback&& callback);
void Create(const ::tuffite::dispatch::Context& context,
            generated::window::CreateRequest&& request,
            generated::window::CreateCallback&& callback);
void Focus(const ::tuffite::dispatch::Context& context,
           generated::window::FocusRequest&& request,
           generated::window::FocusCallback&& callback);
void Minimize(const ::tuffite::dispatch::Context& context,
              generated::window::MinimizeRequest&& request,
              generated::window::MinimizeCallback&& callback);
void Maximize(const ::tuffite::dispatch::Context& context,
              generated::window::MaximizeRequest&& request,
              generated::window::MaximizeCallback&& callback);
void Restore(const ::tuffite::dispatch::Context& context,
             generated::window::RestoreRequest&& request,
             generated::window::RestoreCallback&& callback);
void SetFullscreen(const ::tuffite::dispatch::Context& context,
                   generated::window::SetFullscreenRequest&& request,
                   generated::window::SetFullscreenCallback&& callback);
void Close(const ::tuffite::dispatch::Context& context,
           generated::window::CloseRequest&& request,
           generated::window::CloseCallback&& callback);
void SetTitle(const ::tuffite::dispatch::Context& context,
              generated::window::SetTitleRequest&& request,
              generated::window::SetTitleCallback&& callback);
void GetBounds(const ::tuffite::dispatch::Context& context,
               generated::window::GetBoundsRequest&& request,
               generated::window::GetBoundsCallback&& callback);
void SetBounds(const ::tuffite::dispatch::Context& context,
               generated::window::SetBoundsRequest&& request,
               generated::window::SetBoundsCallback&& callback);

}  // namespace tuffite::shell_api::window

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_WINDOW_API_H_
