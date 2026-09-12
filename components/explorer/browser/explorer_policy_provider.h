// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_POLICY_PROVIDER_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_POLICY_PROVIDER_H_
#include <string>
#include <string_view>

#include "tuffite/components/explorer/browser/explorer_policy.h"
namespace content {
class RenderFrameHost;
class WebContents;
}  // namespace content
class GURL;
namespace url {
class Origin;
}
namespace tuffite {
class ExplorerPolicyProvider {
 public:
  virtual ~ExplorerPolicyProvider() = default;
  virtual bool IsSurfaceAllowed(content::RenderFrameHost&) const = 0;
  virtual bool IsOwnerAllowed(content::RenderFrameHost&) const = 0;
  virtual bool IsApp(const content::WebContents*) const = 0;
  virtual bool Resolve(content::RenderFrameHost&,
                       const std::string&,
                       ExplorerPolicy*) const = 0;
  virtual bool NavigationAllowed(content::RenderFrameHost&,
                                 const GURL&) const = 0;
  virtual bool ExecuteJavaScriptAllowed(content::RenderFrameHost&,
                                        const url::Origin&,
                                        const ExplorerPolicy&) const = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_POLICY_PROVIDER_H_
