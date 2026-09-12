// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/services/utility/utility_process_launcher.h"

#include "content/public/browser/service_process_host.h"
namespace tuffite {
namespace {
class ChromiumUtilityProcessLauncher final : public UtilityProcessLauncher {
 public:
  mojo::Remote<mojom::UtilityService> Launch(std::string_view name) override {
    return content::ServiceProcessHost::Launch<mojom::UtilityService>(
        content::ServiceProcessHost::Options()
            .WithDisplayName("Tuffite " + std::string(name))
            .Pass());
  }
};
}  // namespace
std::unique_ptr<UtilityProcessLauncher> CreateUtilityProcessLauncher() {
  return std::make_unique<ChromiumUtilityProcessLauncher>();
}
}  // namespace tuffite
