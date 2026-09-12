// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_SAFE_STORAGE_KEY_PROVIDER_MAC_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_SAFE_STORAGE_KEY_PROVIDER_MAC_H_

#include <memory>
#include <string>

#include "base/files/file_path.h"
#include "components/os_crypt/async/browser/key_provider.h"

namespace tuffite {
std::unique_ptr<os_crypt_async::KeyProvider> CreateSafeStorageKeyProvider(
    std::string application_id,
    std::string product_name,
    base::FilePath profile);
}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_SAFE_STORAGE_KEY_PROVIDER_MAC_H_
