// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_ISOLATED_BROWSER_ISOLATED_SOURCE_PROVIDER_H_
#define TUFFITE_COMPONENTS_ISOLATED_BROWSER_ISOLATED_SOURCE_PROVIDER_H_
#include <optional>
#include <string>

namespace content {
class RenderFrameHost;
}

namespace tuffite {
// The embedder owns document authorization and immutable source lookup.
class IsolatedSourceProvider {
 public:
  virtual ~IsolatedSourceProvider() = default;
  virtual std::optional<std::string> LoadForFrame(
      content::RenderFrameHost& frame,
      const std::string& name) = 0;
  virtual void Release(const std::string& entry_url) = 0;
};
}  // namespace tuffite
#endif
