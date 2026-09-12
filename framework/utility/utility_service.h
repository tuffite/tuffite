// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_UTILITY_UTILITY_SERVICE_H_
#define TUFFITE_FRAMEWORK_UTILITY_UTILITY_SERVICE_H_

#include <memory>

#include "mojo/public/cpp/bindings/pending_receiver.h"
#include "tuffite/framework/common/utility/utility_service.mojom.h"
#include "tuffite/framework/public/abi.h"

namespace tuffite {

// Scopes the immutable host application callback table to the lifetime of the
// Chromium utility client. ServiceFactory accepts only plain factory function
// pointers, so factories copy this table at service construction instead of
// reaching through the browser-process AbiHostBridge.
class UtilityApplicationContext {
 public:
  explicit UtilityApplicationContext(const tuffite_application& application);
  UtilityApplicationContext(const UtilityApplicationContext&) = delete;
  UtilityApplicationContext& operator=(const UtilityApplicationContext&) =
      delete;
  ~UtilityApplicationContext();
};

std::unique_ptr<mojom::UtilityService> RunUtilityService(
    mojo::PendingReceiver<mojom::UtilityService> receiver);

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_UTILITY_UTILITY_SERVICE_H_
