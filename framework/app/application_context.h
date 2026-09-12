// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_APPLICATION_CONTEXT_H_
#define TUFFITE_FRAMEWORK_APP_APPLICATION_CONTEXT_H_

#include <type_traits>

#include "base/memory/raw_ref.h"
#include "components/keyed_service/core/keyed_service.h"

namespace tuffite {

class AbiHostBridge;
class AppManifest;

namespace internal {
template <typename Service>
const void* ApplicationServiceTypeKey() {
  static const int key = 0;
  return &key;
}
}  // namespace internal

class ApplicationContext final {
 public:
  ApplicationContext(const AppManifest&, AbiHostBridge&);
  ~ApplicationContext();
  ApplicationContext(const ApplicationContext&) = delete;
  ApplicationContext& operator=(const ApplicationContext&) = delete;

  template <typename Service>
  Service* GetService(bool create = true) {
    static_assert(std::is_base_of_v<KeyedService, Service>);
    return static_cast<Service*>(GetServiceByType(
        internal::ApplicationServiceTypeKey<Service>(), create));
  }

  const AppManifest& manifest() const { return *manifest_; }
  AbiHostBridge& bridge() const { return *bridge_; }

 private:
  KeyedService* GetServiceByType(const void* service_type, bool create);

  const raw_ref<const AppManifest> manifest_;
  const raw_ref<AbiHostBridge> bridge_;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_APPLICATION_CONTEXT_H_
