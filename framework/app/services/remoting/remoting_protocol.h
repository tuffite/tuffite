// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_PROTOCOL_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_PROTOCOL_H_

#include "tuffite/framework/app/services/remoting/remoting_service.h"
#include "tuffite/framework/public/shell_api/remoting.mojom.h"
namespace tuffite::remoting {
// Data conversion shared by the independently authorized renderer and native
// adapters.
class Protocol final {
 public:
  static void Capabilities(
      RemotingService&,
      shell_api::generated::Remoting::CapabilitiesRequestPtr,
      shell_api::generated::Remoting::CapabilitiesCallback);
  static void Start(RemotingService&,
                    shell_api::generated::Remoting::StartRequestPtr,
                    shell_api::generated::Remoting::StartCallback);
  static void ProcessSignal(
      RemotingService&,
      shell_api::generated::Remoting::ProcessSignalRequestPtr,
      shell_api::generated::Remoting::ProcessSignalCallback);
  static void TakeSignals(RemotingService&,
                          shell_api::generated::Remoting::TakeSignalsRequestPtr,
                          shell_api::generated::Remoting::TakeSignalsCallback);
  static void SendInput(RemotingService&,
                        shell_api::generated::Remoting::SendInputRequestPtr,
                        shell_api::generated::Remoting::SendInputCallback);
  static void InjectInput(RemotingService&,
                          shell_api::generated::Remoting::InjectInputRequestPtr,
                          shell_api::generated::Remoting::InjectInputCallback);
  static void Stop(RemotingService&,
                   shell_api::generated::Remoting::StopRequestPtr,
                   shell_api::generated::Remoting::StopCallback);
  static void Stats(RemotingService&,
                    shell_api::generated::Remoting::StatsRequestPtr,
                    shell_api::generated::Remoting::StatsCallback);
  static void Capture(RemotingService&,
                      shell_api::generated::Remoting::CaptureRequestPtr,
                      shell_api::generated::Remoting::CaptureCallback);
};
}  // namespace tuffite::remoting
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_REMOTING_PROTOCOL_H_
