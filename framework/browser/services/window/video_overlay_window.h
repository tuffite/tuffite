// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_VIDEO_OVERLAY_WINDOW_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_VIDEO_OVERLAY_WINDOW_H_

#include <memory>

namespace content {
class VideoOverlayWindow;
class VideoPictureInPictureWindowController;
}  // namespace content

namespace tuffite {

std::unique_ptr<content::VideoOverlayWindow> CreateVideoOverlayWindow(
    content::VideoPictureInPictureWindowController* controller);

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_VIDEO_OVERLAY_WINDOW_H_
