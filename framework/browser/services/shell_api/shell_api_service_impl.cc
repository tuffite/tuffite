// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/shell_api_service_impl.h"

#include "base/functional/bind.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/dispatch/common/codec.h"
#include "tuffite/components/dispatch/common/status.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/builtin.h"
#include "tuffite/framework/browser/web_contents_role.h"

namespace tuffite {

ShellApiServiceImpl::~ShellApiServiceImpl() {
  Shutdown();
}

base::WeakPtr<ShellApiService> ShellApiServiceImpl::GetWeakPtr() {
  return weak_factory_.GetWeakPtr();
}

ShellApiServiceImpl::ShellApiServiceImpl(content::BrowserContext& context,
                                         HostDispatch& host,
                                         ManifestService& manifest,
                                         UtilityService& utility)
    : router_(shell_api::BuiltinServices()),
      browser_context_(context),
      host_(host),
      manifest_(manifest),
      utility_(utility) {}
bool ShellApiServiceImpl::IsAuthorized(content::RenderFrameHost& frame) const {
  if (stopped_ || frame.GetBrowserContext() != &browser_context_.get()) {
    return false;
  }
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  auto role = WebContentsRoleData::GetRole(contents);
  const auto* explorer = WebContentsRoleData::GetExplorerPolicy(contents);
  return contents && contents->GetPrimaryMainFrame() == &frame &&
         (role == WebContentsRole::kApp ||
          (role == WebContentsRole::kExplorer && explorer &&
           explorer->allow_shell_api)) &&
         manifest_->IsShellApiOriginAllowed(frame.GetLastCommittedOrigin());
}

bool ShellApiServiceImpl::IsEventAuthorized(
    content::RenderFrameHost& frame) const {
  // A command allowlist must not implicitly expose unrelated business events.
  return IsAuthorized(frame) &&
         manifest_->IsAllShellApiAllowed(frame.GetLastCommittedOrigin());
}

void ShellApiServiceImpl::Dispatch(content::RenderFrameHost& frame,
                                   const std::string& channel,
                                   const std::vector<uint8_t>& payload,
                                   const base::UnguessableToken& document,
                                   Reply reply) {
  DispatchRequest(frame, next_legacy_id_++, channel, payload, document,
                  std::move(reply));
}

void ShellApiServiceImpl::CancelRequest(
    const base::UnguessableToken& connection,
    uint64_t id) {
  requests_.Cancel({connection, id});
}

void ShellApiServiceImpl::DispatchRequest(
    content::RenderFrameHost& frame,
    uint64_t id,
    const std::string& channel,
    const std::vector<uint8_t>& payload,
    const base::UnguessableToken& document,
    Reply reply) {
  const auto& manifest = *manifest_;
  const auto interface = manifest.ApplicationInterface(channel);
  const bool application_command = !interface.empty();
  const auto request =
      application_command ? rpc::DecodeRequest(payload) : std::nullopt;
  if (!IsAuthorized(frame) ||
      !manifest.IsShellApiCommandAllowed(frame.GetLastCommittedOrigin(),
                                         channel) ||
      (application_command &&
       (!manifest.IsShellApiScopeAllowed(frame.GetLastCommittedOrigin(),
                                         channel, "", "") ||
        !request ||
        !manifest.IsApplicationCommandRequest(channel, request->command_id)))) {
    std::move(reply).Run(static_cast<int32_t>(rpc::Status::kAccessDenied), {});
    return;
  }
  if (payload.size() > 16 * 1024 * 1024) {
    std::move(reply).Run(static_cast<int32_t>(rpc::Status::kInvalidArgument),
                         {});
    return;
  }
  utility_owners_.insert_or_assign(document, document);
  const auto operation = base::UnguessableToken::Create();
  auto service = utility_->ResolveUtilityInterface(interface);
  // Older application tables registered each Utility under its interface name.
  if (service.empty() && !interface.empty() &&
      utility_->IsRegisteredUtility(interface)) {
    service = interface;
  }
  base::OnceClosure cancel;
  if (service.empty()) {
    cancel = base::BindOnce(
        [](HostDispatch* host, base::UnguessableToken op) {
          host->CancelDocument(op);
        },
        &host_.get(), operation);
  } else {
    cancel = base::BindOnce(
        [](UtilityService* utility, base::UnguessableToken owner,
           std::string service, uint64_t id) {
          utility->CancelUtilityService(owner, std::move(service), id);
        },
        &utility_.get(), document, service, id);
  }
  auto admitted =
      requests_.Admit({document, id}, std::move(reply), std::move(cancel));
  if (!admitted) {
    return;
  }
  reply = std::move(*admitted);
  if (!service.empty()) {
    utility_->InvokeUtilityService(
        document, id, service, payload,
        base::BindOnce(
            [](Reply reply, uint64_t expected, int32_t status,
               std::vector<uint8_t> response) {
              const auto frame = rpc::DecodeResponse(response);
              if (status != 0 || !frame || frame->request_id != expected) {
                std::move(reply).Run(
                    static_cast<int32_t>(rpc::Status::kUnavailable), {});
                return;
              }
              std::move(reply).Run(static_cast<int32_t>(rpc::Status::kOk),
                                   response);
            },
            std::move(reply), request->request_id));
    return;
  }
  if (application_command) {
    host_->Dispatch(payload,
                    base::BindOnce(
                        [](Reply reply, bool ok, std::vector<uint8_t> bytes) {
                          const auto status =
                              ok ? rpc::Status::kOk : rpc::Status::kUnavailable;
                          std::move(reply).Run(static_cast<int32_t>(status),
                                               std::move(bytes));
                        },
                        std::move(reply)),
                    operation);
    return;
  }
  if (!router_.Dispatch(frame, channel, payload, document, &reply)) {
    std::move(reply).Run(static_cast<int32_t>(rpc::Status::kInvalidArgument),
                         {});
  }
}

void ShellApiServiceImpl::Subscribe(const base::UnguessableToken& document,
                                    Event event) {
  if (!stopped_) {
    subscriptions_.emplace(document, std::move(event));
  }
}

void ShellApiServiceImpl::CancelDocument(
    const base::UnguessableToken& document) {
  subscriptions_.erase(document);
  requests_.CancelConnection(document);
  auto owner = utility_owners_.extract(document);
  if (!owner.empty()) {
    utility_->CancelUtilityOwner(owner.mapped());
  }
}

void ShellApiServiceImpl::Broadcast(const std::string& name,
                                    const std::vector<uint8_t>& bytes) {
  if (stopped_) {
    return;
  }
  auto weak = weak_factory_.GetWeakPtr();
  auto callbacks = subscriptions_;
  for (auto& [id, event] : callbacks) {
    if (!weak || weak->stopped_) {
      break;
    }
    event.Run(name, bytes);
  }
}

void ShellApiServiceImpl::Shutdown() {
  if (stopped_) {
    return;
  }
  stopped_ = true;
  weak_factory_.InvalidateWeakPtrs();
  subscriptions_.clear();
  for (const auto& [document, owner] : utility_owners_) {
    utility_->CancelUtilityOwner(owner);
  }
  utility_owners_.clear();
  requests_.Shutdown();
}
}  // namespace tuffite
