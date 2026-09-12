// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/document_picture_in_picture_window.h"

#include <algorithm>

#include "base/functional/bind.h"
#include "base/memory/weak_ptr.h"
#include "base/strings/utf_string_conversions.h"
#include "base/task/single_thread_task_runner.h"
#include "content/public/browser/document_picture_in_picture_window_controller.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents_delegate.h"
#include "content/public/browser/web_contents_observer.h"
#include "content/public/browser/web_contents_user_data.h"
#include "third_party/blink/public/mojom/picture_in_picture_window_options/picture_in_picture_window_options.mojom.h"
#include "third_party/blink/public/mojom/window_features/window_features.mojom.h"
#include "ui/display/display.h"
#include "ui/display/screen.h"
#include "ui/views/controls/webview/webview.h"
#include "ui/views/widget/widget.h"
#include "ui/views/widget/widget_delegate.h"
#include "ui/views/widget/widget_observer.h"
namespace tuffite {
namespace {

class DocumentPipWindow final
    : public content::WebContentsUserData<DocumentPipWindow>,
      public content::WebContentsObserver,
      public content::WebContentsDelegate,
      public views::WidgetObserver {
 public:
  // The source WebContents owns this user data and tears it down with its
  // observers. Release the child and widget here, without a destruction
  // callback.
  ~DocumentPipWindow() override { Reset(); }

  content::WebContents* Open(std::unique_ptr<content::WebContents> child,
                             const blink::mojom::WindowFeatures& features) {
    Reset();
    auto* source = content::WebContentsObserver::web_contents();
    const gfx::Size size(
        std::clamp(features.bounds.width() > 0 ? features.bounds.width() : 480,
                   240, 1280),
        std::clamp(
            features.bounds.height() > 0 ? features.bounds.height() : 270, 160,
            960));
    child_ = std::move(child);
    child_->SetDelegate(this);
    auto* controller = content::PictureInPictureWindowController::
        GetOrCreateDocumentPictureInPictureController(
            content::WebContentsObserver::web_contents());
    controller->SetChildWebContents(child_.get());
    auto view = std::make_unique<views::WebView>(source->GetBrowserContext());
    view_ = view.get();
    delegate_ = std::make_unique<views::WidgetDelegate>();
    delegate_->SetContentsView(std::move(view));
    delegate_->SetTitle(base::UTF8ToUTF16(
        source->GetPrimaryMainFrame()->GetLastCommittedOrigin().Serialize()));
    delegate_->SetCanResize(true);
    widget_destroyed_ = false;
    widget_ = std::make_unique<views::Widget>();
    views::Widget::InitParams init(
        views::Widget::InitParams::CLIENT_OWNS_WIDGET,
        views::Widget::InitParams::TYPE_WINDOW);
    init.delegate = delegate_.get();
    init.name = "TuffiteDocumentPictureInPictureWindow";
    init.z_order = ui::ZOrderLevel::kFloatingWindow;
    init.visible_on_all_workspaces = true;
    const auto work = display::Screen::Get()->GetPrimaryDisplay().work_area();
    init.bounds = gfx::Rect(work.right() - size.width() - 24,
                            work.bottom() - size.height() - 24, size.width(),
                            size.height());
    widget_->Init(std::move(init));
    widget_->AddObserver(this);
    view_->SetWebContents(child_.get());
    controller->Show();
    widget_->Show();
    widget_->GetContentsView()->InvalidateLayout();
    widget_->GetContentsView()->DeprecatedLayoutImmediately();
    child_->WasShown();
    return child_.get();
  }

  void CloseContents(content::WebContents*) override {
    if (widget_) {
      widget_->Close();
    }
  }

  std::optional<gfx::Rect> GetWindowBoundsInScreen() override {
    return widget_ ? widget_->GetWindowBoundsInScreen() : gfx::Rect();
  }

  void OnWidgetDestroyed(views::Widget*) override {
    widget_destroyed_ = true;
    view_ = nullptr;
  }

  void OnWidgetClosing(views::Widget*) override {
    if (view_) {
      view_->SetWebContents(nullptr);
      view_ = nullptr;
    }
    // Destroy the child after the widget callback, allowing pagehide and the
    // controller's child observer to run without reentrant widget destruction.
    base::SingleThreadTaskRunner::GetCurrentDefault()->PostTask(
        FROM_HERE,
        base::BindOnce(&DocumentPipWindow::Reset, weak_.GetWeakPtr()));
  }

  void PrimaryPageChanged(content::Page&) override { Reset(); }

 private:
  friend class content::WebContentsUserData<DocumentPipWindow>;
  explicit DocumentPipWindow(content::WebContents* source)
      : WebContentsUserData(*source), WebContentsObserver(source) {}
  void Reset() {
    weak_.InvalidateWeakPtrs();
    if (view_) {
      view_->SetWebContents(nullptr);
      view_ = nullptr;
    }
    if (widget_) {
      widget_->RemoveObserver(this);
      if (!widget_destroyed_) {
        widget_->CloseNow();
      }
      widget_.reset();
    }
    delegate_.reset();
    if (child_) {
      child_->SetDelegate(nullptr);
      child_.reset();
    }
  }
  std::unique_ptr<content::WebContents> child_;
  std::unique_ptr<views::Widget> widget_;
  std::unique_ptr<views::WidgetDelegate> delegate_;
  bool widget_destroyed_ = false;
  raw_ptr<views::WebView> view_ = nullptr;
  base::WeakPtrFactory<DocumentPipWindow> weak_{this};
  WEB_CONTENTS_USER_DATA_KEY_DECL();
};

WEB_CONTENTS_USER_DATA_KEY_IMPL(DocumentPipWindow);
}  // namespace

content::WebContents* ShowDocumentPictureInPictureWindow(
    content::WebContents* source,
    std::unique_ptr<content::WebContents> child,
    const blink::mojom::WindowFeatures& features) {
  DocumentPipWindow::CreateForWebContents(source);
  return DocumentPipWindow::FromWebContents(source)->Open(std::move(child),
                                                          features);
}
}  // namespace tuffite
