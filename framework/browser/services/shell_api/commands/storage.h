// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_STORAGE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_STORAGE_H_

#include "tuffite/framework/shell_api/storage.mojom.h"

namespace tuffite::shell_api::storage {

class StorageImpl final : public generated::Storage {
 public:
  void Get(const dispatch::Context& context,
           generated::Storage::GetRequestPtr request,
           generated::Storage::GetCallback callback) override;
  void Set(const dispatch::Context& context,
           generated::Storage::SetRequestPtr request,
           generated::Storage::SetCallback callback) override;
  void Remove(const dispatch::Context& context,
              generated::Storage::RemoveRequestPtr request,
              generated::Storage::RemoveCallback callback) override;
  void Clear(const dispatch::Context& context,
             generated::Storage::ClearRequestPtr request,
             generated::Storage::ClearCallback callback) override;
};

}  // namespace tuffite::shell_api::storage

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_STORAGE_H_
