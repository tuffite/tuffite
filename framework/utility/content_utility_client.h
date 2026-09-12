// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_UTILITY_CONTENT_UTILITY_CLIENT_H_
#define TUFFITE_FRAMEWORK_UTILITY_CONTENT_UTILITY_CLIENT_H_

#include "content/public/utility/content_utility_client.h"
#include "tuffite/framework/utility/utility_service.h"

namespace tuffite {

class ContentUtilityClient final : public content::ContentUtilityClient {
 public:
  explicit ContentUtilityClient(const tuffite_application& application);
  ~ContentUtilityClient() override;

  void RegisterMainThreadServices(mojo::ServiceFactory& services) override;

 private:
  UtilityApplicationContext application_context_;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_UTILITY_CONTENT_UTILITY_CLIENT_H_
