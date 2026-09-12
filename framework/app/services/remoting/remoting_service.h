// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/framework/app/services/remoting/remoting_types.h"

namespace tuffite::remoting {

class RemotingService : public KeyedService {
 public:
  virtual CapabilitiesResult GetCapabilities() const = 0;
  virtual SessionResult Start(StartOptions options) = 0;
  virtual EmptyResult ProcessSignal(uint64_t session_id, Signal signal) = 0;
  virtual SignalsResult TakeSignals(uint64_t session_id,
                                    uint64_t after_sequence) = 0;
  virtual EmptyResult SendInput(uint64_t session_id, InputEvent event) = 0;
  virtual EmptyResult InjectInput(InputEvent event) = 0;
  virtual void Capture(std::optional<int64_t> screen_id,
                       SnapshotCallback callback) = 0;
  virtual EmptyResult Stop(uint64_t session_id) = 0;
  virtual StatsResult GetStats(uint64_t session_id) const = 0;
};

}  // namespace tuffite::remoting

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_H_
