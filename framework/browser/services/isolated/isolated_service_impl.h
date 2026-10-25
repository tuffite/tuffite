// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_ISOLATED_ISOLATED_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_ISOLATED_ISOLATED_SERVICE_IMPL_H_
#include <memory>

#include "tuffite/framework/browser/services/isolated/isolated_registry.h"
#include "tuffite/framework/browser/services/isolated/isolated_service.h"

namespace tuffite {
class ManifestService;
class IsolatedServiceImpl final : public IsolatedService {
 public:
  explicit IsolatedServiceImpl(const ManifestService& manifest);
  ~IsolatedServiceImpl() override;
  std::optional<std::string> SourceForFrame(
      content::RenderFrameHost& frame,
      const std::string& name) const override;
  std::optional<std::string> Source(std::string_view name,
                                    const url::Origin& origin) const override;
  std::optional<std::string> Apply(const base::DictValue& request) override;
  base::WeakPtr<IsolatedService> GetWeakPtr() override;
  void Shutdown() override;

 private:
  std::unique_ptr<IsolatedRegistry> registry_;
  base::WeakPtrFactory<IsolatedServiceImpl> weak_factory_{this};
};
}  // namespace tuffite
#endif
