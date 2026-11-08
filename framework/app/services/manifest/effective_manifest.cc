// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/manifest/effective_manifest.h"

#include <utility>

#include "tuffite/components/manifest/common/manifest.h"

namespace tuffite {

EffectiveManifest::EffectiveManifest(std::unique_ptr<app::Manifest> data)
    : data_(std::move(data)) {}
EffectiveManifest::EffectiveManifest(EffectiveManifest&&) noexcept = default;
EffectiveManifest& EffectiveManifest::operator=(EffectiveManifest&&) noexcept =
    default;
EffectiveManifest::~EffectiveManifest() = default;

}  // namespace tuffite
