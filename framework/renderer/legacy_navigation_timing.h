// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_RENDERER_LEGACY_NAVIGATION_TIMING_H_
#define TUFFITE_FRAMEWORK_RENDERER_LEGACY_NAVIGATION_TIMING_H_

#include "content/public/renderer/render_frame_observer.h"

namespace tuffite {

// Installs Chromium's legacy navigation timing APIs for each document context.
class LegacyNavigationTiming final : public content::RenderFrameObserver {
 public:
  static void Create(content::RenderFrame* frame);

 private:
  explicit LegacyNavigationTiming(content::RenderFrame* frame);
  ~LegacyNavigationTiming() override;
  void DidCreateScriptContext(v8::Local<v8::Context> context,
                              int32_t world_id) override;
  void OnDestruct() override;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_RENDERER_LEGACY_NAVIGATION_TIMING_H_
