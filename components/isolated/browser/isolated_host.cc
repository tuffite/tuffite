// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/isolated/browser/isolated_host.h"

#include "tuffite/components/isolated/browser/isolated_source_provider.h"
namespace tuffite {
void IsolatedHost::Create(base::WeakPtr<IsolatedSourceProvider> provider,
                          content::RenderFrameHost* frame,
                          mojo::PendingReceiver<mojom::IsolatedHost> receiver) {
  if (provider && frame) {
    new IsolatedHost(std::move(provider), *frame, std::move(receiver));
  }
}
IsolatedHost::IsolatedHost(base::WeakPtr<IsolatedSourceProvider> provider,
                           content::RenderFrameHost& frame,
                           mojo::PendingReceiver<mojom::IsolatedHost> receiver)
    : DocumentService(frame, std::move(receiver)),
      provider_(std::move(provider)) {}
IsolatedHost::~IsolatedHost() = default;
void IsolatedHost::Load(const std::string& name, LoadCallback callback) {
  std::move(callback).Run(
      provider_ ? provider_->SourceForFrame(render_frame_host(), name)
                : std::nullopt);
}
}  // namespace tuffite
