// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/client_hints/client_hints_service_factory.h"

#include "base/functional/bind.h"
#include "components/client_hints/browser/in_memory_client_hints_controller_delegate.h"
#include "components/embedder_support/user_agent_utils.h"
#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "content/public/browser/network_service_instance.h"
#include "services/network/public/cpp/network_quality_tracker.h"
#include "tuffite/framework/browser/services/client_hints/client_hints_service.h"

namespace tuffite {
namespace {
class ClientHintsServiceImpl final : public ClientHintsService {
 public:
  ClientHintsServiceImpl()
      : network_quality_tracker_(
            base::BindRepeating(&content::GetNetworkService)),
        delegate_(&network_quality_tracker_,
                  // Explorer currently enables JavaScript for all documents.
                  base::BindRepeating([](const GURL&) { return true; }),
                  embedder_support::GetUserAgentMetadata()) {}

  content::ClientHintsControllerDelegate* delegate() override {
    return &delegate_;
  }

 private:
  // The tracker must outlive the delegate that borrows it.
  network::NetworkQualityTracker network_quality_tracker_;
  client_hints::InMemoryClientHintsControllerDelegate delegate_;
};
}  // namespace

ClientHintsServiceFactory* ClientHintsServiceFactory::Get() {
  static base::NoDestructor<ClientHintsServiceFactory> instance;
  return instance.get();
}
ClientHintsServiceFactory::ClientHintsServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteClientHints",
          BrowserContextDependencyManager::GetInstance()) {}
ClientHintsService* ClientHintsServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<ClientHintsService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
ClientHintsServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<ClientHintsServiceImpl>();
}
}  // namespace tuffite
