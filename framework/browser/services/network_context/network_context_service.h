// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_NETWORK_CONTEXT_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_NETWORK_CONTEXT_SERVICE_H_

#include "base/files/file_path.h"
#include "components/keyed_service/core/keyed_service.h"
#include "services/network/public/mojom/network_context.mojom-forward.h"

namespace tuffite {
class NetworkContextService : public KeyedService {
 public:
  virtual void Configure(bool in_memory,
                         const base::FilePath& relative_partition_path,
                         network::mojom::NetworkContextParams* params) = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_NETWORK_CONTEXT_NETWORK_CONTEXT_SERVICE_H_
