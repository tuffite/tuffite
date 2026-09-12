// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/video_overlay_window.h"

#include <algorithm>
#include <memory>
#include <utility>
#include <vector>

#include "base/memory/raw_ptr.h"
#include "cc/layers/deadline_policy.h"
#include "components/viz/common/surfaces/surface_id.h"
#include "content/public/browser/immersive_playback_options.h"
#include "content/public/browser/overlay_window.h"
#include "content/public/browser/video_picture_in_picture_window_controller.h"
#include "services/media_session/public/cpp/media_image.h"
#include "services/media_session/public/cpp/media_position.h"
#include "third_party/skia/include/core/SkColor.h"
#include "ui/compositor/compositor.h"
#include "ui/compositor/layer.h"
#include "ui/compositor/layer_surface.h"
#include "ui/display/display.h"
#include "ui/display/screen.h"
#include "ui/gfx/geometry/rect.h"
#include "ui/gfx/geometry/size.h"
#include "ui/views/view.h"
#include "ui/views/widget/widget.h"
#include "ui/views/widget/widget_delegate.h"
#include "ui/views/widget/widget_observer.h"

namespace tuffite {
namespace {

class VideoOverlayWindow final : public content::VideoOverlayWindow,
                                 public views::WidgetDelegate,
                                 public views::WidgetObserver {
 public:
  explicit VideoOverlayWindow(
      content::VideoPictureInPictureWindowController* controller)
      : controller_(controller) {
    auto contents = std::make_unique<views::View>();
    contents->SetUseDefaultFillLayout(true);
    video_view_ = contents->AddChildView(std::make_unique<views::View>());
    video_view_->SetPaintToLayer(ui::LAYER_SURFACE);
    video_view_->layer()->SetFillsBoundsOpaquely(true);
    SetContentsView(std::move(contents));

    widget_ = std::make_unique<views::Widget>();
    views::Widget::InitParams params(
        views::Widget::InitParams::CLIENT_OWNS_WIDGET,
        views::Widget::InitParams::TYPE_WINDOW);
    params.bounds = InitialBounds(gfx::Size(480, 270));
    params.delegate = this;
    params.z_order = ui::ZOrderLevel::kFloatingWindow;
    params.visible_on_all_workspaces = true;
    params.name = "TuffitePictureInPictureWindow";
    widget_->Init(std::move(params));
    widget_->AddObserver(this);
  }

  ~VideoOverlayWindow() override {
    UnregisterSurface();
    if (widget_) {
      widget_->RemoveObserver(this);
      if (!widget_destroyed_) {
        widget_->CloseNow();
      }
    }
  }

  bool IsActive() const override { return widget_ && widget_->IsActive(); }
  void Close() override {
    if (widget_) {
      widget_->Close();
    }
  }
  void ShowInactive() override {
    if (widget_) {
      widget_->ShowInactive();
      widget_->GetContentsView()->InvalidateLayout();
      widget_->GetContentsView()->DeprecatedLayoutImmediately();
      UpdateSurfaceSize();
      widget_->SetVisibleOnAllWorkspaces(true);
    }
  }
  void Hide() override {
    if (widget_) {
      widget_->Hide();
    }
  }
  bool IsVisible() const override { return widget_ && widget_->IsVisible(); }
  gfx::Rect GetBounds() override {
    return widget_ ? widget_->GetWindowBoundsInScreen() : gfx::Rect();
  }

  void UpdateNaturalSize(const gfx::Size& natural_size) override {
    if (!widget_ || natural_size.IsEmpty()) {
      return;
    }
    const double ratio = static_cast<double>(natural_size.width()) /
                         static_cast<double>(natural_size.height());
    const int width = std::clamp(natural_size.width(), 320, 640);
    const int height = std::clamp(static_cast<int>(width / ratio), 180, 480);
    widget_->SetBounds(InitialBounds(gfx::Size(width, height)));
    UpdateSurfaceSize();
  }

  void SetPlaybackState(PlaybackState playback_state) override {}
  void SetPlayPauseButtonVisibility(bool is_visible) override {}
  void SetSkipAdButtonVisibility(bool is_visible) override {}
  void SetNextTrackButtonVisibility(bool is_visible) override {}
  void SetPreviousTrackButtonVisibility(bool is_visible) override {}
  void SetHidePictureInPictureButtonVisibility(bool is_visible) override {}
  void SetMicrophoneMuted(bool muted) override {}
  void SetCameraState(bool turned_on) override {}
  void SetMediaMuted(bool muted) override {}
  void SetToggleMicrophoneButtonVisibility(bool is_visible) override {}
  void SetToggleCameraButtonVisibility(bool is_visible) override {}
  void SetHangUpButtonVisibility(bool is_visible) override {}
  void SetNextSlideButtonVisibility(bool is_visible) override {}
  void SetPreviousSlideButtonVisibility(bool is_visible) override {}
  void SetMediaPosition(const media_session::MediaPosition& position) override {
  }
  void SetSourceTitle(const std::u16string& source_title) override {}
  void SetFaviconImages(
      const std::vector<media_session::MediaImage>& images) override {}

  void SetSurfaceId(const viz::SurfaceId& surface_id) override {
    UnregisterSurface();
    if (!widget_ || !widget_->GetCompositor() || !video_view_) {
      return;
    }
    surface_id_ = surface_id;
    widget_->GetCompositor()->AddChildFrameSink(surface_id.frame_sink_id());
    surface_registered_ = true;
    ui::LayerSurface* surface = video_view_->layer()->AsSurface();
    surface->SetFallbackBackgroundColor(SkColors::kBlack);
    surface->SetShowSurface(surface_id, video_view_->size(),
                            cc::DeadlinePolicy::UseDefaultDeadline(),
                            /*stretch_content_to_fill_bounds=*/true);
  }

  void SetPlaybackControlsVisibility(bool is_visible) override {}
  void SetImmersiveVideoOptions(
      const content::ImmersiveOptions& options) override {}

  void OnWidgetBoundsChanged(views::Widget* widget,
                             const gfx::Rect& new_bounds) override {
    UpdateSurfaceSize();
  }

  void OnWidgetDestroyed(views::Widget* widget) override {
    widget_destroyed_ = true;
    UnregisterSurface();
    if (controller_) {
      auto* controller = controller_.get();
      controller_ = nullptr;
      controller->OnWindowDestroyed(/*should_pause_video=*/false);
    }
  }

 private:
  static gfx::Rect InitialBounds(const gfx::Size& size) {
    display::Screen* screen = display::Screen::Get();
    if (!screen) {
      return gfx::Rect(size);
    }
    gfx::Rect work_area = screen->GetPrimaryDisplay().work_area();
    constexpr int kInset = 24;
    return gfx::Rect(work_area.right() - size.width() - kInset,
                     work_area.bottom() - size.height() - kInset, size.width(),
                     size.height());
  }

  void UpdateSurfaceSize() {
    if (!video_view_) {
      return;
    }
    if (video_view_->layer()->HasExternalContent()) {
      video_view_->layer()->AsSurface()->SetSurfaceSize(video_view_->size());
    }
    if (controller_) {
      controller_->UpdateLayerBounds();
    }
  }

  void UnregisterSurface() {
    if (!surface_registered_ || !widget_ || !widget_->GetCompositor()) {
      return;
    }
    widget_->GetCompositor()->RemoveChildFrameSink(surface_id_.frame_sink_id());
    surface_registered_ = false;
  }

  raw_ptr<content::VideoPictureInPictureWindowController> controller_;
  std::unique_ptr<views::Widget> widget_;
  raw_ptr<views::View> video_view_ = nullptr;
  viz::SurfaceId surface_id_;
  bool surface_registered_ = false;
  bool widget_destroyed_ = false;
};

}  // namespace

std::unique_ptr<content::VideoOverlayWindow> CreateVideoOverlayWindow(
    content::VideoPictureInPictureWindowController* controller) {
  return std::make_unique<VideoOverlayWindow>(controller);
}

}  // namespace tuffite
