// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/web_contents_view_delegate.h"

#include <utility>

#include "content/public/browser/web_contents_view_delegate.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service.h"
namespace tuffite {
namespace {
class TuffiteWebContentsViewDelegate final
    : public content::WebContentsViewDelegate {
 public:
  explicit TuffiteWebContentsViewDelegate(
      std::unique_ptr<ContextMenuSession> menu)
      : menu_(std::move(menu)) {}
  void ShowContextMenu(content::RenderFrameHost& frame,
                       const content::ContextMenuParams& params) override {
    menu_->Show(frame, params);
  }
  void DismissContextMenu() override { menu_->Dismiss(); }

 private:
  const std::unique_ptr<ContextMenuSession> menu_;
};
}  // namespace
std::unique_ptr<content::WebContentsViewDelegate> CreateWebContentsViewDelegate(
    content::WebContents* contents,
    ContextMenuService& context_menus) {
  return std::make_unique<TuffiteWebContentsViewDelegate>(
      context_menus.CreateSession(contents));
}
}  // namespace tuffite
