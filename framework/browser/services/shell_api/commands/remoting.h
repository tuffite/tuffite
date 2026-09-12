// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REMOTING_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REMOTING_H_

#include "tuffite/framework/shell_api/remoting.h"

namespace tuffite::shell_api::remoting {

void Capabilities(const ::tuffite::dispatch::Context& context,
                  generated::remoting::CapabilitiesRequest&& request,
                  generated::remoting::CapabilitiesCallback&& callback);
void Start(const ::tuffite::dispatch::Context& context,
           generated::remoting::StartRequest&& request,
           generated::remoting::StartCallback&& callback);
void ProcessSignal(const ::tuffite::dispatch::Context& context,
                   generated::remoting::ProcessSignalRequest&& request,
                   generated::remoting::ProcessSignalCallback&& callback);
void TakeSignals(const ::tuffite::dispatch::Context& context,
                 generated::remoting::TakeSignalsRequest&& request,
                 generated::remoting::TakeSignalsCallback&& callback);
void SendInput(const ::tuffite::dispatch::Context& context,
               generated::remoting::SendInputRequest&& request,
               generated::remoting::SendInputCallback&& callback);
void InjectInput(const ::tuffite::dispatch::Context& context,
                 generated::remoting::InjectInputRequest&& request,
                 generated::remoting::InjectInputCallback&& callback);

void Stop(const ::tuffite::dispatch::Context& context,
          generated::remoting::StopRequest&& request,
          generated::remoting::StopCallback&& callback);
void Stats(const ::tuffite::dispatch::Context& context,
           generated::remoting::StatsRequest&& request,
           generated::remoting::StatsCallback&& callback);
void Capture(const ::tuffite::dispatch::Context& context,
             generated::remoting::CaptureRequest&& request,
             generated::remoting::CaptureCallback&& callback);

}  // namespace tuffite::shell_api::remoting

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REMOTING_H_
