// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_DOWNLOAD_DOWNLOAD_SERVICE_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_DOWNLOAD_DOWNLOAD_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"

namespace content {
class DownloadManagerDelegate;
}
namespace tuffite {
class DownloadService : public KeyedService {
 public:
  virtual content::DownloadManagerDelegate* delegate() = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_DOWNLOAD_DOWNLOAD_SERVICE_H_
