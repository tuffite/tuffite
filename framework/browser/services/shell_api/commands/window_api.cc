// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/window_api.h"

#include <cmath>
#include <limits>
#include <optional>
#include <utility>

#include "base/functional/bind.h"
#include "base/location.h"
#include "base/notreached.h"
#include "base/strings/utf_string_conversions.h"
#include "base/task/sequenced_task_runner.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
#include "tuffite/framework/browser/services/window/window_service.h"
#include "tuffite/framework/browser/services/window/window_service_factory.h"
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

rpc::Status ToRpcStatus(::tuffite::window::Status status) {
  switch (status) {
    case ::tuffite::window::Status::kOk:
      return kStatusOk;
    case ::tuffite::window::Status::kInvalidArgument:
      return kStatusInvalidArgument;
    case ::tuffite::window::Status::kAccessDenied:
      return kStatusAccessDenied;
    case ::tuffite::window::Status::kUnavailable:
      return kStatusUnavailable;
  }
  NOTREACHED();
}

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

WindowInfo FirstInfo(const ::tuffite::window::Result& result) {
  return result.windows.empty() ? WindowInfo() : ToInfo(result.windows.front());
}

std::vector<WindowInfo> AllInfo(const ::tuffite::window::Result& result) {
  std::vector<WindowInfo> output;
  output.reserve(result.windows.size());
  for (const auto& info : result.windows) {
    output.push_back(ToInfo(info));
  }
  return output;
}

template <typename Callback, typename Convert>
void Invoke(content::RenderFrameHost& frame,
            ::tuffite::window::Command command,
            Callback callback,
            Convert convert) {
  auto* service = ::tuffite::window::WindowServiceFactory::GetForBrowserContext(
      frame.GetBrowserContext());
  auto weak_service = service->GetWeakPtr();
  ::tuffite::window::Result result = service->Invoke(
      command, content::WebContents::FromRenderFrameHost(&frame));
  const rpc::Status status = ToRpcStatus(result.status);
  const std::optional<int64_t> close_after_reply = result.close_after_reply;
  std::move(callback).Run(status, convert(result));
  if (close_after_reply) {
    base::SequencedTaskRunner::GetCurrentDefault()->PostTask(
        FROM_HERE,
        base::BindOnce(
            [](base::WeakPtr<::tuffite::window::WindowService> service,
               int64_t id) {
              if (service) {
                service->CloseById(id);
              }
            },
            weak_service, *close_after_reply));
  }
}

::tuffite::window::Command TargetCommand(::tuffite::window::Operation operation,
                                         std::optional<WindowId> id) {
  return {.operation = operation, .target = id};
}

}  // namespace

void Current(const ::tuffite::dispatch::Context& context,
             CurrentRequest&&,
             CurrentCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  Invoke(context.frame(), {.operation = ::tuffite::window::Operation::kCurrent},
         std::move(callback), &FirstInfo);
}

void List(const ::tuffite::dispatch::Context& context,
          ListRequest&&,
          ListCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  Invoke(context.frame(), {.operation = ::tuffite::window::Operation::kList},
         std::move(callback), &AllInfo);
}

void Create(const ::tuffite::dispatch::Context& context,
            CreateRequest&& request,
            CreateCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  ::tuffite::window::Command command{.operation =
                                         ::tuffite::window::Operation::kCreate};
  if (request.options) {
    command.create.label = std::move(request.options->label);
    if (request.options->url) {
      command.create.url = GURL(*request.options->url);
    }
    command.create.title = std::move(request.options->title);
    command.create.focused = request.options->focused;
    if (request.options->bounds) {
      command.create.bounds = ToRect(*request.options->bounds);
      if (!command.create.bounds) {
        std::move(callback).Run(kStatusInvalidArgument, {});
        return;
      }
    }
  }
  Invoke(context.frame(), std::move(command), std::move(callback), &FirstInfo);
}

void Focus(const ::tuffite::dispatch::Context& context,
           FocusRequest&& request,
           FocusCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  Invoke(context.frame(),
         TargetCommand(::tuffite::window::Operation::kFocus, request.id),
         std::move(callback), &FirstInfo);
}

void Minimize(const ::tuffite::dispatch::Context& context,
              MinimizeRequest&& request,
              MinimizeCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  Invoke(context.frame(),
         TargetCommand(::tuffite::window::Operation::kMinimize, request.id),
         std::move(callback), &FirstInfo);
}

void Maximize(const ::tuffite::dispatch::Context& context,
              MaximizeRequest&& request,
              MaximizeCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  Invoke(context.frame(),
         TargetCommand(::tuffite::window::Operation::kMaximize, request.id),
         std::move(callback), &FirstInfo);
}

void Restore(const ::tuffite::dispatch::Context& context,
             RestoreRequest&& request,
             RestoreCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  Invoke(context.frame(),
         TargetCommand(::tuffite::window::Operation::kRestore, request.id),
         std::move(callback), &FirstInfo);
}

void SetFullscreen(const ::tuffite::dispatch::Context& context,
                   SetFullscreenRequest&& request,
                   SetFullscreenCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto command =
      TargetCommand(::tuffite::window::Operation::kSetFullscreen, request.id);
  command.fullscreen = request.fullscreen;
  Invoke(context.frame(), std::move(command), std::move(callback), &FirstInfo);
}

void Close(const ::tuffite::dispatch::Context& context,
           CloseRequest&& request,
           CloseCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  Invoke(context.frame(),
         TargetCommand(::tuffite::window::Operation::kClose, request.id),
         std::move(callback),
         [](const ::tuffite::window::Result&) { return std::monostate(); });
}

void SetTitle(const ::tuffite::dispatch::Context& context,
              SetTitleRequest&& request,
              SetTitleCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  if (request.title.size() > kMaxTitleBytes) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto command =
      TargetCommand(::tuffite::window::Operation::kSetTitle, request.id);
  command.title = std::move(request.title);
  Invoke(context.frame(), std::move(command), std::move(callback), &FirstInfo);
}

void GetBounds(const ::tuffite::dispatch::Context& context,
               GetBoundsRequest&& request,
               GetBoundsCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  Invoke(context.frame(),
         TargetCommand(::tuffite::window::Operation::kGetBounds, request.id),
         std::move(callback), [](const ::tuffite::window::Result& result) {
           return result.bounds ? ToBounds(*result.bounds) : WindowBounds();
         });
}

void SetBounds(const ::tuffite::dispatch::Context& context,
               SetBoundsRequest&& request,
               SetBoundsCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto bounds = ToRect(request.bounds);
  if (!bounds) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto command =
      TargetCommand(::tuffite::window::Operation::kSetBounds, request.id);
  command.bounds = *bounds;
  Invoke(context.frame(), std::move(command), std::move(callback), &FirstInfo);
}

}  // namespace tuffite::shell_api::window
