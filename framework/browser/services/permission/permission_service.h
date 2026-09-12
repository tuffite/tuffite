// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_PERMISSION_PERMISSION_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_PERMISSION_PERMISSION_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"
#include "content/public/browser/permission_controller_delegate.h"

namespace tuffite {
class PermissionService : public KeyedService,
                          public content::PermissionControllerDelegate {};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_PERMISSION_PERMISSION_SERVICE_H_
