// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_MANIFEST_BROWSER_CONTENT_DOCUMENT_CONFIGURATION_PROVIDER_H_
#define TUFFITE_COMPONENTS_MANIFEST_BROWSER_CONTENT_DOCUMENT_CONFIGURATION_PROVIDER_H_

#include "base/functional/callback.h"
#include "content/common/content_export.h"
#include "third_party/blink/public/mojom/frame/policy_container.mojom-forward.h"

namespace content {
class BrowserContext;
class RenderFrameHost;
}  // namespace content

namespace url {
class Origin;
}  // namespace url

namespace tuffite {

struct DocumentExposure {
  bool shell_api = false;
  bool explorer = false;
};

using DocumentConfigurationProvider =
    base::RepeatingCallback<DocumentExposure(content::RenderFrameHost&,
                                             const url::Origin&)>;
// The callback is owned by this BrowserContext and resolves document exposure
// before existing frame-creation or navigation-commit Mojo messages are sent.
CONTENT_EXPORT void SetDocumentConfigurationProvider(
    content::BrowserContext& context,
    DocumentConfigurationProvider provider);

CONTENT_EXPORT void ConfigureDocument(content::RenderFrameHost& frame,
                                      const url::Origin& origin,
                                      blink::mojom::PolicyContainer* container);

}  // namespace tuffite

#endif
