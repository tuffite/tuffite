// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/isolated/isolated_service_impl.h"

#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/web_contents_role.h"

namespace tuffite {
IsolatedServiceImpl::IsolatedServiceImpl(const ManifestService& manifest)
    : registry_(std::make_unique<IsolatedRegistry>()) {
  for (const auto& [name, definition] : manifest.IsolatedWorldDefinitions()) {
    registry_->AddBundled(name, definition);
  }
  const auto& windows = manifest.startup_windows();
  if (!windows.empty()) {
    const auto packaged = url::Origin::Create(manifest.app_info().entry_url);
    const auto effective = url::Origin::Create(windows.front().url);
    if (packaged != effective) {
      registry_->SetDevelopmentAlias(packaged, effective);
    }
  }
}
IsolatedServiceImpl::~IsolatedServiceImpl() = default;
std::optional<std::string> IsolatedServiceImpl::SourceForFrame(
    content::RenderFrameHost& frame,
    const std::string& name) const {
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  if (frame.GetParent() ||
      WebContentsRoleData::GetRole(contents) != WebContentsRole::kApp) {
    return std::nullopt;
  }
  return Source(name, frame.GetLastCommittedOrigin());
}
std::optional<std::string> IsolatedServiceImpl::Source(
    std::string_view name,
    const url::Origin& origin) const {
  return registry_ ? registry_->Source(name, origin) : std::nullopt;
}
std::optional<std::string> IsolatedServiceImpl::Apply(
    const base::DictValue& request) {
  return registry_ ? registry_->Apply(request) : std::nullopt;
}
base::WeakPtr<IsolatedService> IsolatedServiceImpl::GetWeakPtr() {
  return weak_factory_.GetWeakPtr();
}
void IsolatedServiceImpl::Shutdown() {
  weak_factory_.InvalidateWeakPtrs();
  registry_.reset();
}
}  // namespace tuffite
