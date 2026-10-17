// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/browser/explorer_policy_provider.h"
namespace tuffite {
void ExplorerPolicyProvider::ResolveMessageWindow(
    content::RenderFrameHost&,
    int64_t,
    base::OnceCallback<void(content::RenderFrameHost*)> callback) const {
  std::move(callback).Run(nullptr);
}
}  // namespace tuffite
