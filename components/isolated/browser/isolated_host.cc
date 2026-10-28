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
IsolatedHost::~IsolatedHost() {
  if (provider_) {
    for (const auto& entry : entries_) {
      provider_->Release(entry);
    }
  }
}
void IsolatedHost::Load(const std::string& name, LoadCallback callback) {
  auto entry = provider_ && entries_.size() < 32
                   ? provider_->LoadForFrame(render_frame_host(), name)
                   : std::nullopt;
  if (entry) {
    entries_.insert(*entry);
  }
  std::move(callback).Run(entry);
}
void IsolatedHost::Release(const std::string& entry_url) {
  if (entries_.erase(entry_url) && provider_) {
    provider_->Release(entry_url);
  }
}
}  // namespace tuffite
