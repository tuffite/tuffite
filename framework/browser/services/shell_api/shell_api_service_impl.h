// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_IMPL_H_
#include <map>

#include "base/memory/raw_ref.h"
#include "tuffite/components/dispatch/browser/host_dispatch.h"
#include "tuffite/components/dispatch/browser/request_registry.h"
#include "tuffite/components/dispatch/browser/rpc_router.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service.h"
namespace content {
class BrowserContext;
}
namespace tuffite {
class ApplicationPolicyService;
class UtilityService;
class ShellApiServiceImpl final : public ShellApiService {
 public:
  ShellApiServiceImpl(content::BrowserContext&,
                      HostDispatch&,
                      ApplicationPolicyService&,
                      UtilityService&);
  ~ShellApiServiceImpl() override;
  base::WeakPtr<ShellApiService> GetWeakPtr() override;
  bool IsAuthorized(content::RenderFrameHost&) const override;
  bool IsEventAuthorized(content::RenderFrameHost&) const override;
  void Dispatch(content::RenderFrameHost&,
                const std::string&,
                const std::vector<uint8_t>&,
                const base::UnguessableToken&,
                Reply) override;
  void DispatchRequest(content::RenderFrameHost&,
                       uint64_t,
                       const std::string&,
                       const std::vector<uint8_t>&,
                       const base::UnguessableToken&,
                       Reply) override;
  void CancelRequest(const base::UnguessableToken&, uint64_t) override;
  void CancelDocument(const base::UnguessableToken&) override;
  void Subscribe(const base::UnguessableToken&, Event) override;
  void Broadcast(const std::string&, const std::vector<uint8_t>&) override;
  void Shutdown() override;

 private:
  rpc::RequestRegistry requests_;
  uint64_t next_legacy_id_ = 1;
  rpc::Router router_;
  const raw_ref<content::BrowserContext> browser_context_;
  const raw_ref<HostDispatch> host_;
  const raw_ref<ApplicationPolicyService> policy_;
  const raw_ref<UtilityService> utility_;
  std::map<base::UnguessableToken, base::UnguessableToken> utility_owners_;
  std::map<base::UnguessableToken, Event> subscriptions_;
  bool stopped_ = false;
  base::WeakPtrFactory<ShellApiServiceImpl> weak_factory_{this};
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_SHELL_API_SERVICE_IMPL_H_
