// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REMOTING_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REMOTING_H_

#include "tuffite/framework/public/shell_api/remoting.mojom.h"

namespace tuffite::shell_api::remoting {

class RemotingImpl final : public generated::Remoting {
 public:
  void Capabilities(
      const dispatch::RendererRequestContext& context,
      generated::Remoting::CapabilitiesRequestPtr request,
      generated::Remoting::CapabilitiesCallback callback) override;
  void Start(const dispatch::RendererRequestContext& context,
             generated::Remoting::StartRequestPtr request,
             generated::Remoting::StartCallback callback) override;
  void ProcessSignal(
      const dispatch::RendererRequestContext& context,
      generated::Remoting::ProcessSignalRequestPtr request,
      generated::Remoting::ProcessSignalCallback callback) override;
  void TakeSignals(const dispatch::RendererRequestContext& context,
                   generated::Remoting::TakeSignalsRequestPtr request,
                   generated::Remoting::TakeSignalsCallback callback) override;
  void SendInput(const dispatch::RendererRequestContext& context,
                 generated::Remoting::SendInputRequestPtr request,
                 generated::Remoting::SendInputCallback callback) override;
  void InjectInput(const dispatch::RendererRequestContext& context,
                   generated::Remoting::InjectInputRequestPtr request,
                   generated::Remoting::InjectInputCallback callback) override;

  void Stop(const dispatch::RendererRequestContext& context,
            generated::Remoting::StopRequestPtr request,
            generated::Remoting::StopCallback callback) override;
  void Stats(const dispatch::RendererRequestContext& context,
             generated::Remoting::StatsRequestPtr request,
             generated::Remoting::StatsCallback callback) override;
  void Capture(const dispatch::RendererRequestContext& context,
               generated::Remoting::CaptureRequestPtr request,
               generated::Remoting::CaptureCallback callback) override;
};

}  // namespace tuffite::shell_api::remoting

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REMOTING_H_
