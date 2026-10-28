// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_IMPL_H_

#include "tuffite/framework/browser/services/webauthn/web_authentication_service.h"

namespace views {
class Widget;
}
class PrefService;

namespace tuffite {

// Explorer is an inner WebContents. System prompts must belong to its outer
// application window, rather than to the offscreen child native view.
views::Widget* WebAuthnWindow(content::WebContents* contents);

class WebAuthenticationServiceImpl final : public WebAuthenticationService {
 public:
  explicit WebAuthenticationServiceImpl(content::BrowserContext* context);
  ~WebAuthenticationServiceImpl() override;
#if BUILDFLAG(IS_MAC)
  std::optional<TouchIdAuthenticatorConfig> GetTouchIdAuthenticatorConfig(
      content::BrowserContext* context) override;
#endif
  bool IsSecurityLevelAcceptable(content::RenderFrameHost* frame) override;
  std::unique_ptr<content::AuthenticatorRequestClientDelegate> CreateRequest(
      content::RenderFrameHost* frame) override;
  bool SupportsResidentKeys(content::RenderFrameHost* frame) override;
  bool IsFocused(content::WebContents* contents) override;
  void IsUserVerifyingPlatformAuthenticatorAvailableOverride(
      content::RenderFrameHost* frame,
      base::OnceCallback<void(std::optional<bool>)> callback) override;

 private:
#if BUILDFLAG(IS_MAC)
  std::unique_ptr<PrefService> preferences_;
#endif
};

#if BUILDFLAG(IS_MAC)
// Bare development executables have no Info.plist Bluetooth usage declaration.
// Do not invoke CoreBluetooth in them: macOS terminates such processes.
bool WebAuthnBluetoothAllowed();
std::optional<std::string> WebAuthnKeychainAccessGroup();
#endif

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_IMPL_H_
