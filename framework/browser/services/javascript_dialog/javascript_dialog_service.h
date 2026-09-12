// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_JAVASCRIPT_DIALOG_JAVASCRIPT_DIALOG_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_JAVASCRIPT_DIALOG_JAVASCRIPT_DIALOG_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"
#include "content/public/browser/javascript_dialog_manager.h"

namespace tuffite {
class JavaScriptDialogService : public KeyedService,
                                public content::JavaScriptDialogManager {};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_JAVASCRIPT_DIALOG_JAVASCRIPT_DIALOG_SERVICE_H_
