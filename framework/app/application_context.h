// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_APPLICATION_CONTEXT_H_
#define TUFFITE_FRAMEWORK_APP_APPLICATION_CONTEXT_H_

#include <memory>
#include <type_traits>

#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "components/keyed_service/core/keyed_service.h"

namespace tuffite {

class AbiHostBridge;
class ManifestService;
class ManifestServiceFactory;

namespace internal {
template <typename Service>
const void* ApplicationServiceTypeKey() {
  static const int key = 0;
  return &key;
}
}  // namespace internal

class ApplicationContext final {
 public:
  ApplicationContext(std::unique_ptr<ManifestService>, AbiHostBridge&);
  ~ApplicationContext();
  ApplicationContext(const ApplicationContext&) = delete;
  ApplicationContext& operator=(const ApplicationContext&) = delete;

  template <typename Service>
  Service* GetService(bool create = true) {
    static_assert(std::is_base_of_v<KeyedService, Service>);
    return static_cast<Service*>(GetServiceByType(
        internal::ApplicationServiceTypeKey<Service>(), create));
  }

  AbiHostBridge& bridge() const { return *bridge_; }
  base::WeakPtr<ApplicationContext> GetWeakPtr() {
    return weak_factory_.GetWeakPtr();
  }

 private:
  KeyedService* GetServiceByType(const void* service_type, bool create);

  friend class ManifestServiceFactory;
  // Consumed once when the service graph eagerly creates ManifestService.
  std::unique_ptr<ManifestService> pending_manifest_;
  const raw_ref<AbiHostBridge> bridge_;
  base::WeakPtrFactory<ApplicationContext> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_APPLICATION_CONTEXT_H_
