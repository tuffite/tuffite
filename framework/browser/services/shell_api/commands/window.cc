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
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "tuffite/framework/browser/services/window/window_service.h"
#include "tuffite/framework/browser/services/window/window_service_factory.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "url/gurl.h"
namespace tuffite::shell_api::window {
using generated::WindowBounds;
using WindowId = int64_t;
using generated::WindowInfo;
using CloseCallback = generated::Window::CloseCallback;
using CloseRequestPtr = generated::Window::CloseRequestPtr;
using CreateCallback = generated::Window::CreateCallback;
using CreateRequestPtr = generated::Window::CreateRequestPtr;
using CurrentCallback = generated::Window::CurrentCallback;
using CurrentRequestPtr = generated::Window::CurrentRequestPtr;
using FocusCallback = generated::Window::FocusCallback;
using FocusRequestPtr = generated::Window::FocusRequestPtr;
using GetBoundsCallback = generated::Window::GetBoundsCallback;
using GetBoundsRequestPtr = generated::Window::GetBoundsRequestPtr;
using ListCallback = generated::Window::ListCallback;
using ListRequestPtr = generated::Window::ListRequestPtr;
using MaximizeCallback = generated::Window::MaximizeCallback;
using MaximizeRequestPtr = generated::Window::MaximizeRequestPtr;
using MinimizeCallback = generated::Window::MinimizeCallback;
using MinimizeRequestPtr = generated::Window::MinimizeRequestPtr;
using RestoreCallback = generated::Window::RestoreCallback;
using RestoreRequestPtr = generated::Window::RestoreRequestPtr;
using SetBoundsCallback = generated::Window::SetBoundsCallback;
using SetBoundsRequestPtr = generated::Window::SetBoundsRequestPtr;
using SetFullscreenCallback = generated::Window::SetFullscreenCallback;
using SetFullscreenRequestPtr = generated::Window::SetFullscreenRequestPtr;
using SetTitleCallback = generated::Window::SetTitleCallback;
using SetTitleRequestPtr = generated::Window::SetTitleRequestPtr;
namespace {
constexpr size_t kMaxTitleBytes = 4096;
generated::WindowBoundsPtr ToBounds(const gfx::Rect& bounds) {
  return WindowBounds::New(bounds.x(), bounds.y(), bounds.width(),
                           bounds.height());
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

generated::WindowInfoPtr ToInfo(const ::tuffite::window::Info& info) {
  return WindowInfo::New(
      info.id, info.label, base::UTF16ToUTF8(info.state.title), info.url,
      ToBounds(info.state.bounds), info.state.focused, info.state.minimized,
      info.state.maximized, info.state.fullscreen);
}

struct WindowCall {
  raw_ptr<::tuffite::window::WindowService> service;
  raw_ptr<Window> current;
  raw_ptr<Window> target;
  raw_ptr<const ManifestService> manifest;
  url::Origin origin;
};
// Shared authorization and serialization, without dispatching native
// operations.
template <typename Callback>
std::optional<WindowCall> Authorize(const ::tuffite::dispatch::Context& context,
                                    std::string_view command,
                                    std::optional<WindowId> id,
                                    Callback& callback,
                                    bool check_scope = true) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
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
  if (!service->GetInfo(current) ||
      !manifest->IsShellApiCommandAllowed(origin, command)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return std::nullopt;
  }
  auto* target = id ? service->FindById(*id) : current;
  if (!target) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return std::nullopt;
  }
  if (!manifest->IsShellApiScopeAllowed(
          origin, command, target == current ? "current" : "others",
          check_scope ? "window" : "")) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return std::nullopt;
  }
  return WindowCall{service, current, target, manifest, std::move(origin)};
}
template <typename Callback>
void ReplyInfo(const WindowCall& call,
               Callback callback,
               bool succeeded = true) {
  auto info = succeeded ? call.service->GetInfo(call.target) : std::nullopt;
  std::move(callback).Run(info ? rpc::Status::kOk : rpc::Status::kUnavailable,
                          info ? ToInfo(*info) : nullptr);
}
}  // namespace
void WindowImpl::Current(const dispatch::Context& context,
                         CurrentRequestPtr,
                         CurrentCallback callback) {
  if (auto call = Authorize(context, generated::Window::kCurrent, {}, callback,
                            false)) {
    ReplyInfo(*call, std::move(callback));
  }
}
void WindowImpl::List(const dispatch::Context& context,
                      ListRequestPtr,
                      ListCallback callback) {
  if (auto call =
          Authorize(context, generated::Window::kList, {}, callback, false)) {
    std::vector<generated::WindowInfoPtr> result;
    for (const auto& info : call->service->List()) {
      result.push_back(ToInfo(info));
    }
    std::move(callback).Run(rpc::Status::kOk, std::move(result));
  }
}
void WindowImpl::Create(const dispatch::Context& context,
                        CreateRequestPtr request,
                        CreateCallback callback) {
  auto call =
      Authorize(context, generated::Window::kCreate, {}, callback, false);
  if (!call) {
    return;
  }
  const auto& options = request->options;
  std::optional<gfx::Rect> bounds;
  if (options && options->bounds) {
    bounds = ToRect(*options->bounds);
    if (!bounds) {
      rpc::Reject(callback, rpc::Status::kInvalidArgument);
      return;
    }
    if (!::tuffite::window::Constraints{}.Contains(*bounds)) {
      rpc::Reject(callback, rpc::Status::kAccessDenied);
      return;
    }
  }
  if (options && options->label) {
    bool duplicate = options->label->empty();
    for (const auto& info : call->service->List()) {
      duplicate |= info.label == *options->label;
    }
    if (duplicate) {
      rpc::Reject(callback, rpc::Status::kInvalidArgument);
      return;
    }
  }
  const GURL url = options && options->url
                       ? GURL(*options->url)
                       : call->manifest->app_info().entry_url;
  if (!call->manifest->IsWindowUrlAllowed(call->origin, url)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  ::tuffite::Window* created =
      call->service->Create(bounds ? bounds->size() : gfx::Size());
  if (!created) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
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
    rpc::Reject(callback, rpc::Status::kUnavailable);
    return;
  }
  if (options && !options->focused.value_or(true)) {
    call->service->Focus(call->current);
  }
  call->target = created;
  ReplyInfo(*call, std::move(callback));
}
void WindowImpl::Focus(const dispatch::Context& context,
                       FocusRequestPtr request,
                       FocusCallback callback) {
  if (auto call = Authorize(context, generated::Window::kFocus, request->id,
                            callback)) {
    const bool succeeded = call->service->Focus(call->target);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void WindowImpl::Minimize(const dispatch::Context& context,
                          MinimizeRequestPtr request,
                          MinimizeCallback callback) {
  if (auto call = Authorize(context, generated::Window::kMinimize, request->id,
                            callback)) {
    const bool succeeded = call->service->Minimize(call->target);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void WindowImpl::Maximize(const dispatch::Context& context,
                          MaximizeRequestPtr request,
                          MaximizeCallback callback) {
  if (auto call = Authorize(context, generated::Window::kMaximize, request->id,
                            callback)) {
    const bool succeeded = call->service->Maximize(call->target);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void WindowImpl::Restore(const dispatch::Context& context,
                         RestoreRequestPtr request,
                         RestoreCallback callback) {
  if (auto call = Authorize(context, generated::Window::kRestore, request->id,
                            callback)) {
    const bool succeeded = call->service->Restore(call->target);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void WindowImpl::SetFullscreen(const dispatch::Context& context,
                               SetFullscreenRequestPtr request,
                               SetFullscreenCallback callback) {
  if (auto call = Authorize(context, generated::Window::kSetFullscreen,
                            request->id, callback)) {
    const bool succeeded =
        call->service->SetFullscreen(call->target, request->fullscreen);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void WindowImpl::Close(const dispatch::Context& context,
                       CloseRequestPtr request,
                       CloseCallback callback) {
  if (auto call = Authorize(context, generated::Window::kClose, request->id,
                            callback)) {
    const auto info = call->service->GetInfo(call->target);
    if (!info) {
      rpc::Reject(callback, rpc::Status::kUnavailable);
      return;
    }
    auto service = call->service->GetWeakPtr();
    std::move(callback).Run(rpc::Status::kOk);
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
void WindowImpl::SetTitle(const dispatch::Context& context,
                          SetTitleRequestPtr request,
                          SetTitleCallback callback) {
  if (request->title.size() > kMaxTitleBytes) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (auto call = Authorize(context, generated::Window::kSetTitle, request->id,
                            callback)) {
    const bool succeeded = call->service->SetTitle(
        call->target, base::UTF8ToUTF16(request->title));
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
void WindowImpl::GetBounds(const dispatch::Context& context,
                           GetBoundsRequestPtr request,
                           GetBoundsCallback callback) {
  if (auto call = Authorize(context, generated::Window::kGetBounds, request->id,
                            callback)) {
    auto info = call->service->GetInfo(call->target);
    std::move(callback).Run(info ? rpc::Status::kOk : rpc::Status::kUnavailable,
                            info ? ToBounds(info->state.bounds) : nullptr);
  }
}
void WindowImpl::SetBounds(const dispatch::Context& context,
                           SetBoundsRequestPtr request,
                           SetBoundsCallback callback) {
  auto bounds = ToRect(*request->bounds);
  if (!bounds) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (auto call = Authorize(context, generated::Window::kSetBounds, request->id,
                            callback)) {
    if (!call->target->constraints().Contains(*bounds)) {
      rpc::Reject(callback, rpc::Status::kAccessDenied);
      return;
    }
    const bool succeeded = call->service->SetBounds(call->target, *bounds);
    ReplyInfo(*call, std::move(callback), succeeded);
  }
}
}  // namespace tuffite::shell_api::window
