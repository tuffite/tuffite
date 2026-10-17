// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/context_menu/context_menu_service_impl.h"

#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_session.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"

namespace tuffite {
ContextMenuServiceImpl::ContextMenuServiceImpl(ManifestService& manifest,
                                               DevToolsService& devtools)
    : manifest_(manifest), devtools_(devtools) {}
std::unique_ptr<ContextMenuSession> ContextMenuServiceImpl::CreateSession(
    content::WebContents* contents) {
  return CreateContextMenuSession(contents, manifest_->GetWeakPtr(),
                                  devtools_->GetWeakPtr());
}
}  // namespace tuffite
