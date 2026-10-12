// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/shell_api/commands/window.h"

#include <cmath>
#include <limits>
#include <optional>
#include <string>
#include <utility>

#include "base/functional/bind.h"
#include "base/location.h"
#include "base/memory/raw_ptr.h"
#include "base/strings/utf_string_conversions.h"
#include "base/task/sequenced_task_runner.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/browser/services/shell_api/commands/application_policy.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "tuffite/framework/browser/services/window/window_service.h"
#include "tuffite/framework/browser/services/window/window_service_factory.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/common/app_manifest.h"
#include "url/gurl.h"
namespace tuffite::shell_api::window {
using generated::WindowBounds;
using generated::WindowId;
using generated::WindowInfo;
using generated::window::CloseCallback;
using generated::window::CloseRequest;
using generated::window::CreateCallback;
using generated::window::CreateRequest;
using generated::window::CurrentCallback;
using generated::window::CurrentRequest;
using generated::window::FocusCallback;
using generated::window::FocusRequest;
using generated::window::GetBoundsCallback;
using generated::window::GetBoundsRequest;
using generated::window::ListCallback;
using generated::window::ListRequest;
using generated::window::MaximizeCallback;
using generated::window::MaximizeRequest;
using generated::window::MinimizeCallback;
using generated::window::MinimizeRequest;
using generated::window::RestoreCallback;
using generated::window::RestoreRequest;
using generated::window::SetBoundsCallback;
using generated::window::SetBoundsRequest;
using generated::window::SetFullscreenCallback;
using generated::window::SetFullscreenRequest;
using generated::window::SetTitleCallback;
using generated::window::SetTitleRequest;
namespace {
constexpr size_t kMaxTitleBytes = 4096;
WindowBounds ToBounds(const gfx::Rect& bounds) {
  return {
      .x = static_cast<double>(bounds.x()),
      .y = static_cast<double>(bounds.y()),
      .width = static_cast<double>(bounds.width()),
      .height = static_cast<double>(bounds.height()),
  };
}

bool ToCoordinate(double value, int* result) {
  if (!std::isfinite(value) || std::trunc(value) != value ||
      value < std::numeric_limits<int>::min() ||
      value > std::numeric_limits<int>::max()) {
    return false;
  }
  *result = static_cast<int>(value);
  return true;
}

std::optional<gfx::Rect> ToRect(const WindowBounds& bounds) {
  int x = 0;
  int y = 0;
  int width = 0;
  int height = 0;
  if (!ToCoordinate(bounds.x, &x) || !ToCoordinate(bounds.y, &y) ||
      !ToCoordinate(bounds.width, &width) ||
      !ToCoordinate(bounds.height, &height)) {
    return std::nullopt;
  }
  return gfx::Rect(x, y, width, height);
}

WindowInfo ToInfo(const ::tuffite::window::Info& info) {
  return {
      .id = info.id,
      .label = info.label,
      .title = base::UTF16ToUTF8(info.state.title),
      .url = info.url,
      .bounds = ToBounds(info.state.bounds),
      .focused = info.state.focused,
      .minimized = info.state.minimized,
      .maximized = info.state.maximized,
      .fullscreen = info.state.fullscreen,
  };
}

struct WindowCall {
  raw_ptr<::tuffite::window::WindowService> service;
  raw_ptr<Window> current;
  raw_ptr<Window> target;
  raw_ptr<const AppManifest> manifest;
  url::Origin origin;
};
// Shared authorization and serialization, without dispatching native
// operations.
template <typename Callback>
std::optional<WindowCall> Authorize(const ::tuffite::dispatch::Context& context,
                                    std::string_view method,
                                    std::optional<WindowId> id,
                                    Callback& callback,
                                    bool check_scope = true) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return std::nullopt;
  }
  auto& frame = context.frame();
  auto* service = ::tuffite::window::WindowServiceFactory::GetForBrowserContext(
      frame.GetBrowserContext());
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  auto* current =
      contents ? Window::FromWebContents(contents->GetOutermostWebContents())
               : nullptr;
  auto* manifest = ManifestFor(frame);
  auto origin = frame.GetLastCommittedOrigin();
  const std::string command = "tuffite.window." + std::string(method);
  if (!service->GetInfo(current) ||
      !manifest->IsShellApiCommandAllowed(origin, command)) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return std::nullopt;
  }
  auto* target = id ? service->FindById(*id) : current;
  if (!target) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return std::nullopt;
  }
  if (check_scope &&
      !manifest->IsShellApiScopeAllowed(
          origin, command, target == current ? "current" : "others")) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return std::nullopt;
  }
  return WindowCall{service, current, target, manifest, std::move(origin)};
}
template <typename Callback>
void ReplyInfo(const WindowCall& call,
               Callback callback,
               bool succeeded = true) {
  auto info = succeeded ? call.service->GetInfo(call.target) : std::nullopt;
  std::move(callback).Run(info ? kStatusOk : kStatusUnavailable,
                          info ? ToInfo(*info) : WindowInfo());
}
}  // namespace
void Current(const ::tuffite::dispatch::Context& context,
             CurrentRequest&&,
             CurrentCallback&& callback) {
  if (auto call = Authorize(context, "current", {}, callback, false)) {
    ReplyInfo(*call, std::move(callback));
  }
}
void List(const ::tuffite::dispatch::Context& context,
          ListRequest&&,
          ListCallback&& callback) {
  if (auto call = Authorize(context, "list", {}, callback, false)) {
    std::vector<WindowInfo> result;
    for (const auto& info : call->service->List()) {
      result.push_back(ToInfo(info));
    }
    std::move(callback).Run(kStatusOk, std::move(result));
  }
}
void Create(const ::tuffite::dispatch::Context& context,
            CreateRequest&& request,
            CreateCallback&& callback) {
  auto call = Authorize(context, "create", {}, callback, false);
  if (!call) {
    return;
  }
  const auto& options = request.options;
  std::optional<gfx::Rect> bounds;
  if (options && options->bounds) {
    bounds = ToRect(*options->bounds);
    if (!bounds) {
      std::move(callback).Run(kStatusInvalidArgument, {});
      return;
    }
    if (!::tuffite::window::Constraints{}.Contains(*bounds)) {
      std::move(callback).Run(kStatusAccessDenied, {});
      return;
    }
  }
  if (options && options->label) {
    bool duplicate = options->label->empty();
    for (const auto& info : call->service->List()) {
      duplicate |= info.label == *options->label;
    }
    if (duplicate) {
      std::move(callback).Run(kStatusInvalidArgument, {});
      return;
    }
  }
  const GURL url = options && options->url ? GURL(*options->url)
                                           : call->manifest->entry_url();
  if (!call->manifest->IsWindowUrlAllowed(call->origin, url)) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  Window* created =
      call->service->Create(bounds ? bounds->size() : gfx::Size());
  if (!created) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  WebContentsRoleData::SetRole(created->web_contents(), WebContentsRole::kApp);
  if (options && options->label) {
    call->service->Register(created, *options->label);
  }
  created->LoadURL(url);
  if ((bounds && !call->service->SetBounds(created, *bounds)) ||
      (options && options->title &&
       !call->service->SetTitle(created, base::UTF8ToUTF16(*options->title))) ||
      ((!options || options->focused.value_or(true)) &&
       !call->service->Focus(created))) {
    call->service->Close(created);
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  if (options && !options->focused.value_or(true)) {
    call->service->Focus(call->current);
  }
  call->target = created;
  ReplyInfo(*call, std::move(callback));
}
void Focus(const ::tuffite::dispatch::Context& context,
           FocusRequest&& request,
           FocusCallback&& callback) {
  if (auto call = Authorize(context, "focus", request.id, callback)) {
    const bool succeeded = call->service->Focus(call->target);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void Minimize(const ::tuffite::dispatch::Context& context,
              MinimizeRequest&& request,
              MinimizeCallback&& callback) {
  if (auto call = Authorize(context, "minimize", request.id, callback)) {
    const bool succeeded = call->service->Minimize(call->target);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void Maximize(const ::tuffite::dispatch::Context& context,
              MaximizeRequest&& request,
              MaximizeCallback&& callback) {
  if (auto call = Authorize(context, "maximize", request.id, callback)) {
    const bool succeeded = call->service->Maximize(call->target);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void Restore(const ::tuffite::dispatch::Context& context,
             RestoreRequest&& request,
             RestoreCallback&& callback) {
  if (auto call = Authorize(context, "restore", request.id, callback)) {
    const bool succeeded = call->service->Restore(call->target);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void SetFullscreen(const ::tuffite::dispatch::Context& context,
                   SetFullscreenRequest&& request,
                   SetFullscreenCallback&& callback) {
  if (auto call = Authorize(context, "setFullscreen", request.id, callback)) {
    const bool succeeded =
        call->service->SetFullscreen(call->target, request.fullscreen);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void Close(const ::tuffite::dispatch::Context& context,
           CloseRequest&& request,
           CloseCallback&& callback) {
  if (auto call = Authorize(context, "close", request.id, callback)) {
    const auto info = call->service->GetInfo(call->target);
    if (!info) {
      std::move(callback).Run(kStatusUnavailable, {});
      return;
    }
    auto service = call->service->GetWeakPtr();
    std::move(callback).Run(kStatusOk, {});
    // The transport must receive its reply before its owning WebContents
    // closes.
    base::SequencedTaskRunner::GetCurrentDefault()->PostTask(
        FROM_HERE,
        base::BindOnce(
            [](base::WeakPtr<::tuffite::window::WindowService> service,
               int64_t id) {
              if (service) {
                service->CloseById(id);
              }
            },
            service, info->id));
  }
}
void SetTitle(const ::tuffite::dispatch::Context& context,
              SetTitleRequest&& request,
              SetTitleCallback&& callback) {
  if (request.title.size() > kMaxTitleBytes) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  if (auto call = Authorize(context, "setTitle", request.id, callback)) {
    const bool succeeded =
        call->service->SetTitle(call->target, base::UTF8ToUTF16(request.title));
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void GetBounds(const ::tuffite::dispatch::Context& context,
               GetBoundsRequest&& request,
               GetBoundsCallback&& callback) {
  if (auto call = Authorize(context, "getBounds", request.id, callback)) {
    auto info = call->service->GetInfo(call->target);
    std::move(callback).Run(
        info ? kStatusOk : kStatusUnavailable,
        info ? ToBounds(info->state.bounds) : WindowBounds());
  }
}
void SetBounds(const ::tuffite::dispatch::Context& context,
               SetBoundsRequest&& request,
               SetBoundsCallback&& callback) {
  auto bounds = ToRect(request.bounds);
  if (!bounds) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  if (auto call = Authorize(context, "setBounds", request.id, callback)) {
    if (!call->target->constraints().Contains(*bounds)) {
      std::move(callback).Run(kStatusAccessDenied, {});
      return;
    }
    const bool succeeded = call->service->SetBounds(call->target, *bounds);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
}  // namespace tuffite::shell_api::window
