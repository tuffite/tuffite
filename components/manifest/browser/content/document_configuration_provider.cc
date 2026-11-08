// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/manifest/browser/content/document_configuration_provider.h"

#include <memory>
#include <utility>

#include "base/supports_user_data.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/render_frame_host.h"
#include "third_party/blink/public/mojom/frame/policy_container.mojom.h"
#include "tuffite/components/manifest/common/document_configuration.mojom.h"
#include "url/origin.h"

namespace tuffite {
namespace {
const char kProviderKey = 0;
class ProviderData final : public base::SupportsUserData::Data {
 public:
  explicit ProviderData(DocumentConfigurationProvider value)
      : callback(std::move(value)) {}
  DocumentConfigurationProvider callback;
};
}  // namespace

void SetDocumentConfigurationProvider(content::BrowserContext& context,
                                      DocumentConfigurationProvider provider) {
  context.SetUserData(&kProviderKey,
                      std::make_unique<ProviderData>(std::move(provider)));
}

void ConfigureDocument(content::RenderFrameHost& frame,
                       const url::Origin& origin,
                       blink::mojom::PolicyContainer* container) {
  if (!container || !container->policies) {
    return;
  }
  container->policies->tuffite_configuration.reset();
  auto* provider = static_cast<ProviderData*>(
      frame.GetBrowserContext()->GetUserData(&kProviderKey));
  if (!origin.opaque() && provider && provider->callback) {
    const auto exposure = provider->callback.Run(frame, origin);
    if (exposure.shell_api || exposure.explorer) {
      container->policies->tuffite_configuration =
          mojom::DocumentConfiguration::New(exposure.shell_api,
                                            exposure.explorer);
    }
  }
}

}  // namespace tuffite
