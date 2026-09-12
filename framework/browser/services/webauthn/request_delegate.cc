// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/webauthn/request_delegate.h"

#include <array>
#include <utility>

#include "base/functional/bind.h"
#include "base/strings/utf_string_conversions.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/storage_partition.h"
#include "content/public/browser/web_contents.h"
#include "crypto/random.h"
#include "device/fido/cable/v2_handshake.h"
#include "device/fido/fido_authenticator.h"
#include "device/fido/fido_discovery_factory.h"
#include "device/fido/pin.h"
#include "tuffite/framework/browser/services/webauthn/request_dialog.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service_impl.h"
#include "ui/views/widget/widget.h"

#if BUILDFLAG(IS_WIN)
#include "device/fido/win/webauthn_api.h"
#endif

namespace tuffite {

WebAuthnRequestDelegate::WebAuthnRequestDelegate(
    content::RenderFrameHost* frame)
    : contents_(
          content::WebContents::FromRenderFrameHost(frame)->GetWeakPtr()) {}

WebAuthnRequestDelegate::~WebAuthnRequestDelegate() {
  CloseDialog();
}

void WebAuthnRequestDelegate::SetRelyingPartyId(const std::string& rp_id) {
  rp_id_ = rp_id;
}

void WebAuthnRequestDelegate::SetUIPresentation(UIPresentation presentation) {
  presentation_ = presentation;
}

bool WebAuthnRequestDelegate::DoesBlockRequestOnFailure(
    InterestingFailureReason reason) {
  if (IsVirtualEnvironmentEnabled() ||
      reason != InterestingFailureReason::kNoPasskeys || !cancel_ ||
      presentation_ == UIPresentation::kDisabled ||
      presentation_ == UIPresentation::kPasskeyUpgrade) {
    return false;
  }
  if (!CreateRequestDialog(
          u"No passkey is available with the configured authenticators. "
          u"Cancel and choose another sign-in method on the website.")) {
    return false;
  }
  // Keep the error visible until the user dismisses it. AbortSignal and
  // WebContents teardown also terminate this request and close its dialog.
  ShowDialog();
  return true;
}

void WebAuthnRequestDelegate::RegisterActionCallbacks(
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
        bluetooth_permission) {
  cancel_ = std::move(cancel);
  immediate_not_found_ = std::move(immediate_not_found);
  request_ = std::move(request);
  cancel_ui_timeout_ = std::move(cancel_ui_timeout);
  bluetooth_power_on_ = std::move(bluetooth_power_on);
  bluetooth_permission_ = std::move(bluetooth_permission);
}

void WebAuthnRequestDelegate::ConfigureDiscoveries(
    const url::Origin& origin,
    const std::string& rp_id,
    RequestSource source,
    device::FidoRequestType type,
    std::optional<device::ResidentKeyRequirement> resident_key_requirement,
    device::UserVerificationRequirement user_verification,
    bool cmtg_requested,
    std::optional<std::string_view> user_name,
    bool enclave_available,
    device::FidoDiscoveryFactory* factory) {
  if (!factory) {
    // Immediate mediation has no discovery factory or credential store in this
    // embedder. Resolve its explicit no-match callback instead of hanging.
    if (immediate_not_found_) {
      std::move(immediate_not_found_).Run();
    } else {
      Cancel();
    }
    return;
  }
  if (IsVirtualEnvironmentEnabled()) {
    return;
  }
  auto* window = WebAuthnWindow(contents_.get());
#if BUILDFLAG(IS_MAC)
  if (window) {
    factory->set_nswindow(window->GetNativeWindow());
  }
  if (!WebAuthnBluetoothAllowed()) {
    return;
  }
#endif
  if (!window || presentation_ == UIPresentation::kDisabled ||
      presentation_ == UIPresentation::kPasskeyUpgrade) {
    return;
  }
#if BUILDFLAG(IS_WIN)
  auto* api = device::WinWebAuthnApi::GetDefault();
  if (api && api->SupportsHybrid()) {
    // Modern Windows owns phone QR UI as well as Hello and USB keys. Its
    // discovery factory deliberately does not create a second hybrid device.
    return;
  }
#endif
  std::array<uint8_t, device::cablev2::kQRKeySize> key;
  crypto::RandBytes(key);
  qr_ = device::cablev2::qr::Encode(key, type);
  factory->set_cable_data(type, key);
  factory->set_cable_event_callback(base::BindRepeating(
      [](base::WeakPtr<WebAuthnRequestDelegate> delegate,
         device::cablev2::Event event) {
        if (!delegate || delegate->selected_transport_ != Transport::kHybrid ||
            event != device::cablev2::Event::kReady) {
          return;
        }
        delegate->CreateRequestDialog(
            u"Connected. Complete verification on your phone.");
        delegate->ShowDialog();
      },
      weak_factory_.GetWeakPtr()));
  // Preserve the requesting Explorer's network partition and lifetime.
  factory->set_network_context_factory(base::BindRepeating(
      [](base::WeakPtr<content::WebContents> contents)
          -> network::mojom::NetworkContext* {
        return contents ? contents->GetPrimaryMainFrame()
                              ->GetStoragePartition()
                              ->GetNetworkContext()
                        : nullptr;
      },
      contents_));
}

void WebAuthnRequestDelegate::StartObserving(
    device::FidoRequestHandlerBase* handler) {
  observation_.Observe(handler);
}

void WebAuthnRequestDelegate::StopObserving(
    device::FidoRequestHandlerBase* handler) {
  observation_.Reset();
}

void WebAuthnRequestDelegate::OnTransportAvailabilityEnumerated(
    device::FidoRequestHandlerBase::TransportAvailabilityInfo data) {
  availability_ = std::move(data);
  if (IsVirtualEnvironmentEnabled()) {
    return;
  }
  // Conditional creation upgrades are not offered without a password manager.
  if (presentation_ == UIPresentation::kDisabled ||
      presentation_ == UIPresentation::kPasskeyUpgrade ||
      !WebAuthnWindow(contents_.get())) {
    Cancel();
    return;
  }
  ShowChoices();
}

bool WebAuthnRequestDelegate::EmbedderControlsAuthenticatorDispatch(
    const device::FidoAuthenticator& authenticator) {
  return !IsVirtualEnvironmentEnabled();
}

void WebAuthnRequestDelegate::FidoAuthenticatorAdded(
    const device::FidoAuthenticator& authenticator) {
  authenticators_[authenticator.GetId()] =
      authenticator.AuthenticatorTransport();
  if (selected_transport_ &&
      (authenticator.AuthenticatorTransport() == selected_transport_ ||
       (!authenticator.AuthenticatorTransport() &&
        selected_transport_ == Transport::kInternal))) {
    DispatchAuthenticator(authenticator.GetId());
  }
}

void WebAuthnRequestDelegate::FidoAuthenticatorRemoved(std::string_view id) {
  auto entry = authenticators_.find(id);
  if (entry != authenticators_.end()) {
    authenticators_.erase(entry);
  }
}

void WebAuthnRequestDelegate::Cancel() {
  CloseDialog();
  if (cancel_) {
    std::move(cancel_).Run();  // This destroys the delegate.
  }
}

WebAuthnRequestDialog* WebAuthnRequestDelegate::CreateRequestDialog(
    const std::u16string& message) {
  CloseDialog();
  auto* window = WebAuthnWindow(contents_.get());
  if (!window) {
    return nullptr;
  }
  auto dialog = std::make_unique<WebAuthnRequestDialog>(
      rp_id_,
      base::BindRepeating(&WebAuthnRequestDelegate::Cancel,
                          weak_factory_.GetWeakPtr()),
      presentation_ == UIPresentation::kAutofill ||
          presentation_ == UIPresentation::kAmbient);
  auto* result = dialog.get();
  result->AddMessage(message);
  dialog_ = result->GetWeakPtr();
  pending_dialog_ = std::move(dialog);
  return result;
}

void WebAuthnRequestDelegate::ShowDialog() {
  if (pending_dialog_) {
    auto* window = WebAuthnWindow(contents_.get());
    if (!window) {
      Cancel();
      return;
    }
    WebAuthnRequestDialog::Show(std::move(pending_dialog_),
                                window->GetNativeWindow(),
                                window->GetNativeView());
  }
}

void WebAuthnRequestDelegate::CloseDialog() {
  if (dialog_) {
    dialog_->CloseWithoutCancelling();
    dialog_.reset();
  }
  pending_dialog_.reset();
}

void WebAuthnRequestDelegate::ShowChoices() {
  auto* dialog =
      CreateRequestDialog(u"Choose how to verify your identity for this site.");
  if (!dialog) {
    Cancel();
    return;
  }
  const auto add = [&](Transport transport, const std::u16string& label) {
    dialog->AddAction(
        label, base::BindRepeating(&WebAuthnRequestDelegate::ChooseTransport,
                                   weak_factory_.GetWeakPtr(), transport));
  };
  bool mechanism = false;
  if (availability_->has_win_native_api_authenticator ||
      availability_->has_icloud_keychain) {
    add(Transport::kInternal, u"Use system passkeys or a security key");
    mechanism = true;
  }
  if (!availability_->has_win_native_api_authenticator &&
      availability_->available_transports.contains(
          Transport::kUsbHumanInterfaceDevice)) {
    add(Transport::kUsbHumanInterfaceDevice, u"Use a USB security key");
    mechanism = true;
  }
  if (!qr_.empty() &&
      availability_->available_transports.contains(Transport::kHybrid)) {
    add(Transport::kHybrid, u"Use a phone or tablet");
    mechanism = true;
  }
  if (!mechanism) {
    // Keep a dismissible explanation visible. Cancellation always resolves the
    // page's pending WebAuthn request, including in bare development launches.
    dialog->AddMessage(
        u"No supported authenticator is available. Try another sign-in method "
        u"on the website.");
  }
#if BUILDFLAG(IS_MAC)
  if (!availability_->has_icloud_keychain) {
    dialog->AddMessage(
        u"This application cannot access iCloud passkeys. Use a security key "
        u"or a phone instead.");
  }
  if (!WebAuthnBluetoothAllowed()) {
    dialog->AddMessage(
        u"Phone sign-in requires a packaged application with Bluetooth "
        u"permission information, launched from Finder.");
  }
#endif
  ShowDialog();
}

void WebAuthnRequestDelegate::ChooseTransport(Transport transport) {
  auto weak = weak_factory_.GetWeakPtr();
  selected_transport_ = transport;
  if (cancel_ui_timeout_) {
    std::move(cancel_ui_timeout_).Run();
  }
  if (transport == Transport::kHybrid) {
    ShowPhone();
  } else if (transport == Transport::kInternal) {
    CloseDialog();
  } else {
    CreateRequestDialog(
        u"Connect your security key and touch it when its light flashes.");
    ShowDialog();
  }
  if (!weak) {
    return;
  }
  // Callback dispatch can synchronously remove an authenticator or destroy us.
  std::vector<std::string> ids;
  for (const auto& [id, candidate] : authenticators_) {
    if (candidate == transport ||
        (!candidate && transport == Transport::kInternal)) {
      ids.push_back(id);
    }
  }
  for (const auto& id : ids) {
    if (!weak) {
      return;
    }
    weak->DispatchAuthenticator(id);
  }
}

void WebAuthnRequestDelegate::DispatchAuthenticator(const std::string& id) {
  if (request_) {
    request_.Run(id);
  }
}

void WebAuthnRequestDelegate::ShowPhone() {
  using BleStatus = device::FidoRequestHandlerBase::BleStatus;
  auto* dialog = CreateRequestDialog(
      u"Use a nearby phone or tablet containing a passkey for this site.");
  if (!dialog) {
    Cancel();
    return;
  }
  if (availability_->ble_status == BleStatus::kPendingPermissionRequest) {
    dialog->AddAction(
        u"Allow Bluetooth",
        base::BindRepeating(
            [](base::WeakPtr<WebAuthnRequestDelegate> delegate) {
              if (delegate && delegate->bluetooth_permission_) {
                delegate->bluetooth_permission_.Run(base::BindOnce(
                    &WebAuthnRequestDelegate::BluetoothAdapterStatusChanged,
                    delegate));
              }
            },
            weak_factory_.GetWeakPtr()));
  } else if (availability_->ble_status == BleStatus::kOff) {
    dialog->AddMessage(
        u"Turn on Bluetooth to verify that your phone is nearby.");
    if (availability_->can_power_on_ble_adapter) {
      dialog->AddAction(u"Turn on Bluetooth", bluetooth_power_on_);
    }
  } else if (availability_->ble_status == BleStatus::kPermissionDenied) {
    dialog->AddMessage(
        u"Bluetooth permission was denied. Enable it in system settings, or "
        u"cancel and choose a security key.");
  } else if (!dialog->AddQRCode(qr_)) {
    dialog->AddMessage(
        u"Unable to generate a QR code. Cancel and choose another sign-in "
        u"method.");
  } else {
    dialog->AddMessage(
        u"Scan this QR code with your phone camera, then follow the "
        u"instructions on your phone.");
  }
  ShowDialog();
}

void WebAuthnRequestDelegate::BluetoothAdapterStatusChanged(
    device::FidoRequestHandlerBase::BleStatus status) {
  if (!availability_) {
    return;
  }
  availability_->ble_status = status;
  if (selected_transport_ == Transport::kHybrid) {
    ShowPhone();
  }
}

bool WebAuthnRequestDelegate::SupportsPIN() const {
  return true;
}

void WebAuthnRequestDelegate::CollectPIN(
    CollectPINOptions options,
    base::OnceCallback<void(std::u16string)> callback) {
  bool new_pin = options.reason != device::pin::PINEntryReason::kChallenge;
  auto* dialog = CreateRequestDialog(new_pin ? u"Set a new security key PIN."
                                             : u"Enter your security key PIN.");
  if (!dialog) {
    Cancel();
    return;
  }
  if (options.error != device::pin::PINEntryError::kNoError) {
    dialog->AddMessage(u"The previous verification failed. Please try again.");
  }
  if (!new_pin && options.attempts > 0) {
    dialog->AddMessage(base::UTF8ToUTF16(std::to_string(options.attempts)) +
                       u" attempts remaining.");
  }
  dialog->CollectPIN(options.min_pin_length, new_pin, std::move(callback));
  ShowDialog();
}

void WebAuthnRequestDelegate::StartBioEnrollment(
    base::OnceClosure next_callback) {
  // Enrollment changes authenticator configuration. It belongs in the key's
  // management application, rather than an arbitrary website's sign-in flow.
  Cancel();
}

void WebAuthnRequestDelegate::OnRetryUserVerification(int attempts) {
  auto* dialog =
      CreateRequestDialog(u"Verification failed. Try your security key again.");
  if (!dialog) {
    Cancel();
    return;
  }
  dialog->AddMessage(base::UTF8ToUTF16(std::to_string(attempts)) +
                     u" attempts remaining.");
  ShowDialog();
}

void WebAuthnRequestDelegate::FinishCollectToken() {
  CreateRequestDialog(u"Touch your security key to continue.");
  ShowDialog();
}

void WebAuthnRequestDelegate::SelectAccount(
    std::vector<device::AuthenticatorGetAssertionResponse> responses,
    base::OnceCallback<void(device::AuthenticatorGetAssertionResponse)>
        callback) {
  if (IsVirtualEnvironmentEnabled()) {
    content::AuthenticatorRequestClientDelegate::SelectAccount(
        std::move(responses), std::move(callback));
    return;
  }
  accounts_ = std::move(responses);
  account_callback_ = std::move(callback);
  auto* dialog =
      CreateRequestDialog(u"Choose an account to sign in to this site.");
  if (!dialog || accounts_.empty()) {
    Cancel();
    return;
  }
  for (size_t i = 0; i < accounts_.size(); ++i) {
    const auto& user = accounts_[i].user_entity;
    std::string label =
        user ? user->name.value_or(user->display_name.value_or("Account"))
             : "Account";
    dialog->AddAction(
        base::UTF8ToUTF16(label),
        base::BindRepeating(&WebAuthnRequestDelegate::ChooseAccount,
                            weak_factory_.GetWeakPtr(), i));
  }
  ShowDialog();
}

void WebAuthnRequestDelegate::ChooseAccount(size_t index) {
  if (!account_callback_ || index >= accounts_.size()) {
    return;
  }
  CloseDialog();
  std::move(account_callback_).Run(std::move(accounts_[index]));
}

}  // namespace tuffite
