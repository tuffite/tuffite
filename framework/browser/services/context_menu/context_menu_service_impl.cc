// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/context_menu/context_menu_service_impl.h"

#include "tuffite/framework/app/services/policy/application_policy_service.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_session.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"

namespace tuffite {
ContextMenuServiceImpl::ContextMenuServiceImpl(ApplicationPolicyService& policy,
                                               DevToolsService& devtools)
    : policy_(policy), devtools_(devtools) {}
std::unique_ptr<ContextMenuSession> ContextMenuServiceImpl::CreateSession(
    content::WebContents* contents) {
  return CreateContextMenuSession(contents, *policy_, *devtools_);
}
}  // namespace tuffite
