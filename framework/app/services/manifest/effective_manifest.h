// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_EFFECTIVE_MANIFEST_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_EFFECTIVE_MANIFEST_H_

#include <memory>

#include "tuffite/components/manifest/common/manifest.h"

namespace tuffite {

// Can only be created by the loader. Published services cannot mutate it.
class EffectiveManifest {
 public:
  EffectiveManifest(EffectiveManifest&&) noexcept;
  EffectiveManifest& operator=(EffectiveManifest&&) noexcept;
  ~EffectiveManifest();

  // The resolved manifest is immutable and remains owned by this snapshot.
  const app::Manifest& manifest() const { return *data_; }

 private:
  friend class ManifestLoader;
  explicit EffectiveManifest(std::unique_ptr<app::Manifest> data);
  std::unique_ptr<const app::Manifest> data_;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_MANIFEST_EFFECTIVE_MANIFEST_H_
