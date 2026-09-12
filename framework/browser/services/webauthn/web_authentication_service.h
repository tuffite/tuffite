// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_H_

#include <memory>

#include "components/keyed_service/core/keyed_service.h"
#include "content/public/browser/authenticator_request_client_delegate.h"
#include "content/public/browser/web_authentication_delegate.h"

namespace content {
class AuthenticatorRequestClientDelegate;
}

namespace tuffite {

// Profile policy and platform capability discovery belong to the service;
// request delegates retain Content's per-request lifetime.
class WebAuthenticationService : public KeyedService,
                                 public content::WebAuthenticationDelegate {
 public:
  virtual bool IsSecurityLevelAcceptable(content::RenderFrameHost* frame) = 0;
  virtual std::unique_ptr<content::AuthenticatorRequestClientDelegate>
  CreateRequest(content::RenderFrameHost* frame) = 0;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_WEB_AUTHENTICATION_SERVICE_H_
