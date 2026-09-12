// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_DEPENDENCY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_DEPENDENCY_H_

#include "components/keyed_service/core/keyed_service_factory.h"

namespace tuffite {
class ApplicationContext;
class ApplicationDependencyManager;

// Private type-erased adapter shared by the manager implementation and the
// public typed factory template. Manager operations stay out of public headers.
class ApplicationDependency : public KeyedServiceFactory {
 protected:
  ApplicationDependency(const char* name, const void* service_type);
  ~ApplicationDependency() override;

 private:
  friend class ApplicationDependencyManager;
  KeyedService* GetForApplication(ApplicationContext* context, bool create);
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_APPLICATION_DEPENDENCY_H_
