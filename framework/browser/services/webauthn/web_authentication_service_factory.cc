// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/webauthn/web_authentication_service_factory.h"

#include <utility>

#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service_impl.h"

namespace tuffite {
namespace {
// Content requests one process-level delegate without a BrowserContext
// argument. This stateless adapter routes frame/document callbacks to the
// owning profile.
class WebAuthenticationDelegate final
    : public content::WebAuthenticationDelegate {
 public:
  bool SupportsResidentKeys(content::RenderFrameHost* frame) override {
    return WebAuthenticationServiceFactory::GetForBrowserContext(
               frame->GetBrowserContext())
        ->SupportsResidentKeys(frame);
  }
  bool IsFocused(content::WebContents* contents) override {
    return contents && WebAuthenticationServiceFactory::GetForBrowserContext(
                           contents->GetBrowserContext())
                           ->IsFocused(contents);
  }
  void IsUserVerifyingPlatformAuthenticatorAvailableOverride(
      content::RenderFrameHost* frame,
      base::OnceCallback<void(std::optional<bool>)> callback) override {
    WebAuthenticationServiceFactory::GetForBrowserContext(
        frame->GetBrowserContext())
        ->IsUserVerifyingPlatformAuthenticatorAvailableOverride(
            frame, std::move(callback));
  }
};
}  // namespace
WebAuthenticationServiceFactory* WebAuthenticationServiceFactory::Get() {
  static base::NoDestructor<WebAuthenticationServiceFactory> instance;
  return instance.get();
}
WebAuthenticationServiceFactory::WebAuthenticationServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteWebAuthn",
          BrowserContextDependencyManager::GetInstance()) {}
WebAuthenticationService* WebAuthenticationServiceFactory::GetForBrowserContext(
    content::BrowserContext* context) {
  return static_cast<WebAuthenticationService*>(
      Get()->GetServiceForBrowserContext(context, true));
}
content::WebAuthenticationDelegate*
WebAuthenticationServiceFactory::GetDelegate() {
  static base::NoDestructor<WebAuthenticationDelegate> delegate;
  return delegate.get();
}
std::unique_ptr<KeyedService>
WebAuthenticationServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<WebAuthenticationServiceImpl>();
}
}  // namespace tuffite
