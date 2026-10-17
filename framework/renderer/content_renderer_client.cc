// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/renderer/content_renderer_client.h"

#include "base/command_line.h"
#include "content/public/renderer/render_frame.h"
#include "third_party/blink/public/platform/web_string.h"
#include "third_party/blink/public/web/web_security_policy.h"
#include "tuffite/components/dispatch/renderer/dispatch_bindings.h"
#include "tuffite/components/explorer/renderer/explorer_surface_bindings.h"
#include "tuffite/framework/common/switches.h"

namespace tuffite {
ContentRendererClient::ContentRendererClient() = default;

ContentRendererClient::~ContentRendererClient() = default;

void ContentRendererClient::RenderThreadStarted() {
  const std::string scheme =
      base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
          kAppSchemeSwitch);
  if (!scheme.empty()) {
    blink::WebSecurityPolicy::RegisterURLSchemeAsSupportingFetchAPI(
        blink::WebString::FromAscii(scheme));
  }
}

void ContentRendererClient::RenderFrameCreated(content::RenderFrame* frame) {
  if (frame->IsMainFrame()) {
    DispatchBindings::Create(frame);
    ExplorerSurfaceBindings::Create(frame);
  }
}

}  // namespace tuffite
