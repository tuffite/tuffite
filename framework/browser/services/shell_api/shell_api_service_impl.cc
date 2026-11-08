// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/shell_api_service_impl.h"

#include "base/functional/bind.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/dispatch/common/status.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/builtin.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/public/c_api/types.h"

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
std::optional<MojomMethod> ShellApiServiceImpl::ResolveMojomMethod(
    std::string_view name,
    uint32_t ordinal) const {
  if (auto method = router_.ResolveMethod(name, ordinal)) {
    return MojomMethod{*method};
  }
  auto method = host_->ResolveMojomMethod(std::string(name), ordinal);
  if (!method || !manifest_->IsApplicationCommandRequest(method->channel,
                                                         method->command_id)) {
    return std::nullopt;
  }
  return method;
}

void ShellApiServiceImpl::DispatchMojom(content::RenderFrameHost& frame,
                                        const MojomMethod& method,
                                        const std::vector<uint8_t>& parameters,
                                        const base::UnguessableToken& document,
                                        uint64_t id,
                                        Reply reply) {
  const auto& channel = method.channel;
  const auto interface = manifest_->ApplicationInterface(channel);
  if (!IsAuthorized(frame) ||
      !manifest_->IsShellApiCommandAllowed(frame.GetLastCommittedOrigin(),
                                           channel) ||
      (!interface.empty() &&
       !manifest_->IsShellApiScopeAllowed(frame.GetLastCommittedOrigin(),
                                          channel, "", ""))) {
    std::move(reply).Run(static_cast<int32_t>(rpc::Status::kAccessDenied), {});
    return;
  }
  if (parameters.size() > 16 * 1024 * 1024) {
    std::move(reply).Run(1, {});
    return;
  }
  const auto operation = base::UnguessableToken::Create();
  const auto service = utility_->ResolveUtilityInterface(interface);
  utility_owners_.insert_or_assign(document, document);
  auto cancel =
      service.empty()
          ? base::BindOnce(
                [](HostDispatch* host, base::UnguessableToken op) {
                  host->CancelDocument(op);
                },
                &host_.get(), operation)
          : base::BindOnce(
                [](UtilityService* utility, base::UnguessableToken owner,
                   std::string service, uint64_t id) {
                  utility->CancelUtilityService(owner, std::move(service), id);
                },
                &utility_.get(), document, service, id);
  auto admitted =
      requests_.Admit({document, id}, std::move(reply), std::move(cancel));
  if (!admitted) {
    return;
  }
  reply = std::move(*admitted);
  if (!service.empty()) {
    utility_->InvokeUtilityMojom(
        document, id, service, method.command_id, method.schema_fingerprint,
        parameters,
        base::BindOnce(
            [](Reply reply, int32_t status, std::vector<uint8_t> bytes) {
              std::move(reply).Run(
                  status == TUFFITE_STATUS_OK                   ? 0
                  : status == TUFFITE_STATUS_INVALID_ARGUMENT   ? 1
                  : status == TUFFITE_STATUS_RESOURCE_EXHAUSTED ? 6
                                                                : 3,
                  bytes);
            },
            std::move(reply)));
  } else if (!interface.empty()) {
    host_->DispatchMojom(method, parameters, operation, std::move(reply));
  } else if (!router_.Dispatch(
                 dispatch::RendererRequestContext(frame, document, id), channel,
                 parameters, &reply)) {
    std::move(reply).Run(1, {});
  }
}

void ShellApiServiceImpl::CancelMojomRequest(
    const base::UnguessableToken& document,
    uint64_t id) {
  requests_.Cancel({document, id});
}

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
