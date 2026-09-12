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
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_RENDERER_CONTENT_RENDERER_CLIENT_H_
