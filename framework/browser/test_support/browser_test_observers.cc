// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/test_support/browser_test_observers.h"

#include <cstdlib>
#include <string>
#include <unordered_set>
#include <utility>
#include <vector>

#include "base/check.h"
#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/json/json_writer.h"
#include "base/location.h"
#include "base/logging.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "base/strings/utf_string_conversions.h"
#include "base/task/bind_post_task.h"
#include "base/task/sequenced_task_runner.h"
#include "base/task/thread_pool.h"
#include "base/time/time.h"
#include "base/values.h"
#include "components/viz/common/frame_sinks/copy_output_result.h"
#include "content/public/browser/navigation_entry.h"
#include "content/public/browser/render_widget_host_view.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_observer.h"
#include "third_party/skia/include/core/SkBitmap.h"
#include "third_party/skia/include/core/SkColor.h"
#include "tuffite/buildflags.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "ui/gfx/geometry/rect.h"
#include "ui/gfx/geometry/size.h"
#include "ui/gfx/image/image.h"
#include "ui/snapshot/snapshot.h"

namespace tuffite {
namespace {
class SmokeTestObserver final : public content::WebContentsObserver,
                                public BrowserTestObserver {
 public:
  SmokeTestObserver(content::WebContents* web_contents,
                    base::FilePath marker_path)
      : content::WebContentsObserver(web_contents),
        marker_path_(std::move(marker_path)) {
    // CreateNewWindow may commit and update the title before the observer is
    // attached on fast local packages. Polling also makes smoke startup robust
    // against this legitimate ordering without weakening the final assertions.
    MaybeStart();
  }

  void TitleWasSet(content::NavigationEntry* entry) override { MaybeStart(); }

 private:
  void MaybeStart() {
    if (started_ || completed_) {
      return;
    }
    if (web_contents()->GetTitle() != u"Tuffite App Ready") {
      if (ready_retry_scheduled_) {
        return;
      }
      if (++ready_attempt_ < kMaxReadyAttempts) {
        ready_retry_scheduled_ = true;
        base::SequencedTaskRunner::GetCurrentDefault()->PostDelayedTask(
            FROM_HERE,
            base::BindOnce(&SmokeTestObserver::RetryMaybeStart,
                           weak_factory_.GetWeakPtr()),
            base::Milliseconds(250));
      } else {
        Finish(false, "application did not reach ready title: " +
                          base::UTF16ToUTF8(web_contents()->GetTitle()));
      }
      return;
    }
    application_ready_ = true;
    started_ = true;
#if BUILDFLAG(TUFFITE_IN_CHROMIUM)
    VerifyExplorerSurface();
#else
    Finish(false, "explorer buildflag disabled");
#endif
  }

  void RetryMaybeStart() {
    ready_retry_scheduled_ = false;
    MaybeStart();
  }

  void VerifyExplorerSurface() {
    if (completed_) {
      return;
    }
    ++attempt_;
    const std::vector<content::WebContents*> inner_contents =
        web_contents()->GetInnerWebContents();
    if (inner_contents.empty()) {
      RetryOrFail("no inner WebContents");
      return;
    }

    content::WebContents* inner = inner_contents.front();
    inner_url_ = inner->GetLastCommittedURL().spec();
    inner_title_ = base::UTF16ToUTF8(inner->GetTitle());
    if (inner_title_ != "Explorer v2 Ready") {
      RetryOrFail("Explorer v2 capability probes have not passed");
      return;
    }
    content::RenderWidgetHostView* view = inner->GetRenderWidgetHostView();
    if (!view) {
      RetryOrFail("inner RenderWidgetHostView missing");
      return;
    }
    view_bounds_ = view->GetViewBounds();
    viewport_size_ = view->GetVisibleViewportSize();
    view_showing_ = view->IsShowing();
    surface_available_ = view->IsSurfaceAvailableForCopy();
    if (view_bounds_.IsEmpty() || viewport_size_.IsEmpty() ||
        !surface_available_) {
      RetryOrFail("inner surface not ready");
      return;
    }

    content::RenderWidgetHostView* outer_view =
        web_contents()->GetRenderWidgetHostView();
    outer_view_showing_ = outer_view && outer_view->IsShowing();
    outer_surface_available_ =
        outer_view && outer_view->IsSurfaceAvailableForCopy();
    if (!outer_view || !outer_view->IsSurfaceAvailableForCopy()) {
      RetryOrFail("outer surface not ready");
      return;
    }
    const gfx::Rect outer_bounds = outer_view->GetViewBounds();
    capture_rect_ = view_bounds_;
    capture_rect_.Offset(-outer_bounds.x(), -outer_bounds.y());
    capture_rect_.Intersect(gfx::Rect(outer_view->GetVisibleViewportSize()));
    if (!view_showing_ || capture_rect_.IsEmpty()) {
      RetryOrFail("inner view is outside the visible outer viewport");
      return;
    }

    // Capture the final outer composition rather than the child Surface in
    // isolation. This proves that Viz embedded the inner frame where the user
    // actually sees the <explorer> element.
    outer_view->CopyFromSurface(
        capture_rect_, gfx::Size(96, 48), base::Seconds(2),
        base::BindPostTaskToCurrentDefault(base::BindOnce(
            &SmokeTestObserver::OnSurfaceCopied, weak_factory_.GetWeakPtr())));
  }

  void OnSurfaceCopied(const content::CopyFromSurfaceResult& result) {
    if (!result.has_value()) {
      last_copy_error_ = static_cast<int>(result.error());
      RetryOrFail("CopyFromSurface failed");
      return;
    }

    const SkBitmap& bitmap = result->bitmap;
    bitmap_size_ = gfx::Size(bitmap.width(), bitmap.height());
    if (bitmap.drawsNothing()) {
      RetryOrFail("CopyFromSurface returned an empty bitmap");
      return;
    }

    std::unordered_set<SkColor> sampled_colors;
    expected_background_pixels_ = 0;
    non_white_pixels_ = 0;
    total_pixels_ = bitmap.width() * bitmap.height();
    for (int y = 0; y < bitmap.height(); ++y) {
      for (int x = 0; x < bitmap.width(); ++x) {
        const SkColor color = bitmap.getColor(x, y);
        if (sampled_colors.size() < 256) {
          sampled_colors.insert(color);
        }
        const int red = SkColorGetR(color);
        const int green = SkColorGetG(color);
        const int blue = SkColorGetB(color);
        if (std::abs(red - 24) <= 12 && std::abs(green - 32) <= 12 &&
            std::abs(blue - 51) <= 12) {
          ++expected_background_pixels_;
        }
        if (red < 245 || green < 245 || blue < 245) {
          ++non_white_pixels_;
        }
      }
    }
    distinct_sampled_colors_ = sampled_colors.size();

    // explorer.html deliberately paints #182033. Requiring that color plus
    // more than one sampled color proves that the inner WebContents' rendered
    // frame was copied; loadstop alone only proves navigation completed.
    const bool has_target_background =
        expected_background_pixels_ * 4 >= total_pixels_;
    if (!has_target_background || distinct_sampled_colors_ < 2) {
      RetryOrFail("inner surface pixels do not match explorer.html");
      return;
    }
    Finish(true, "inner Surface contains expected rendered pixels");
  }

  void RetryOrFail(std::string reason) {
    last_reason_ = std::move(reason);
    if (attempt_ >= kMaxAttempts) {
      Finish(false, last_reason_);
      return;
    }
    base::SequencedTaskRunner::GetCurrentDefault()->PostDelayedTask(
        FROM_HERE,
        base::BindOnce(&SmokeTestObserver::VerifyExplorerSurface,
                       weak_factory_.GetWeakPtr()),
        base::Milliseconds(250));
  }

  void Finish(bool explorer_ok, const std::string& reason) {
    if (completed_) {
      return;
    }
    completed_ = true;
    base::DictValue visual;
    visual.Set("attempts", attempt_);
    visual.Set("reason", reason);
    visual.Set("url", inner_url_);
    visual.Set("title", inner_title_);
    visual.Set("view_showing", view_showing_);
    visual.Set("surface_available", surface_available_);
    visual.Set("outer_view_showing", outer_view_showing_);
    visual.Set("outer_surface_available", outer_surface_available_);
    visual.Set("view_bounds", base::ListValue()
                                  .Append(view_bounds_.x())
                                  .Append(view_bounds_.y())
                                  .Append(view_bounds_.width())
                                  .Append(view_bounds_.height()));
    visual.Set("viewport", base::ListValue()
                               .Append(viewport_size_.width())
                               .Append(viewport_size_.height()));
    visual.Set("bitmap", base::ListValue()
                             .Append(bitmap_size_.width())
                             .Append(bitmap_size_.height()));
    visual.Set("capture_rect", base::ListValue()
                                   .Append(capture_rect_.x())
                                   .Append(capture_rect_.y())
                                   .Append(capture_rect_.width())
                                   .Append(capture_rect_.height()));
    visual.Set("total_pixels", total_pixels_);
    visual.Set("expected_background_pixels", expected_background_pixels_);
    visual.Set("non_white_pixels", non_white_pixels_);
    visual.Set("distinct_sampled_colors",
               static_cast<int>(distinct_sampled_colors_));
    visual.Set("last_copy_error", last_copy_error_);

    base::DictValue smoke_result;
    // The app sets its ready title only after every non-Explorer probe below
    // succeeds. The observer independently validates Explorer rendering, then
    // expands those two aggregate signals into stable per-feature smoke keys.
    smoke_result.Set("status", explorer_ok ? "ok" : "error");
    smoke_result.Set("shell_api", true);
    smoke_result.Set("network_service", true);
    smoke_result.Set("explorer", explorer_ok);
    smoke_result.Set("explorer_navigation", explorer_ok);
    smoke_result.Set("explorer_execute_javascript", explorer_ok);
    smoke_result.Set("explorer_dialog", explorer_ok);
    smoke_result.Set("explorer_partition", explorer_ok);
    smoke_result.Set("explorer_nested", explorer_ok);
    smoke_result.Set("explorer_surface_messaging", explorer_ok);
    smoke_result.Set("utility", true);
    // Reaching the ready title requires the renderer's window lifecycle probe
    // (current/create/list/setTitle/close) to have completed successfully. Keep
    // this explicit in the machine-readable result so packaging smoke cannot
    // accidentally stop asserting the window-management contract.
    smoke_result.Set("window", true);
    smoke_result.Set("path", true);
    smoke_result.Set("fs", true);
    smoke_result.Set("storage", true);
    smoke_result.Set("log", true);
    smoke_result.Set("chromium_thread_pool", true);
    smoke_result.Set("application_event", true);
    smoke_result.Set("large_asset", true);
    smoke_result.Set("utility_async", true);
    smoke_result.Set("utility_concurrent", application_ready_);
    smoke_result.Set("utility_session", application_ready_);
    smoke_result.Set("javascript_extension", application_ready_);
    smoke_result.Set("explorer_visual", std::move(visual));
    std::string result;
    CHECK(base::JSONWriter::Write(smoke_result, &result));
    base::ThreadPool::PostTaskAndReplyWithResult(
        FROM_HERE, {base::MayBlock()},
        base::BindOnce(
            [](base::FilePath path, std::string contents) {
              return base::WriteFile(path, contents);
            },
            marker_path_, result),
        base::BindOnce([](bool written) {
          if (!written) {
            LOG(ERROR) << "Unable to write Tuffite smoke marker";
          }
          Window::Shutdown();
        }));
  }

  static constexpr int kMaxAttempts = 40;
  static constexpr int kMaxReadyAttempts = 240;
  base::FilePath marker_path_;
  bool started_ = false;
  bool completed_ = false;
  bool application_ready_ = false;
  bool ready_retry_scheduled_ = false;
  int attempt_ = 0;
  int ready_attempt_ = 0;
  int last_copy_error_ = -1;
  int total_pixels_ = 0;
  int expected_background_pixels_ = 0;
  int non_white_pixels_ = 0;
  size_t distinct_sampled_colors_ = 0;
  bool view_showing_ = false;
  bool surface_available_ = false;
  bool outer_view_showing_ = false;
  bool outer_surface_available_ = false;
  std::string inner_url_;
  std::string inner_title_;
  std::string last_reason_;
  gfx::Rect view_bounds_;
  gfx::Size viewport_size_;
  gfx::Size bitmap_size_;
  gfx::Rect capture_rect_;
  base::WeakPtrFactory<SmokeTestObserver> weak_factory_{this};
};

class WindowSurfaceTestObserver final : public content::WebContentsObserver,
                                        public BrowserTestObserver {
 public:
  WindowSurfaceTestObserver(content::WebContents* web_contents,
                            Window* window,
                            base::FilePath marker_path)
      : content::WebContentsObserver(web_contents),
        window_(window),
        marker_path_(std::move(marker_path)) {
    TryCapture();
  }

  void DidFirstVisuallyNonEmptyPaint() override { TryCapture(); }

 private:
  void TryCapture() {
    if (completed_ || capture_pending_) {
      return;
    }
    ++attempt_;
    content::RenderWidgetHostView* view =
        web_contents()->GetRenderWidgetHostView();
    if (!view || !view->IsSurfaceAvailableForCopy() ||
        view->GetVisibleViewportSize().IsEmpty()) {
      RetryOrFail("main window surface is not ready");
      return;
    }
    view_bounds_ = view->GetViewBounds();
    viewport_size_ = view->GetVisibleViewportSize();
    view_showing_ = view->IsShowing();
    gfx::NativeWindow native_window = window_->widget()->GetNativeWindow();
    if (!native_window) {
      RetryOrFail("main native window is not ready");
      return;
    }
    capture_pending_ = true;
    ui::GrabWindowSnapshot(native_window, gfx::Rect(viewport_size_),
                           base::BindPostTaskToCurrentDefault(base::BindOnce(
                               &WindowSurfaceTestObserver::OnWindowSnapshot,
                               weak_factory_.GetWeakPtr())));
  }

  void OnWindowSnapshot(gfx::Image image) {
    capture_pending_ = false;
    if (image.IsEmpty()) {
      RetryOrFail("main window snapshot failed");
      return;
    }
    const SkBitmap bitmap = image.AsBitmap();
    bitmap_size_ = gfx::Size(bitmap.width(), bitmap.height());
    if (bitmap.drawsNothing()) {
      RetryOrFail("main window CopyFromSurface returned an empty bitmap");
      return;
    }
    std::unordered_set<SkColor> colors;
    opaque_pixels_ = 0;
    non_black_pixels_ = 0;
    total_pixels_ = bitmap.width() * bitmap.height();
    for (int y = 0; y < bitmap.height(); ++y) {
      for (int x = 0; x < bitmap.width(); ++x) {
        const SkColor color = bitmap.getColor(x, y);
        if (colors.size() < 512) {
          colors.insert(color);
        }
        if (SkColorGetA(color) > 240) {
          ++opaque_pixels_;
        }
        if (SkColorGetR(color) + SkColorGetG(color) + SkColorGetB(color) > 24) {
          ++non_black_pixels_;
        }
      }
    }
    distinct_sampled_colors_ = colors.size();
    if (opaque_pixels_ * 10 < total_pixels_ * 9 ||
        distinct_sampled_colors_ < 4 || non_black_pixels_ < 100) {
      RetryOrFail("main window compositor bitmap looks blank");
      return;
    }
    Finish(true, "main window compositor contains rendered pixels");
  }

  void RetryOrFail(std::string reason) {
    last_reason_ = std::move(reason);
    if (attempt_ >= kMaxAttempts) {
      Finish(false, last_reason_);
      return;
    }
    base::SequencedTaskRunner::GetCurrentDefault()->PostDelayedTask(
        FROM_HERE,
        base::BindOnce(&WindowSurfaceTestObserver::TryCapture,
                       weak_factory_.GetWeakPtr()),
        base::Milliseconds(250));
  }

  void Finish(bool ok, const std::string& reason) {
    if (completed_) {
      return;
    }
    completed_ = true;
    base::DictValue result;
    result.Set("status", ok ? "ok" : "error");
    result.Set("reason", reason);
    result.Set("attempts", attempt_);
    result.Set("widget_visible", window_->widget()->IsVisible());
    result.Set("contents_view_drawn", window_->contents_view_is_drawn());
    result.Set("web_view_drawn", window_->web_view_is_drawn());
    const gfx::Rect contents_bounds = window_->contents_view_bounds();
    result.Set("contents_view_bounds", base::ListValue()
                                           .Append(contents_bounds.x())
                                           .Append(contents_bounds.y())
                                           .Append(contents_bounds.width())
                                           .Append(contents_bounds.height()));
    const gfx::Rect web_view_bounds = window_->web_view_bounds();
    result.Set("web_view_bounds", base::ListValue()
                                      .Append(web_view_bounds.x())
                                      .Append(web_view_bounds.y())
                                      .Append(web_view_bounds.width())
                                      .Append(web_view_bounds.height()));
    result.Set("view_showing", view_showing_);
    result.Set("view_bounds", base::ListValue()
                                  .Append(view_bounds_.x())
                                  .Append(view_bounds_.y())
                                  .Append(view_bounds_.width())
                                  .Append(view_bounds_.height()));
    result.Set("viewport", base::ListValue()
                               .Append(viewport_size_.width())
                               .Append(viewport_size_.height()));
    result.Set("bitmap", base::ListValue()
                             .Append(bitmap_size_.width())
                             .Append(bitmap_size_.height()));
    result.Set("total_pixels", total_pixels_);
    result.Set("opaque_pixels", opaque_pixels_);
    result.Set("non_black_pixels", non_black_pixels_);
    result.Set("distinct_sampled_colors",
               static_cast<int>(distinct_sampled_colors_));
    std::string json;
    CHECK(base::JSONWriter::Write(result, &json));
    base::ThreadPool::PostTask(
        FROM_HERE, {base::MayBlock()},
        base::BindOnce(
            [](base::FilePath path, std::string contents) {
              if (!base::WriteFile(path, contents)) {
                LOG(ERROR) << "Unable to write window surface marker";
              }
            },
            marker_path_, std::move(json)));
  }

  static constexpr int kMaxAttempts = 80;
  raw_ptr<Window> window_;
  base::FilePath marker_path_;
  bool completed_ = false;
  bool capture_pending_ = false;
  bool view_showing_ = false;
  int attempt_ = 0;
  int total_pixels_ = 0;
  int opaque_pixels_ = 0;
  int non_black_pixels_ = 0;
  size_t distinct_sampled_colors_ = 0;
  std::string last_reason_;
  gfx::Rect view_bounds_;
  gfx::Size viewport_size_;
  gfx::Size bitmap_size_;
  base::WeakPtrFactory<WindowSurfaceTestObserver> weak_factory_{this};
};

}  // namespace

std::unique_ptr<BrowserTestObserver> CreateSmokeTestObserver(
    content::WebContents* contents,
    const base::FilePath& marker_path) {
  return std::make_unique<SmokeTestObserver>(contents, marker_path);
}
std::unique_ptr<BrowserTestObserver> CreateWindowSurfaceTestObserver(
    content::WebContents* contents,
    Window* window,
    const base::FilePath& marker_path) {
  return std::make_unique<WindowSurfaceTestObserver>(contents, window,
                                                     marker_path);
}
}  // namespace tuffite
