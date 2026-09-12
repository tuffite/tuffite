// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_CLIENT_HINTS_CLIENT_HINTS_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_CLIENT_HINTS_CLIENT_HINTS_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"
#include "content/public/browser/client_hints_controller_delegate.h"

namespace tuffite {
class ClientHintsService : public KeyedService {
 public:
  virtual content::ClientHintsControllerDelegate* delegate() = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_CLIENT_HINTS_CLIENT_HINTS_SERVICE_H_
