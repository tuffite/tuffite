// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_H_

#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/framework/app/services/remoting/remoting_types.h"

namespace tuffite::remoting {

using SessionCallback = base::OnceCallback<void(SessionResult)>;
using EmptyCallback = base::OnceCallback<void(EmptyResult)>;
using SignalsCallback = base::OnceCallback<void(SignalsResult)>;
using StatsCallback = base::OnceCallback<void(StatsResult)>;

class RemotingService : public KeyedService {
 public:
  virtual CapabilitiesResult GetCapabilities() const = 0;
  virtual void Start(StartOptions options, SessionCallback callback) = 0;
  virtual void ProcessSignal(uint64_t session_id,
                             Signal signal,
                             EmptyCallback callback) = 0;
  virtual void TakeSignals(uint64_t session_id,
                           uint64_t after_sequence,
                           SignalsCallback callback) = 0;
  virtual void SendInput(uint64_t session_id,
                         InputEvent event,
                         EmptyCallback callback) = 0;
  virtual void InjectInput(InputEvent event, EmptyCallback callback) = 0;
  virtual void Capture(std::optional<int64_t> screen_id,
                       SnapshotCallback callback) = 0;
  virtual void Stop(uint64_t session_id, EmptyCallback callback) = 0;
  virtual void GetStats(uint64_t session_id, StatsCallback callback) const = 0;
};

}  // namespace tuffite::remoting

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_SERVICE_H_
