// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/remoting.h"

#include <utility>

#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/remoting/remoting_protocol.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"
namespace tuffite::shell_api::remoting {
namespace {
namespace api = ::tuffite::remoting;
api::RemotingService* ServiceFor(
    const dispatch::RendererRequestContext& context) {
  return BrowserContext::GetFrom(context.frame())
      .application_context()
      .GetService<api::RemotingService>();
}

bool IsAllowed(const dispatch::RendererRequestContext& context,
               std::string_view method) {
  auto& frame = context.frame();
  const ManifestService* manifest = ManifestFor(frame);
  return manifest &&
         manifest->IsShellApiCommandAllowed(frame.GetLastCommittedOrigin(),
                                            method) &&
         IsUnscopedAllowed(context, method);
}

}  // namespace
void RemotingImpl::Capabilities(
    const dispatch::RendererRequestContext& context,
    generated::Remoting::CapabilitiesRequestPtr request,
    generated::Remoting::CapabilitiesCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.Capabilities")) {
    std::move(callback).Run(rpc::Status::kOk,
                            generated::RemotingCapabilities::New());
    return;
  }
  api::Protocol::Capabilities(*ServiceFor(context), std::move(request),
                              std::move(callback));
}
void RemotingImpl::Start(const dispatch::RendererRequestContext& context,
                         generated::Remoting::StartRequestPtr request,
                         generated::Remoting::StartCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.Start")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  api::Protocol::Start(*ServiceFor(context), std::move(request),
                       std::move(callback));
}
void RemotingImpl::ProcessSignal(
    const dispatch::RendererRequestContext& context,
    generated::Remoting::ProcessSignalRequestPtr request,
    generated::Remoting::ProcessSignalCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.ProcessSignal")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  api::Protocol::ProcessSignal(*ServiceFor(context), std::move(request),
                               std::move(callback));
}
void RemotingImpl::TakeSignals(
    const dispatch::RendererRequestContext& context,
    generated::Remoting::TakeSignalsRequestPtr request,
    generated::Remoting::TakeSignalsCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.TakeSignals")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  api::Protocol::TakeSignals(*ServiceFor(context), std::move(request),
                             std::move(callback));
}
void RemotingImpl::SendInput(const dispatch::RendererRequestContext& context,
                             generated::Remoting::SendInputRequestPtr request,
                             generated::Remoting::SendInputCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.SendInput")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  api::Protocol::SendInput(*ServiceFor(context), std::move(request),
                           std::move(callback));
}
void RemotingImpl::InjectInput(
    const dispatch::RendererRequestContext&,
    generated::Remoting::InjectInputRequestPtr,
    generated::Remoting::InjectInputCallback callback) {
  rpc::Reject(callback, rpc::Status::kAccessDenied);
}
void RemotingImpl::Stop(const dispatch::RendererRequestContext& context,
                        generated::Remoting::StopRequestPtr request,
                        generated::Remoting::StopCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.Stop")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  api::Protocol::Stop(*ServiceFor(context), std::move(request),
                      std::move(callback));
}
void RemotingImpl::Stats(const dispatch::RendererRequestContext& context,
                         generated::Remoting::StatsRequestPtr request,
                         generated::Remoting::StatsCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.Stats")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  api::Protocol::Stats(*ServiceFor(context), std::move(request),
                       std::move(callback));
}
void RemotingImpl::Capture(const dispatch::RendererRequestContext& context,
                           generated::Remoting::CaptureRequestPtr request,
                           generated::Remoting::CaptureCallback callback) {
  if (!IsAllowed(context, "tuffite.Remoting.Capture")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  api::Protocol::Capture(*ServiceFor(context), std::move(request),
                         std::move(callback));
}
}  // namespace tuffite::shell_api::remoting
