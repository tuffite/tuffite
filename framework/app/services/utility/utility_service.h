// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_SERVICE_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_SERVICE_H_
#include <map>
#include <memory>

#include "base/unguessable_token.h"
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/framework/app/services/utility/utility_types.h"
namespace tuffite {
class UtilityProcessLauncher;
class UtilityService : public KeyedService {
 public:
  virtual void SetUtilityStartupConfigurations(
      std::map<std::string, std::vector<uint8_t>> configurations) {}
  virtual void SetUtilityModuleResolver(UtilityModuleResolver resolver) {}
  virtual void SetUtilityRegistrationChecker(
      UtilityRegistrationChecker checker) = 0;
  virtual bool IsRegisteredUtility(std::string_view name) const = 0;
  virtual void SetUtilityInterfaceResolver(
      UtilityInterfaceResolver resolver) = 0;
  virtual std::string ResolveUtilityInterface(std::string_view name) const = 0;
  virtual void InvokeUtilityService(const base::UnguessableToken& owner,
                                    uint64_t request_id,
                                    std::string name,
                                    std::vector<uint8_t> request,
                                    UtilityResponseCallback callback) = 0;
  virtual void CancelUtilityService(const base::UnguessableToken& owner,
                                    std::string name,
                                    uint64_t request_id) = 0;
  virtual void CancelUtilityOwner(const base::UnguessableToken& owner) = 0;
  virtual void CreateUtilitySession(
      std::string name,
      uint64_t session_id,
      std::vector<uint8_t> init,
      uint32_t initial_event_credits,
      uint32_t max_event_bytes,
      UtilitySessionEventCallback event_callback,
      UtilitySessionClosedCallback closed_callback,
      base::OnceCallback<void(int32_t)> callback) = 0;
  virtual void SendUtilitySession(std::string_view name,
                                  uint64_t session_id,
                                  uint64_t command_id,
                                  std::vector<uint8_t> command,
                                  UtilityResponseCallback callback) = 0;
  virtual void AcknowledgeUtilitySession(std::string_view name,
                                         uint64_t session_id,
                                         uint64_t committed_sequence,
                                         uint32_t credits) = 0;
  virtual void CancelUtilitySessionCommand(std::string_view name,
                                           uint64_t session_id,
                                           uint64_t command_id) = 0;
  virtual void CloseUtilitySession(std::string_view name,
                                   uint64_t session_id,
                                   uint32_t reason,
                                   UtilityResponseCallback callback) = 0;
};
std::unique_ptr<UtilityService> CreateUtilityService(
    std::unique_ptr<UtilityProcessLauncher>);
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_UTILITY_UTILITY_SERVICE_H_
