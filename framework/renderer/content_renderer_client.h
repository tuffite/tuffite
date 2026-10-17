// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_RENDERER_CONTENT_RENDERER_CLIENT_H_
#define TUFFITE_FRAMEWORK_RENDERER_CONTENT_RENDERER_CLIENT_H_

#include "content/public/renderer/content_renderer_client.h"

namespace tuffite {

class ContentRendererClient final : public content::ContentRendererClient {
 public:
  ContentRendererClient();
  ~ContentRendererClient() override;

  void RenderThreadStarted() override;
  void RenderFrameCreated(content::RenderFrame* render_frame) override;
  void PrepareErrorPage(content::RenderFrame* render_frame,
                        const blink::WebURLError& error,
                        const std::string& http_method,
                        content::mojom::AlternativeErrorPageOverrideInfoPtr
                            alternative_error_page_info,
                        std::string* error_html) override;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_RENDERER_CONTENT_RENDERER_CLIENT_H_
