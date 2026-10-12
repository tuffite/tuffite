// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SESSION_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SESSION_H_

#include "base/memory/weak_ptr.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service.h"
namespace tuffite {
class ApplicationPolicyService;
class DevToolsService;
std::unique_ptr<ContextMenuSession> CreateContextMenuSession(
    content::WebContents* contents,
    ApplicationPolicyService& policy,
    base::WeakPtr<DevToolsService> devtools);
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SESSION_H_
