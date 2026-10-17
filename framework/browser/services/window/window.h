// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_H_

#include <memory>
#include <string>
#include <vector>

#include "base/functional/callback_forward.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "content/public/browser/web_contents_delegate.h"
#include "content/public/browser/web_contents_observer.h"
#include "third_party/blink/public/mojom/page/draggable_region.mojom-forward.h"
#include "third_party/skia/include/core/SkRegion.h"
#include "tuffite/framework/browser/services/window/window_types.h"
#include "ui/gfx/geometry/rect.h"
#include "ui/gfx/geometry/size.h"
#include "ui/views/controls/webview/unhandled_keyboard_event_handler.h"
#include "ui/views/widget/widget_delegate.h"
#include "ui/views/widget/widget_observer.h"

class GURL;

namespace content {
class BrowserContext;
class VideoPictureInPictureWindowController;
class WebContents;
}  // namespace content

namespace views {
class WebView;
class Widget;
class ClientView;
}  // namespace views

namespace tuffite {
namespace window {
class WindowService;
struct Constraints;
}  // namespace window

class Window final : public views::WidgetDelegate,
                     public views::WidgetObserver,
                     public content::WebContentsDelegate,
                     public content::WebContentsObserver {
 public:
  static Window* Create(content::BrowserContext* browser_context,
                        const GURL& url,
                        const gfx::Size& size,
                        bool decorations = true,
                        const window::Constraints* constraints = nullptr);
  static Window* FromWebContents(content::WebContents* web_contents);
  static const std::vector<Window*>& windows();
  static void SetQuitClosure(base::OnceClosure quit_closure);
  static void Shutdown();

  Window(const Window&) = delete;
  Window& operator=(const Window&) = delete;
  ~Window() override;

  content::WebContents* web_contents() const { return web_contents_.get(); }
  views::Widget* widget() const;
  gfx::Rect contents_view_bounds() const;
  gfx::Rect web_view_bounds() const;
  bool contents_view_is_drawn() const;
  bool web_view_is_drawn() const;
  //   bool native_web_contents_view_is_visible() const;
  bool decorations() const { return decorations_; }
  window::Constraints constraints() const {
    return {minimum_size_, maximum_size_};
  }
  gfx::Size minimum_size() const { return minimum_size_; }
  gfx::Size maximum_size() const { return maximum_size_; }
  bool IsPointWithinDraggableRegion(const gfx::Point& point) const;
  void Close();
  // Synchronous final teardown, used by the owning WindowService.
  void PrepareForShutdown();
  void LoadURL(const GURL& url);

  bool CanResize() const override;
  bool CanMaximize() const override;
  bool CanMinimize() const override;
  views::ClientView* CreateClientView(views::Widget* widget) override;
  bool ShouldDescendIntoChildForEventHandling(
      gfx::NativeView child,
      const gfx::Point& location) override;
  content::WebContents* AddNewContents(
      content::WebContents* source,
      std::unique_ptr<content::WebContents> new_contents,
      const GURL& target_url,
      WindowOpenDisposition disposition,
      const blink::mojom::WindowFeatures& features,
      bool user_gesture,
      bool* was_blocked) override;
  content::JavaScriptDialogManager* GetJavaScriptDialogManager(
      content::WebContents* source) override;
  void RunFileChooser(content::RenderFrameHost* frame,
                      scoped_refptr<content::FileSelectListener> listener,
                      const blink::mojom::FileChooserParams& params) override;
  void EnumerateDirectory(content::WebContents* contents,
                          scoped_refptr<content::FileSelectListener> listener,
                          const base::FilePath& path) override;
  content::WebContents* OpenURLFromTab(
      content::WebContents* source,
      const content::OpenURLParams& params,
      base::OnceCallback<void(content::NavigationHandle&)> callback) override;
  void CloseContents(content::WebContents* source) override;
  void ActivateContents(content::WebContents* contents) override;
  bool HandleKeyboardEvent(content::WebContents* source,
                           const input::NativeWebKeyboardEvent& event) override;
  std::optional<gfx::Rect> GetWindowBoundsInScreen() override;
  void RequestKeyboardLock(content::WebContents* contents,
                           bool esc_key_locked) override;
  void DraggableRegionsChanged(
      const std::vector<blink::mojom::DraggableRegionPtr>& regions,
      content::WebContents* contents) override;
  void TitleWasSet(content::NavigationEntry* entry) override;
  void EnterFullscreenModeForTab(
      content::RenderFrameHost* requesting_frame,
      const blink::mojom::FullscreenOptions& options) override;
  void ExitFullscreenModeForTab(content::WebContents* web_contents) override;
  bool IsFullscreenForTabOrPending(
      const content::WebContents* web_contents) override;
  content::PictureInPictureResult EnterPictureInPicture(
      content::WebContents* web_contents) override;
  void ExitPictureInPicture() override;
  void WindowClosing() override;
  void OnWidgetVisibilityChanged(views::Widget* widget, bool visible) override;
  void OnWidgetShowStateChanged(views::Widget* widget) override;

 private:
  Window(content::BrowserContext* browser_context,
         const gfx::Size& size,
         bool decorations);
  void SyncWebContentsVisibility();

  std::unique_ptr<content::WebContents> web_contents_;
  std::unique_ptr<views::Widget> widget_;
  raw_ptr<views::WebView> web_view_ = nullptr;
  SkRegion draggable_region_;
  gfx::Size minimum_size_;
  gfx::Size maximum_size_;
  bool decorations_ = true;
  bool web_contents_shown_ = false;
  base::WeakPtr<content::WebContents> picture_in_picture_contents_;
  base::WeakPtr<window::WindowService> owner_;
  views::UnhandledKeyboardEventHandler unhandled_keyboard_event_handler_;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_H_
