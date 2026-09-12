// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_IMPL_H_

#include <memory>

namespace tuffite::remoting {
class RemotingService;

std::unique_ptr<RemotingService> CreateRemotingService();

}  // namespace tuffite::remoting

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_IMPL_H_
