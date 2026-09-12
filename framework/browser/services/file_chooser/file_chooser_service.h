// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_FILE_CHOOSER_FILE_CHOOSER_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_FILE_CHOOSER_FILE_CHOOSER_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"
#include "content/public/browser/file_select_listener.h"

namespace content {
class RenderFrameHost;
class WebContents;
}  // namespace content

namespace tuffite {
class FileChooserService : public KeyedService {
 public:
  virtual void Run(content::RenderFrameHost* frame,
                   scoped_refptr<content::FileSelectListener> listener,
                   const blink::mojom::FileChooserParams& params) = 0;
  virtual void Enumerate(content::WebContents* contents,
                         scoped_refptr<content::FileSelectListener> listener,
                         const base::FilePath& path) = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_FILE_CHOOSER_FILE_CHOOSER_SERVICE_H_
