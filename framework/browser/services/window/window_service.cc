// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/window/window_service.h"
namespace tuffite::window {
void WindowService::ResolveMessageTarget(
    int64_t,
    content::RenderFrameHost&,
    base::OnceCallback<void(content::RenderFrameHost*)> callback) {
  std::move(callback).Run(nullptr);
}
}  // namespace tuffite::window
