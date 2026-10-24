// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/renderer/content_renderer_client.h"

#include "base/command_line.h"
#include "base/strings/escape.h"
#include "content/public/renderer/render_frame.h"
#include "net/base/net_errors.h"
#include "third_party/blink/public/platform/web_string.h"
#include "third_party/blink/public/platform/web_url_error.h"
#include "third_party/blink/public/web/web_security_policy.h"
#include "tuffite/components/dispatch/renderer/dispatch_bindings.h"
#include "tuffite/components/explorer/renderer/explorer_surface_bindings.h"
#include "tuffite/framework/common/switches.h"
#include "tuffite/framework/renderer/legacy_navigation_timing.h"

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
  LegacyNavigationTiming::Create(frame);
  if (frame->IsMainFrame()) {
    DispatchBindings::Create(frame);
  }
  ExplorerSurfaceBindings::Create(frame);
}

void ContentRendererClient::PrepareErrorPage(
    content::RenderFrame* render_frame,
    const blink::WebURLError& error,
    const std::string& http_method,
    content::mojom::AlternativeErrorPageOverrideInfoPtr
        alternative_error_page_info,
    std::string* error_html) {
  // Preserve Chromium's error-document origin and navigation bookkeeping.
  // Display only the error code; a failed URL can contain credentials/tokens.
  if (!error_html || !error_html->empty()) {
    return;
  }
  *error_html =
      "<!doctype html><meta charset=utf-8><title>Page could not be "
      "loaded</title>"
      "<meta name=viewport content='width=device-width, initial-scale=1'>"
      "<style>:root{color-scheme:light dark}body{font:16px "
      "system-ui;max-width:40rem;"
      "margin:12vh auto;padding:2rem}code{font-size:0.9em}</style>"
      "<h1>Page could not be loaded</h1><p>Check the connection or address, "
      "then try reloading the page.</p><code>" +
      base::EscapeForHTML(net::ErrorToShortString(error.reason())) + "</code>";
}

}  // namespace tuffite
