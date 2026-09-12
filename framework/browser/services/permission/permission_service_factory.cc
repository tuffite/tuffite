// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/permission/permission_service_factory.h"

#include <optional>
#include <utility>

#include "base/logging.h"
#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "content/public/browser/permission_request_description.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "services/network/public/cpp/is_potentially_trustworthy.h"
#include "third_party/blink/public/common/permissions/permission_utils.h"
#include "third_party/blink/public/mojom/permissions/permission.mojom.h"
#include "tuffite/framework/browser/services/permission/permission_service.h"
#include "url/origin.h"

namespace tuffite {
namespace {
using Status = blink::mojom::PermissionStatus;
using Descriptor = blink::mojom::PermissionDescriptorPtr;
using Result = content::PermissionResult;

// Diagnostics contain only the permission type, entry point and decision.
// Never log origins, URLs, permission payloads or authentication data.
Status TraceDecision(const char* scope,
                     const Descriptor& permission,
                     Status status) {
  if (VLOG_IS_ON(1)) {
    const auto type =
        permission
            ? blink::MaybePermissionDescriptorToPermissionType(permission)
            : std::nullopt;
    VLOG(1) << "Permission decision: scope=" << scope << " type="
            << (type ? blink::GetPermissionString(*type) : "unknown")
            << " status="
            << (status == Status::GRANTED ? "granted"
                : status == Status::ASK   ? "prompt"
                                          : "denied");
  }
  return status;
}

// No persistent grants: standard clipboard writes require fresh user
// activation.
class PermissionServiceImpl final : public PermissionService {
 public:
  void RequestPermissionsFromCurrentDocument(
      content::RenderFrameHost* frame,
      const content::PermissionRequestDescription& request,
      base::OnceCallback<void(const std::vector<Result>&)> callback) override {
    std::vector<Result> results;
    auto* contents = content::WebContents::FromRenderFrameHost(frame);
    auto* focused_frame = contents ? contents->GetFocusedFrame() : nullptr;
    // Navigation can clear FrameTree's focused node while Chromium retains
    // main-frame page focus. Respect that default without allowing an ancestor
    // of an explicitly focused subframe to write to the clipboard.
    const bool focused =
        contents && contents->ContainsOrIsFocusedWebContents() &&
        frame->IsFocused() &&
        (focused_frame ? focused_frame == frame
                       : contents->GetPrimaryMainFrame() == frame);
    const bool active =
        frame->IsActive() && !frame->IsNestedWithinFencedFrame() &&
        frame->HasTransientUserActivation() && contents && focused &&
        contents->GetVisibility() == content::Visibility::VISIBLE;
    VLOG(1) << "Permission request document state: active=" << frame->IsActive()
            << " activation=" << frame->HasTransientUserActivation()
            << " focused=" << focused << " visible="
            << (contents &&
                contents->GetVisibility() == content::Visibility::VISIBLE);
    for (const auto& permission : request.permissions) {
      const bool allowed =
          active &&
          request.requesting_origin ==
              frame->GetLastCommittedOrigin().GetURL() &&
          Query(permission, request.requesting_origin) == Status::ASK;
      VLOG(1) << "Permission request eligibility: same_origin="
              << (request.requesting_origin ==
                  frame->GetLastCommittedOrigin().GetURL())
              << " query="
              << static_cast<int>(Query(permission, request.requesting_origin));
      results.emplace_back(TraceDecision(
          "request", permission, allowed ? Status::GRANTED : Status::DENIED));
    }
    std::move(callback).Run(results);
  }

  Status GetPermissionStatus(const Descriptor& permission,
                             const GURL& requesting_origin,
                             const GURL& embedding_origin) override {
    return TraceDecision("origin", permission,
                         Query(permission, requesting_origin));
  }

  Result GetPermissionResultForOriginWithoutContext(
      const Descriptor& permission,
      const url::Origin& requesting_origin,
      const url::Origin& embedding_origin) override {
    return Result(TraceDecision("origin_without_context", permission,
                                Query(permission, requesting_origin.GetURL())));
  }

  Result GetPermissionResultForCurrentDocument(
      const Descriptor& permission,
      content::RenderFrameHost* frame,
      bool should_include_device_status) override {
    return Result(TraceDecision(
        "document", permission,
        frame->IsActive() && !frame->IsNestedWithinFencedFrame()
            ? Query(permission, frame->GetLastCommittedOrigin().GetURL())
            : Status::DENIED));
  }

  Result GetPermissionResultForWorker(const Descriptor& permission,
                                      content::RenderProcessHost* process,
                                      const GURL& worker_origin) override {
    return Result(TraceDecision("worker", permission, Status::DENIED));
  }

  Result GetPermissionResultForEmbeddedRequester(
      const Descriptor& permission,
      content::RenderFrameHost* frame,
      const url::Origin& requesting_origin) override {
    return Result(
        TraceDecision("embedded_requester", permission, Status::DENIED));
  }

  void ResetPermission(blink::PermissionType permission,
                       const GURL& requesting_origin,
                       const GURL& embedding_origin) override {}

 private:
  static Status Query(const Descriptor& permission, const GURL& origin) {
    if (!permission || !network::IsUrlPotentiallyTrustworthy(origin) ||
        permission->name != blink::mojom::PermissionName::CLIPBOARD_WRITE ||
        !permission->extension || !permission->extension->is_clipboard() ||
        !permission->extension->get_clipboard()->will_be_sanitized) {
      return Status::DENIED;
    }
    return Status::ASK;
  }
};
}  // namespace

PermissionServiceFactory* PermissionServiceFactory::Get() {
  static base::NoDestructor<PermissionServiceFactory> instance;
  return instance.get();
}
PermissionServiceFactory::PermissionServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffitePermission",
          BrowserContextDependencyManager::GetInstance()) {}
PermissionService* PermissionServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<PermissionService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
PermissionServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<PermissionServiceImpl>();
}
}  // namespace tuffite
