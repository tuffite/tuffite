// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_REQUEST_DELEGATE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_REQUEST_DELEGATE_H_

#include <map>
#include <string>

#include "base/memory/weak_ptr.h"
#include "base/scoped_observation.h"
#include "content/public/browser/authenticator_request_client_delegate.h"

namespace content {
class RenderFrameHost;
class WebContents;
}  // namespace content

namespace tuffite {
class WebAuthnRequestDialog;

class WebAuthnRequestDelegate final
    : public content::AuthenticatorRequestClientDelegate {
 public:
  explicit WebAuthnRequestDelegate(content::RenderFrameHost* frame);
  ~WebAuthnRequestDelegate() override;
  void SetRelyingPartyId(const std::string& rp_id) override;
  void SetUIPresentation(UIPresentation presentation) override;
  bool DoesBlockRequestOnFailure(InterestingFailureReason reason) override;
  void RegisterActionCallbacks(
      base::OnceClosure cancel,
      base::OnceClosure immediate_not_found,
      base::RepeatingClosure start_over,
      AccountPreselectedCallback account_preselected,
      PasswordSelectedCallback password_selected,
      device::FidoRequestHandlerBase::RequestCallback request,
      base::OnceClosure cancel_ui_timeout,
      base::RepeatingClosure bluetooth_power_on,
      base::RepeatingCallback<
          void(device::FidoRequestHandlerBase::BlePermissionCallback)>
          bluetooth_permission) override;
  void ConfigureDiscoveries(
      const url::Origin& origin,
      const std::string& rp_id,
      RequestSource source,
      device::FidoRequestType type,
      std::optional<device::ResidentKeyRequirement> resident_key_requirement,
      device::UserVerificationRequirement user_verification,
      bool cmtg_requested,
      std::optional<std::string_view> user_name,
      bool enclave_available,
      device::FidoDiscoveryFactory* factory) override;
  void SelectAccount(
      std::vector<device::AuthenticatorGetAssertionResponse> responses,
      base::OnceCallback<void(device::AuthenticatorGetAssertionResponse)>
          callback) override;
  void StartObserving(device::FidoRequestHandlerBase* handler) override;
  void StopObserving(device::FidoRequestHandlerBase* handler) override;
  void OnTransportAvailabilityEnumerated(
      device::FidoRequestHandlerBase::TransportAvailabilityInfo data) override;
  bool EmbedderControlsAuthenticatorDispatch(
      const device::FidoAuthenticator& authenticator) override;
  void FidoAuthenticatorAdded(
      const device::FidoAuthenticator& authenticator) override;
  void FidoAuthenticatorRemoved(std::string_view id) override;
  bool SupportsPIN() const override;
  void CollectPIN(CollectPINOptions options,
                  base::OnceCallback<void(std::u16string)> callback) override;
  void StartBioEnrollment(base::OnceClosure next_callback) override;
  void OnRetryUserVerification(int attempts) override;
  void FinishCollectToken() override;
  void BluetoothAdapterStatusChanged(
      device::FidoRequestHandlerBase::BleStatus status) override;

 private:
  using Transport = device::FidoTransportProtocol;
  void Cancel();
  WebAuthnRequestDialog* CreateRequestDialog(const std::u16string& message);
  void ShowDialog();
  void CloseDialog();
  void ShowChoices();
  void ChooseTransport(Transport transport);
  void ShowPhone();
  void DispatchAuthenticator(const std::string& id);
  void ChooseAccount(size_t index);
  base::WeakPtr<content::WebContents> contents_;
  std::string rp_id_;
  std::string qr_;
  UIPresentation presentation_ = UIPresentation::kModal;
  std::optional<Transport> selected_transport_;
  std::optional<device::FidoRequestHandlerBase::TransportAvailabilityInfo>
      availability_;
  std::map<std::string, std::optional<Transport>, std::less<>> authenticators_;
  std::vector<device::AuthenticatorGetAssertionResponse> accounts_;
  base::OnceCallback<void(device::AuthenticatorGetAssertionResponse)>
      account_callback_;
  base::OnceClosure cancel_;
  base::OnceClosure immediate_not_found_;
  base::OnceClosure cancel_ui_timeout_;
  device::FidoRequestHandlerBase::RequestCallback request_;
  base::RepeatingClosure bluetooth_power_on_;
  base::RepeatingCallback<void(
      device::FidoRequestHandlerBase::BlePermissionCallback)>
      bluetooth_permission_;
  std::unique_ptr<WebAuthnRequestDialog> pending_dialog_;
  base::WeakPtr<WebAuthnRequestDialog> dialog_;
  base::ScopedObservation<device::FidoRequestHandlerBase,
                          device::FidoRequestHandlerBase::Observer>
      observation_{this};
  base::WeakPtrFactory<WebAuthnRequestDelegate> weak_factory_{this};
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_REQUEST_DELEGATE_H_
