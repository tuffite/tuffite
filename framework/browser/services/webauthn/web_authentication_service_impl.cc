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
#include "device/fido/mac/icloud_keychain.h"
#endif

namespace tuffite {

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
                       std::move(callback).Run(
                           device::fido::icloud_keychain::IsSupported());
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
