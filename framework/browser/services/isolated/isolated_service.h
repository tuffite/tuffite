// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_ISOLATED_ISOLATED_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_ISOLATED_ISOLATED_SERVICE_H_
#include <optional>
#include <string>
#include <string_view>

#include "base/memory/weak_ptr.h"
#include "base/values.h"
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/components/isolated/browser/isolated_source_provider.h"
#include "tuffite/framework/common/resource.h"
#include "url/origin.h"

namespace tuffite {
class IsolatedService : public KeyedService, public IsolatedSourceProvider {
 public:
  virtual std::optional<std::string> Source(
      std::string_view name,
      const url::Origin& origin) const = 0;
  virtual std::optional<MemoryAsset> ResourceForFrame(
      content::RenderFrameHost& frame,
      const GURL& url) const = 0;
  virtual std::optional<std::string> Apply(const base::DictValue& request) = 0;
  virtual base::WeakPtr<IsolatedService> GetWeakPtr() = 0;
};
}  // namespace tuffite
#endif
