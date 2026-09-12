// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_WEB_CONTENTS_VIEW_DELEGATE_H_
#define TUFFITE_FRAMEWORK_BROWSER_WEB_CONTENTS_VIEW_DELEGATE_H_
#include <memory>
namespace content {
class WebContents;
class WebContentsViewDelegate;
}  // namespace content
namespace tuffite {
class ContextMenuService;
std::unique_ptr<content::WebContentsViewDelegate> CreateWebContentsViewDelegate(
    content::WebContents* contents,
    ContextMenuService& context_menus);
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_WEB_CONTENTS_VIEW_DELEGATE_H_
