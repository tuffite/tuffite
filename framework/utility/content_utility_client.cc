// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/utility/content_utility_client.h"

#include "mojo/public/cpp/bindings/service_factory.h"
#include "tuffite/framework/utility/utility_service.h"

namespace tuffite {

ContentUtilityClient::ContentUtilityClient(
    const tuffite_application& application)
    : application_context_(application) {}
ContentUtilityClient::~ContentUtilityClient() = default;

void ContentUtilityClient::RegisterMainThreadServices(
    mojo::ServiceFactory& services) {
  services.Add(RunUtilityService);
}

}  // namespace tuffite
