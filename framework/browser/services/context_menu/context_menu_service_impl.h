// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SERVICE_IMPL_H_

#include "base/memory/raw_ref.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service.h"

namespace tuffite {
class ApplicationPolicyService;
class DevToolsService;
class ContextMenuServiceImpl final : public ContextMenuService {
 public:
  ContextMenuServiceImpl(ApplicationPolicyService& policy,
                         DevToolsService& devtools);
  std::unique_ptr<ContextMenuSession> CreateSession(
      content::WebContents* contents) override;

 private:
  const raw_ref<ApplicationPolicyService> policy_;
  const raw_ref<DevToolsService> devtools_;
};
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_CONTEXT_MENU_CONTEXT_MENU_SERVICE_IMPL_H_
