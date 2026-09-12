// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_FACTORY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_FACTORY_H_

#include "base/no_destructor.h"
#include "tuffite/framework/app/services/application_keyed_service_factory.h"
#include "tuffite/framework/app/services/remoting/remoting_service.h"

namespace tuffite::remoting {

class RemotingServiceFactory final
    : public ApplicationKeyedServiceFactory<RemotingService> {
 public:
  static RemotingServiceFactory* Get();

 private:
  friend class base::NoDestructor<RemotingServiceFactory>;
  RemotingServiceFactory();
  std::unique_ptr<RemotingService> BuildServiceInstanceForApplication(
      ApplicationContext& context) const override;
};

}  // namespace tuffite::remoting

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_FACTORY_H_
