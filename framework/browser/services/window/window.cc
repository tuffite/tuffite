// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/window.h"

#include <algorithm>
#include <set>
#include <utility>

#include "base/no_destructor.h"
#include "base/task/single_thread_task_runner.h"
#include "build/build_config.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/navigation_controller.h"
#include "content/public/browser/navigation_entry.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/page_navigator.h"
#include "content/public/browser/picture_in_picture_window_controller.h"
#include "content/public/browser/video_picture_in_picture_window_controller.h"
#include "content/public/browser/web_contents.h"
#include "third_party/blink/public/common/renderer_preferences/renderer_preferences.h"
#include "third_party/blink/public/mojom/page/draggable_region.mojom.h"
#include "third_party/skia/include/core/SkColor.h"
#include "third_party/skia/include/core/SkRegion.h"
#include "tuffite/framework/browser/services/file_chooser/file_chooser_service.h"
#include "tuffite/framework/browser/services/file_chooser/file_chooser_service_factory.h"
#include "tuffite/framework/browser/services/javascript_dialog/javascript_dialog_service.h"
#include "tuffite/framework/browser/services/javascript_dialog/javascript_dialog_service_factory.h"
#include "tuffite/framework/browser/services/window/document_picture_in_picture_window.h"
#include "tuffite/framework/browser/services/window/platform.h"
#include "tuffite/framework/browser/services/window/window_service.h"
#include "tuffite/framework/browser/services/window/window_service_factory.h"
#include "tuffite/framework/common/languages.h"
#if !BUILDFLAG(IS_MAC)
#include "ui/aura/window.h"
#include "ui/aura/window_tree_host.h"
#endif
#include "ui/base/hit_test.h"
#include "ui/base/page_transition_types.h"
#include "ui/gfx/geometry/rounded_corners_f.h"
#include "ui/views/controls/webview/web_contents_set_background_color.h"
#include "ui/views/controls/webview/webview.h"
#include "ui/views/view.h"
#include "ui/views/widget/widget.h"
#include "ui/views/window/client_view.h"
#if BUILDFLAG(IS_WIN)
#include "ui/display/win/screen_win.h"
#endif

namespace tuffite {
namespace {

std::vector<Window*>& MutableWindows() {
  static base::NoDestructor<std::vector<Window*>> windows;
  return *windows;
}

base::OnceClosure& QuitClosure() {
  static base::NoDestructor<base::OnceClosure> closure;
  return *closure;
}

int ResizeHitTest(const gfx::Point& point,
                  const gfx::Size& size,
                  int border,
                  int corner) {
  if (!gfx::Rect(size).Contains(point)) {
    return HTNOWHERE;
  }
  const bool top = point.y() < border;
  const bool bottom = point.y() >= size.height() - border;
  const bool left = point.x() < border;
  const bool right = point.x() >= size.width() - border;
  if (!top && !bottom && !left && !right) {
    return HTNOWHERE;
  }
  const bool near_top = point.y() < corner;
  const bool near_bottom = point.y() >= size.height() - corner;
  const bool near_left = point.x() < corner;
  const bool near_right = point.x() >= size.width() - corner;
  if ((top && near_left) || (left && near_top)) {
    return HTTOPLEFT;
  }
  if ((top && near_right) || (right && near_top)) {
    return HTTOPRIGHT;
  }
  if ((bottom && near_left) || (left && near_bottom)) {
    return HTBOTTOMLEFT;
  }
  if ((bottom && near_right) || (right && near_bottom)) {
    return HTBOTTOMRIGHT;
  }
  if (top) {
    return HTTOP;
  }
  if (bottom) {
    return HTBOTTOM;
  }
  return left ? HTLEFT : HTRIGHT;
}

class WindowClientView final : public views::ClientView {
 public:
  WindowClientView(Window* window,
                   views::Widget* widget,
                   views::View* contents_view)
      : views::ClientView(widget, contents_view), window_(window) {}

  int NonClientHitTest(const gfx::Point& point) override {
    views::Widget* widget = window_->widget();
    if (!window_->decorations() && widget && !widget->IsMaximized() &&
        !widget->IsFullscreen()) {
#if BUILDFLAG(IS_WIN)
      constexpr int kResizeCornerSize = 16;
      const int border = std::max(
          4,
          display::win::GetScreenWin()->GetSystemMetricsInDIP(SM_CXSIZEFRAME));
#else
      constexpr int kResizeCornerSize = 16;
      constexpr int border = 8;
#endif
      const int resize =
          widget->widget_delegate()->CanResize()
              ? ResizeHitTest(point, size(), border,
                              std::max(kResizeCornerSize, border))
              : HTNOWHERE;
      if (resize != HTNOWHERE) {
        return resize;
      }
    }
    if (window_->IsPointWithinDraggableRegion(point)) {
      return HTCAPTION;
    }
    return views::ClientView::NonClientHitTest(point);
  }

  gfx::Size GetMinimumSize() const override { return window_->minimum_size(); }
  gfx::Size GetMaximumSize() const override { return window_->maximum_size(); }

  void Layout(PassKey) override {
    gfx::Rect content_bounds = GetLocalBounds();
#if BUILDFLAG(IS_WIN)
    // The renderer owns a child HWND. Keep it off the outermost restored-window
    // pixels so WindowFromPoint() reaches the top-level HWND and Windows can
    // start its native sizing loop. Opaque composition is required here:
    // transparent top-level pixels are intentionally click-through on Windows.
    views::Widget* widget = window_->widget();
    if (!window_->decorations() && widget && !widget->IsMaximized() &&
        !widget->IsFullscreen()) {
      constexpr int kNativeResizeStrip = 2;
      content_bounds.Inset(kNativeResizeStrip);
    }
#endif
    if (contents_view()) {
      contents_view()->SetBoundsRect(content_bounds);
    }
  }

 private:
  const raw_ptr<Window> window_;
};

}  // namespace

Window::Window(content::BrowserContext* browser_context,
               const gfx::Size& size,
               bool decorations)
    : content::WebContentsObserver(nullptr), decorations_(decorations) {
  minimum_size_ = gfx::Size(1, 1);
  maximum_size_ = gfx::Size(16384, 16384);
  SetHasWindowSizeControls(true);

  web_contents_ = content::WebContents::Create(
      content::WebContents::CreateParams(browser_context));
  Observe(web_contents_.get());
  web_contents_->GetMutableRendererPrefs()->accept_languages = kAcceptLanguages;
  web_contents_->SyncRendererPrefs();
  web_contents_->SetDelegate(this);
  if (!decorations_) {
    web_contents_->SetSupportsDraggableRegions(true);
    views::WebContentsSetBackgroundColor::CreateForWebContentsWithColor(
        web_contents_.get(), SK_ColorTRANSPARENT);
  }
  auto contents = std::make_unique<views::View>();
  contents->SetUseDefaultFillLayout(true);
  web_view_ =
      contents->AddChildView(std::make_unique<views::WebView>(browser_context));
  SetContentsView(std::move(contents));
  MutableWindows().push_back(this);
}

Window::~Window() {
  if (web_contents_) {
    GetJavaScriptDialogManager(web_contents_.get())
        ->CancelDialogs(web_contents_.get(), true);
  }
  if (owner_) {
    owner_->Unregister(this);
  }
  std::erase(MutableWindows(), this);
  if (MutableWindows().empty() && QuitClosure()) {
    std::move(QuitClosure()).Run();
  }
}

Window* Window::Create(content::BrowserContext* browser_context,
                       const GURL& url,
                       const gfx::Size& size,
                       bool decorations,
                       const window::Constraints* constraints) {
  auto owned =
      std::unique_ptr<Window>(new Window(browser_context, size, decorations));
  Window* window = owned.get();
  if (constraints) {
    window->minimum_size_ = constraints->minimum_size;
    window->maximum_size_ = constraints->maximum_size;
  }
  window->widget_ = std::make_unique<views::Widget>();
  views::Widget::InitParams params(
      views::Widget::InitParams::CLIENT_OWNS_WIDGET,
      views::Widget::InitParams::TYPE_WINDOW);
  params.bounds = gfx::Rect(size);
  params.delegate = window;
  // Frameless desktop windows still ask the OS compositor for rounded outer
  // corners. Windows 11 maps any positive radius to DWMWCP_ROUND; macOS and
  // Ash honor the requested 12 DIP radius directly.
  params.rounded_corners = gfx::RoundedCornersF(12.0f);
  if (!decorations) {
#if BUILDFLAG(IS_WIN)
    // Views deliberately disables WS_THICKFRAME for translucent HWNDs and
    // transparent edge pixels are click-through. Use the normal opaque DWM
    // surface on Windows so the frameless window retains native edge resizing;
    // the Web application still owns all visible colors and translucency.
    params.opacity = views::Widget::InitParams::WindowOpacity::kOpaque;
#else
    params.opacity = views::Widget::InitParams::WindowOpacity::kTranslucent;
#endif
  }
#if BUILDFLAG(IS_MAC)
  // Create a normal resizable NSWindow, then extend the client content under
  // its transparent titlebar while retaining the native traffic lights.
  params.remove_standard_frame = false;
#else
  params.remove_standard_frame = !decorations;
#endif
#if BUILDFLAG(IS_LINUX)
  params.wm_class_class = "tuffite";
  params.wm_class_name = "tuffite";
#endif
  window->widget_->Init(std::move(params));
  if (!decorations) {
    window::ConfigureCustomTitlebar(window);
  }
  window->widget_->AddObserver(window);
  // Attach only after the Widget and native View hierarchy exist. WebView's
  // visibility propagation happens during attachment; attaching in the
  // delegate constructor can leave the RenderWidgetHostView with a valid
  // compositor Surface that is never presented by the desktop window. This
  // follows content_shell's proven SetContents ordering.
  window->web_view_->SetWebContents(window->web_contents_.get());
  window->web_contents_->Focus();
#if !BUILDFLAG(IS_MAC)
  window->widget_->GetNativeWindow()->GetHost()->Show();
#endif
  window->widget_->Show();
#if BUILDFLAG(IS_MAC)
  // Views may rebuild the native titlebar while showing the Widget. Reapply
  // the transparent full-size titlebar after that transition so AppKit's
  // traffic-light controls stay visible above the web content.
  if (!decorations) {
    window::ConfigureCustomTitlebar(window);
  }
#endif
  // NativeViewHost hides an attached native view while its owning Widget is
  // not drawn and relies on a later layout to call ShowWidget(). Force that
  // first visible layout now; otherwise the renderer can produce frames while
  // the HWND keeps presenting an empty client area.
  window->widget_->GetContentsView()->InvalidateLayout();
  window->widget_->GetContentsView()->DeprecatedLayoutImmediately();
  // Top-level frameworks own WebContents visibility. A WebContents starts with
  // Visibility::VISIBLE but is not considered visible for real until the host
  // calls WasShown(); without it the renderer may never produce the first
  // compositor frame.
  window->SyncWebContentsVisibility();
  auto* owner =
      window::WindowServiceFactory::GetForBrowserContext(browser_context);
  window->owner_ = owner->GetWeakPtr();
  owner->Adopt(std::move(owned));
  if (!url.is_empty()) {
    window->LoadURL(url);
  }
  return window;
}

Window* Window::FromWebContents(content::WebContents* web_contents) {
  auto found =
      std::ranges::find(MutableWindows(), web_contents, &Window::web_contents);
  return found == MutableWindows().end() ? nullptr : *found;
}

const std::vector<Window*>& Window::windows() {
  return MutableWindows();
}

void Window::SetQuitClosure(base::OnceClosure quit_closure) {
  QuitClosure() = std::move(quit_closure);
}

void Window::Shutdown() {
  // WindowService owns native windows; the global index is observational only.
  std::set<content::BrowserContext*> contexts;
  for (auto* window : MutableWindows()) {
    contexts.insert(window->web_contents()->GetBrowserContext());
  }
  for (auto* context : contexts) {
    window::WindowServiceFactory::GetForBrowserContext(context)->Shutdown();
  }
}
content::JavaScriptDialogManager* Window::GetJavaScriptDialogManager(
    content::WebContents* source) {
  return JavaScriptDialogServiceFactory::GetForBrowserContext(
      source->GetBrowserContext());
}

void Window::PrepareForShutdown() {
  if (web_contents_) {
    GetJavaScriptDialogManager(web_contents_.get())
        ->CancelDialogs(web_contents_.get(), true);
  }
  Observe(nullptr);
  if (widget_->HasObserver(this)) {
    widget_->RemoveObserver(this);
  }
  if (web_contents_shown_) {
    web_contents_->WasHidden();
    web_contents_shown_ = false;
  }
  web_contents_->SetDelegate(nullptr);
  if (web_view_) {
    web_view_->SetWebContents(nullptr);
    web_view_ = nullptr;
  }
  widget_->CloseNow();
}

views::Widget* Window::widget() const {
  return widget_.get();
}

gfx::Rect Window::contents_view_bounds() const {
  return widget_ && widget_->GetContentsView()
             ? widget_->GetContentsView()->bounds()
             : gfx::Rect();
}

gfx::Rect Window::web_view_bounds() const {
  return web_view_ ? web_view_->bounds() : gfx::Rect();
}

bool Window::contents_view_is_drawn() const {
  return widget_ && widget_->GetContentsView() &&
         widget_->GetContentsView()->IsDrawn();
}

bool Window::web_view_is_drawn() const {
  return web_view_ && web_view_->IsDrawn();
}

// bool Window::native_web_contents_view_is_visible() const {
//   return web_contents_ && web_contents_->GetNativeView() &&
//          web_contents_->GetNativeView()->IsVisible();
// }

bool Window::IsPointWithinDraggableRegion(const gfx::Point& point) const {
  return !decorations_ && !draggable_region_.isEmpty() &&
         draggable_region_.contains(point.x(), point.y());
}

void Window::Close() {
  widget()->Close();
}

void Window::LoadURL(const GURL& url) {
  content::NavigationController::LoadURLParams params(url);
  params.transition_type = ui::PAGE_TRANSITION_TYPED;
  web_contents_->GetController().LoadURLWithParams(params);
}

bool Window::CanResize() const {
  return true;
}

bool Window::CanMaximize() const {
  return true;
}

bool Window::CanMinimize() const {
  return true;
}

views::ClientView* Window::CreateClientView(views::Widget* widget) {
  return new WindowClientView(this, widget, TransferOwnershipOfContentsView());
}

bool Window::ShouldDescendIntoChildForEventHandling(
    gfx::NativeView child,
    const gfx::Point& location) {
  // The renderer owns a native child window. If hit testing descends into that
  // child along a frameless window edge, it consumes the mouse input before
  // the top-level HWND can enter its native sizing loop. The top edge happened
  // to work because it also overlaps the draggable titlebar; claim every
  // resize component explicitly so the other three edges behave the same way.
  if (!decorations_ && widget_ && widget_->client_view() &&
      ui::IsResizingComponent(
          widget_->client_view()->NonClientHitTest(location))) {
    return false;
  }
  return !IsPointWithinDraggableRegion(location);
}

void Window::DraggableRegionsChanged(
    const std::vector<blink::mojom::DraggableRegionPtr>& regions,
    content::WebContents* contents) {
  if (decorations_ || contents != web_contents_.get()) {
    return;
  }
  SkRegion next;
  for (const blink::mojom::DraggableRegionPtr& region : regions) {
    next.op(SkIRect::MakeXYWH(region->bounds.x(), region->bounds.y(),
                              region->bounds.width(), region->bounds.height()),
            region->draggable ? SkRegion::kUnion_Op : SkRegion::kDifference_Op);
  }
  draggable_region_.swap(next);
}

content::WebContents* Window::AddNewContents(
    content::WebContents* source,
    std::unique_ptr<content::WebContents> new_contents,
    const GURL&,
    WindowOpenDisposition disposition,
    const blink::mojom::WindowFeatures& features,
    bool,
    bool* was_blocked) {
  if (disposition != WindowOpenDisposition::NEW_PICTURE_IN_PICTURE || !source) {
    if (was_blocked) {
      *was_blocked = true;
    }
    return nullptr;
  }
  if (was_blocked) {
    *was_blocked = false;
  }
  return ShowDocumentPictureInPictureWindow(source, std::move(new_contents),
                                            features);
}

void Window::CloseContents(content::WebContents*) {
  Close();
}

void Window::RunFileChooser(content::RenderFrameHost* frame,
                            scoped_refptr<content::FileSelectListener> listener,
                            const blink::mojom::FileChooserParams& params) {
  FileChooserServiceFactory::GetForBrowserContext(frame->GetBrowserContext())
      ->Run(frame, std::move(listener), params);
}

void Window::EnumerateDirectory(
    content::WebContents* contents,
    scoped_refptr<content::FileSelectListener> listener,
    const base::FilePath& path) {
  FileChooserServiceFactory::GetForBrowserContext(contents->GetBrowserContext())
      ->Enumerate(contents, std::move(listener), path);
}

content::WebContents* Window::OpenURLFromTab(
    content::WebContents* source,
    const content::OpenURLParams& params,
    base::OnceCallback<void(content::NavigationHandle&)> callback) {
  if (source != web_contents_.get() ||
      params.disposition != WindowOpenDisposition::CURRENT_TAB) {
    return nullptr;
  }
  auto handle = source->GetController().LoadURLWithParams(
      content::NavigationController::LoadURLParams(params));
  if (handle && callback) {
    std::move(callback).Run(*handle);
  }
  return source;
}

void Window::TitleWasSet(content::NavigationEntry* entry) {
  if (entry) {
    SetTitle(entry->GetTitle());
  }
}

void Window::EnterFullscreenModeForTab(
    content::RenderFrameHost* requesting_frame,
    const blink::mojom::FullscreenOptions& options) {
  window::SetFullscreen(this, true);
}

void Window::ExitFullscreenModeForTab(content::WebContents* web_contents) {
  window::SetFullscreen(this, false);
}

bool Window::IsFullscreenForTabOrPending(
    const content::WebContents* web_contents) {
  return widget_ && widget_->IsFullscreen();
}

content::PictureInPictureResult Window::EnterPictureInPicture(
    content::WebContents* web_contents) {
  if (picture_in_picture_contents_ &&
      picture_in_picture_contents_.get() != web_contents) {
    ExitPictureInPicture();
  }
  content::PictureInPictureWindowController::
      GetOrCreateVideoPictureInPictureController(web_contents);
  picture_in_picture_contents_ = web_contents->GetWeakPtr();
  return content::PictureInPictureResult::kSuccess;
}

void Window::ExitPictureInPicture() {
  // The source can be destroyed or switch to Document PiP independently of
  // its top-level window. Never retain a raw controller across those changes.
  if (picture_in_picture_contents_ &&
      picture_in_picture_contents_->HasPictureInPictureVideo()) {
    content::PictureInPictureWindowController::
        GetOrCreateVideoPictureInPictureController(
            picture_in_picture_contents_.get())
            ->Close(/*should_pause_video=*/false);
  }
  picture_in_picture_contents_.reset();
}

void Window::WindowClosing() {
  Observe(nullptr);
  if (widget_ && widget_->HasObserver(this)) {
    widget_->RemoveObserver(this);
  }
  if (web_contents_shown_) {
    web_contents_->WasHidden();
    web_contents_shown_ = false;
  }
  web_contents_->SetDelegate(nullptr);
  if (web_view_) {
    web_view_->SetWebContents(nullptr);
    web_view_ = nullptr;
  }
  if (owner_) {
    owner_->DestroyWhenIdle(this);
  }
}

void Window::OnWidgetVisibilityChanged(views::Widget* widget, bool visible) {
  CHECK_EQ(widget, widget_.get());
  SyncWebContentsVisibility();
}

void Window::OnWidgetShowStateChanged(views::Widget* widget) {
  CHECK_EQ(widget, widget_.get());
  SyncWebContentsVisibility();
}

void Window::SyncWebContentsVisibility() {
  if (!widget_ || !web_contents_) {
    return;
  }
  const bool should_show = widget_->IsVisible() && !widget_->IsMinimized();
  if (should_show == web_contents_shown_) {
    return;
  }
  if (should_show) {
    web_contents_->WasShown();
  } else {
    web_contents_->WasHidden();
  }
  web_contents_shown_ = should_show;
}

}  // namespace tuffite
