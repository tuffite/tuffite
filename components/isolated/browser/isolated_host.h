// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_ISOLATED_BROWSER_ISOLATED_HOST_H_
#define TUFFITE_COMPONENTS_ISOLATED_BROWSER_ISOLATED_HOST_H_
#include "base/memory/weak_ptr.h"
#include "content/public/browser/document_service.h"
#include "tuffite/components/isolated/common/isolated.mojom.h"
namespace tuffite {
class IsolatedSourceProvider;
// Document-scoped Mojo adapter, analogous to DispatchHost/RpcHandler. Source
// authorization is supplied by the embedder rather than imposed by the
// component.
class IsolatedHost final
    : public content::DocumentService<mojom::IsolatedHost> {
 public:
  static void Create(base::WeakPtr<IsolatedSourceProvider> provider,
                     content::RenderFrameHost* frame,
                     mojo::PendingReceiver<mojom::IsolatedHost> receiver);
  // Generated Mojo dispatch calls this for renderer IsolatedBindings::Load.
  void Load(const std::string& name, LoadCallback callback) override;

 private:
  IsolatedHost(base::WeakPtr<IsolatedSourceProvider> provider,
               content::RenderFrameHost& frame,
               mojo::PendingReceiver<mojom::IsolatedHost> receiver);
  ~IsolatedHost() override;
  base::WeakPtr<IsolatedSourceProvider> provider_;
};
}  // namespace tuffite
#endif
