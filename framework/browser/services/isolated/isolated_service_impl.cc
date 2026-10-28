// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/isolated/isolated_service_impl.h"

#include "base/strings/escape.h"
#include "base/unguessable_token.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "net/base/mime_util.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/web_contents_role.h"

namespace tuffite {
IsolatedServiceImpl::IsolatedServiceImpl(const ManifestService& manifest)
    : scheme_(manifest.app_info().schemes.front()),
      registry_(std::make_unique<IsolatedRegistry>()) {
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
std::optional<std::string> IsolatedServiceImpl::LoadForFrame(
    content::RenderFrameHost& frame,
    const std::string& name) {
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  const auto role = WebContentsRoleData::GetRole(contents);
  // Internal application routes can live in Explorer WebContents. The
  // browser-owned policy establishes the surface; the registration's origin
  // grants still authorize the committed document on every load.
  const bool allowed_surface =
      role == WebContentsRole::kApp ||
      (role == WebContentsRole::kExplorer &&
       WebContentsRoleData::GetExplorerPolicy(contents));
  if (!contents || contents->GetPrimaryMainFrame() != &frame ||
      !allowed_surface) {
    return std::nullopt;
  }
  if (!registry_) {
    return std::nullopt;
  }
  auto definition = registry_->Find(name, frame.GetLastCommittedOrigin());
  if (!definition) {
    return std::nullopt;
  }
  size_t bytes = definition->ByteSize();
  for (const auto& [url, lease] : leases_) {
    if (lease.frame == frame.GetGlobalId()) {
      bytes += lease.definition->ByteSize();
    }
  }
  if (bytes > 32 * 1024 * 1024) {
    return std::nullopt;
  }
  const auto origin = frame.GetLastCommittedOrigin();
  GURL base = origin.scheme() == scheme_ ? origin.GetURL()
                                         : GURL(scheme_ + "://tuffite/");
  base = base.Resolve("/.tuffite-isolated/" +
                      base::UnguessableToken::Create().ToString() + "/");
  const auto entry = base.Resolve(definition->entry).spec();
  leases_.emplace(base.spec(), Lease{std::move(definition), frame.GetGlobalId(),
                                     frame.GetWeakDocumentPtr(), origin});
  return entry;
}
namespace {
std::string PackageBase(const GURL& url) {
  constexpr std::string_view prefix = "/.tuffite-isolated/";
  if (!url.path().starts_with(prefix)) {
    return {};
  }
  auto end = url.path().find('/', prefix.size());
  return end == std::string::npos ? std::string()
                                  : url.GetWithEmptyPath().spec() +
                                        std::string(url.path().substr(1, end));
}
}  // namespace
void IsolatedServiceImpl::Release(const std::string& entry_url) {
  leases_.erase(PackageBase(GURL(entry_url)));
}
std::optional<asset::MemoryAsset> IsolatedServiceImpl::ResourceForFrame(
    content::RenderFrameHost& frame,
    const GURL& url) const {
  auto base = PackageBase(url);
  auto it = leases_.find(base);
  if (it == leases_.end() || it->second.frame != frame.GetGlobalId() ||
      it->second.document.AsRenderFrameHostIfValid() != &frame) {
    return std::nullopt;
  }
  auto path = base::UnescapeURLComponent(
      url.spec().substr(
          base.size(),
          url.spec().find_first_of("?#", base.size()) - base.size()),
      base::UnescapeRule::NORMAL);
  const auto& definition = *it->second.definition;
  auto found = definition.resources.find(path);
  if (found == definition.resources.end() && path != definition.entry) {
    return std::nullopt;
  }
  std::string mime;
  net::GetMimeTypeFromFile(base::FilePath::FromUTF8Unsafe(path), &mime);
  if (path.ends_with(".mjs") || path.ends_with(".js")) {
    mime = "text/javascript";
  }
  if (mime.empty()) {
    mime = "application/octet-stream";
  }
  return asset::MemoryAsset{
      std::move(mime),
      found == definition.resources.end() ? definition.source : found->second,
      it->second.origin.Serialize()};
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
  leases_.clear();
  registry_.reset();
}
}  // namespace tuffite
