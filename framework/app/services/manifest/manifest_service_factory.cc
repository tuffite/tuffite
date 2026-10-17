// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/manifest/manifest_service_factory.h"

#include "base/check.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service_impl.h"
namespace tuffite {
std::unique_ptr<ManifestService> ManifestServiceFactory::Load(
    const base::FilePath& package_root,
    std::string* error,
    const std::optional<GURL>& development_url) {
  return ManifestServiceImpl::Load(package_root, error, development_url);
}
std::optional<std::string> ManifestServiceFactory::ReadApplicationScheme(
    const base::FilePath& package_root,
    std::string* error) {
  return ManifestServiceImpl::ReadApplicationScheme(package_root, error);
}
bool ManifestServiceFactory::ReadNetworkDisableHttp2(
    const base::FilePath& package_root) {
  return ManifestServiceImpl::ReadNetworkDisableHttp2(package_root);
}
ManifestServiceFactory* ManifestServiceFactory::Get() {
  static base::NoDestructor<ManifestServiceFactory> instance;
  return instance.get();
}
ManifestServiceFactory::ManifestServiceFactory()
    : ApplicationKeyedServiceFactory<ManifestService>("TuffiteManifest") {}
bool ManifestServiceFactory::ServiceIsCreatedWithContext() const {
  return true;
}
std::unique_ptr<ManifestService>
ManifestServiceFactory::BuildServiceInstanceForApplication(
    ApplicationContext& context) const {
  CHECK(context.pending_manifest_);
  return std::move(context.pending_manifest_);
}
}  // namespace tuffite
