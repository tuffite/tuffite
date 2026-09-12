// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_DOCUMENT_PICTURE_IN_PICTURE_WINDOW_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_DOCUMENT_PICTURE_IN_PICTURE_WINDOW_H_
#include "content/public/browser/web_contents.h"
#include "third_party/blink/public/mojom/window_features/window_features.mojom-forward.h"
namespace tuffite {
// Creates a same-origin, script-related Document PiP window. The opener owns
// its lifetime; no ShellAPI permissions or application role are inherited.
content::WebContents* ShowDocumentPictureInPictureWindow(
    content::WebContents* source,
    std::unique_ptr<content::WebContents> child,
    const blink::mojom::WindowFeatures& features);
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_DOCUMENT_PICTURE_IN_PICTURE_WINDOW_H_
