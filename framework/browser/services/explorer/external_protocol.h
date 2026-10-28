// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXTERNAL_PROTOCOL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXTERNAL_PROTOCOL_H_

#include <optional>

#include "url/origin.h"

namespace content {
class WebContents;
class RenderFrameHost;
}  // namespace content
namespace tuffite {
void ConfirmExternalProtocol(
    content::WebContents* contents,
    const GURL& url,
    const std::optional<url::Origin>& initiating_origin,
    content::RenderFrameHost* initiator_document);
}
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_EXPLORER_EXTERNAL_PROTOCOL_H_
