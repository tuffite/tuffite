// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/webauthn/web_authentication_service_impl.h"

#include "base/functional/bind.h"
#include "content/public/browser/navigation_controller.h"
#include "content/public/browser/navigation_entry.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/ssl_status.h"
#include "net/cert/cert_status_flags.h"
#include "tuffite/framework/browser/services/webauthn/request_delegate.h"
#include "ui/views/widget/widget.h"

#if BUILDFLAG(IS_MAC)
#include "base/base64.h"
#include "base/task/thread_pool.h"
#include "components/prefs/pref_registry_simple.h"
#include "components/prefs/pref_service.h"
#include "components/prefs/pref_service_factory.h"
#include "content/public/browser/browser_context.h"
#include "device/fido/mac/credential_metadata.h"
#include "device/fido/mac/icloud_keychain.h"
#endif

namespace tuffite {

WebAuthenticationServiceImpl::WebAuthenticationServiceImpl(
    content::BrowserContext* context) {
#if BUILDFLAG(IS_MAC)
  auto registry = base::MakeRefCounted<PrefRegistrySimple>();
  registry->RegisterStringPref("touch_id_metadata_secret", "");
  PrefServiceFactory factory;
  if (!context->IsOffTheRecord()) {
    factory.SetUserPrefsFile(
        context->GetPath().AppendASCII("WebAuthn Local State"),
        base::ThreadPool::CreateSequencedTaskRunner(
            {base::MayBlock(), base::TaskShutdownBehavior::BLOCK_SHUTDOWN})
            .get());
  }
  preferences_ = factory.Create(registry);
#endif
}

WebAuthenticationServiceImpl::~WebAuthenticationServiceImpl() = default;

#if BUILDFLAG(IS_MAC)
std::optional<WebAuthenticationServiceImpl::TouchIdAuthenticatorConfig>
WebAuthenticationServiceImpl::GetTouchIdAuthenticatorConfig(
    content::BrowserContext* context) {
  const auto group = WebAuthnKeychainAccessGroup();
  if (!group || context->IsOffTheRecord()) {
    return std::nullopt;
  }
  std::string secret;
  if (!base::Base64Decode(preferences_->GetString("touch_id_metadata_secret"),
                          &secret) ||
      secret.size() != 32) {
    secret = device::fido::mac::GenerateCredentialMetadataSecret();
    preferences_->SetString("touch_id_metadata_secret",
                            base::Base64Encode(secret));
    preferences_->CommitPendingWrite();
  }
  return TouchIdAuthenticatorConfig{.keychain_access_group = *group,
                                    .metadata_secret = std::move(secret)};
}
#endif

views::Widget* WebAuthnWindow(content::WebContents* contents) {
  if (!contents) {
    return nullptr;
  }
  while (contents->GetOuterWebContents()) {
    contents = contents->GetOuterWebContents();
  }
  return views::Widget::GetTopLevelWidgetForNativeView(
      contents->GetNativeView());
}

bool WebAuthenticationServiceImpl::IsSecurityLevelAcceptable(
    content::RenderFrameHost* frame) {
  auto* contents = content::WebContents::FromRenderFrameHost(frame);
  auto* entry = contents->GetController().GetLastCommittedEntry();
  return entry && !net::IsCertStatusError(entry->GetSSL().cert_status);
}

std::unique_ptr<content::AuthenticatorRequestClientDelegate>
WebAuthenticationServiceImpl::CreateRequest(content::RenderFrameHost* frame) {
  return std::make_unique<WebAuthnRequestDelegate>(frame);
}

bool WebAuthenticationServiceImpl::SupportsResidentKeys(
    content::RenderFrameHost* frame) {
  return true;
}

bool WebAuthenticationServiceImpl::IsFocused(content::WebContents* contents) {
  auto* window = WebAuthnWindow(contents);
  return window && window->IsVisible() &&
         contents->GetVisibility() == content::Visibility::VISIBLE;
}

void WebAuthenticationServiceImpl::
    IsUserVerifyingPlatformAuthenticatorAvailableOverride(
        content::RenderFrameHost* frame,
        base::OnceCallback<void(std::optional<bool>)> callback) {
  // Preserve DevTools virtual authenticator availability in native smoke tests.
  content::WebAuthenticationDelegate::
      IsUserVerifyingPlatformAuthenticatorAvailableOverride(
          frame, base::BindOnce(
                     [](base::OnceCallback<void(std::optional<bool>)> callback,
                        std::optional<bool> virtual_result) {
                       if (virtual_result.has_value()) {
                         std::move(callback).Run(virtual_result);
                         return;
                       }
#if BUILDFLAG(IS_MAC)
                       // Content also probes the configured local Touch ID
                       // authenticator when iCloud Keychain is unavailable.
                       std::move(callback).Run(
                           device::fido::icloud_keychain::IsSupported()
                               ? std::optional<bool>(true)
                               : std::nullopt);
#elif BUILDFLAG(IS_WIN)
                       // Let Content query the Windows WebAuthn API.
                       std::move(callback).Run(std::nullopt);
#else
                       std::move(callback).Run(false);
#endif
                     },
                     std::move(callback)));
}

}  // namespace tuffite
