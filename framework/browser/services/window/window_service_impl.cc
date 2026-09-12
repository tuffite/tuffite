// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/window_service_impl.h"

#include <algorithm>
#include <array>
#include <bit>
#include <cmath>
#include <limits>
#include <optional>
#include <string>
#include <utility>

#include "base/check.h"
#include "base/check_op.h"
#include "base/functional/bind.h"
#include "base/numerics/safe_conversions.h"
#include "base/strings/string_util.h"
#include "base/strings/utf_string_conversions.h"
#include "base/task/single_thread_task_runner.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_observer.h"
#include "tuffite/framework/browser/services/window/platform.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "tuffite/framework/browser/services/window/window_backend.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/common/app_manifest.h"
#include "ui/gfx/geometry/size.h"
#include "url/gurl.h"

namespace tuffite::window {
base::WeakPtr<WindowService> WindowServiceImpl::GetWeakPtr() {
  return weak_factory_.GetWeakPtr();
}

namespace {

bool IsBoundsAllowed(const Window* window, const gfx::Rect& bounds) {
  return bounds.width() >= (window ? window->minimum_size().width() : 1) &&
         bounds.height() >= (window ? window->minimum_size().height() : 1) &&
         bounds.width() <= (window ? window->maximum_size().width() : 16384) &&
         bounds.height() <= (window ? window->maximum_size().height() : 16384);
}

Window* CurrentWindow(content::WebContents* caller) {
  return caller ? Window::FromWebContents(caller->GetOutermostWebContents())
                : nullptr;
}

Result Error(Status status) {
  return {.status = status};
}

}  // namespace

class StartupWindowsObserver final : public content::WebContentsObserver {
 public:
  StartupWindowsObserver(content::WebContents* primary, base::OnceClosure ready)
      : content::WebContentsObserver(primary), ready_(std::move(ready)) {}
  void DidFinishNavigation(content::NavigationHandle* navigation) override;

 private:
  base::OnceClosure ready_;
};

void StartupWindowsObserver::DidFinishNavigation(
    content::NavigationHandle* navigation) {
  if (ready_ && navigation->IsInPrimaryMainFrame() &&
      navigation->HasCommitted() && !navigation->IsErrorPage()) {
    std::move(ready_).Run();
  }
}

Window* WindowServiceImpl::CreateConfiguredWindow(const AppWindow& configured,
                                                  const gfx::Size& size) {
  Window* window = Window::Create(&*context_, GURL(), size,
                                  configured.decorations, &configured);
  CHECK(window);
  WebContentsRoleData::SetRole(window->web_contents(), WebContentsRole::kApp);
  Register(window, configured.label);
  if (!configured.title.empty()) {
    CHECK(SetTitle(window, base::UTF8ToUTF16(configured.title)));
  }
  if (configured.x && configured.y) {
    CHECK(SetBounds(window, gfx::Rect(*configured.x, *configured.y,
                                      size.width(), size.height())));
  }
  if (configured.focused) {
    CHECK(Focus(window));
  }
  return window;
}

Window* WindowServiceImpl::CreateStartupWindows(const gfx::Size& primary_size) {
  CHECK(!stopped_ && !startup_created_);
  startup_created_ = true;
  const auto& primary = manifest_->windows().front();
  Window* window = CreateConfiguredWindow(
      primary, primary_size.IsEmpty() ? gfx::Size(primary.width, primary.height)
                                      : primary_size);
  if (manifest_->windows().size() > 1) {
    startup_windows_observer_ = std::make_unique<StartupWindowsObserver>(
        window->web_contents(),
        base::BindOnce(&WindowServiceImpl::CreateSecondaryWindows,
                       weak_factory_.GetWeakPtr()));
  }
  return window;
}

void WindowServiceImpl::CreateSecondaryWindows() {
  for (size_t i = 1; i < manifest_->windows().size(); ++i) {
    const auto& configured = manifest_->windows()[i];
    CreateConfiguredWindow(configured,
                           gfx::Size(configured.width, configured.height))
        ->LoadURL(configured.url);
  }
}

WindowServiceImpl::WindowServiceImpl(content::BrowserContext& context,
                                     const AppManifest& manifest,
                                     std::unique_ptr<WindowBackend> backend)
    : context_(context), manifest_(manifest), backend_(std::move(backend)) {}
WindowServiceImpl::~WindowServiceImpl() {
  Shutdown();
}
void WindowServiceImpl::Shutdown() {
  if (stopped_) {
    return;
  }
  stopped_ = true;
  startup_windows_observer_.reset();
  weak_factory_.InvalidateWeakPtrs();
  while (!windows_.empty()) {
    auto window = std::move(windows_.back());
    windows_.pop_back();
    window->PrepareForShutdown();
  }
  entries_.clear();
  closing_.clear();
}
void WindowServiceImpl::Adopt(std::unique_ptr<Window> window) {
  CHECK(!stopped_);
  CHECK_EQ(window->web_contents()->GetBrowserContext(), &*context_);
  IdFor(window.get());
  windows_.push_back(std::move(window));
}
void WindowServiceImpl::DestroyWhenIdle(Window* window) {
  if (stopped_ || closing_.contains(window)) {
    return;
  }
  Unregister(window);
  closing_.insert(window);
  // Retain ownership until the task runs. If the loop stops first, Shutdown()
  // still destroys this window and its WebContents before the BrowserContext.
  base::SingleThreadTaskRunner::GetCurrentDefault()->PostTask(
      FROM_HERE, base::BindOnce(&WindowServiceImpl::DestroyWindow,
                                weak_factory_.GetWeakPtr(), window));
}
void WindowServiceImpl::DestroyWindow(Window* window) {
  auto found = std::ranges::find(windows_, window,
                                 [](const auto& value) { return value.get(); });
  if (found == windows_.end()) {
    return;
  }
  auto owned = std::move(*found);
  windows_.erase(found);
}
void WindowServiceImpl::Unregister(Window* window) {
  closing_.erase(window);
  std::erase_if(entries_,
                [window](const Entry& e) { return e.window == window; });
}

void WindowServiceImpl::Reconcile() {
  // Registration and release are explicit; no process-global discovery.
  for (const auto& window : windows_) {
    if (!closing_.contains(window.get())) {
      IdFor(window.get());
    }
  }
}

int64_t WindowServiceImpl::IdFor(Window* window, std::string label) {
  auto found = std::ranges::find_if(entries_, [window](const Entry& entry) {
    return entry.window == window;
  });
  if (found != entries_.end()) {
    return found->id;
  }
  CHECK_GT(next_id_, 0);
  const int64_t id = next_id_++;
  entries_.push_back({id, window, std::move(label)});
  return id;
}

void WindowServiceImpl::Register(Window* window, std::string label) {
  CHECK(window);
  CHECK(!label.empty());
  Reconcile();
  CHECK(std::ranges::none_of(entries_, [&](const Entry& entry) {
    return entry.label == label && entry.window != window;
  }));
  auto entry = std::ranges::find_if(entries_, [window](const Entry& current) {
    return current.window == window;
  });
  if (entry == entries_.end()) {
    IdFor(window, std::move(label));
  } else if (entry->label.empty()) {
    entry->label = std::move(label);
  }
}

Window* WindowServiceImpl::FindById(int64_t id) {
  auto found = std::ranges::find(entries_, id, &Entry::id);
  if (found == entries_.end()) {
    return nullptr;
  }
  Window* window = found->window.get();
  return std::ranges::find(Window::windows(), window) == Window::windows().end()
             ? nullptr
             : window;
}

void WindowServiceImpl::CloseById(int64_t id) {
  Reconcile();
  if (Window* window = FindById(id)) {
    backend_->Close(*window);
  }
}

Result WindowServiceImpl::Invoke(const Command& command,
                                 content::WebContents* caller) {
  const auto& manifest = *manifest_;
  Window* current = CurrentWindow(caller);
  if (stopped_ || !current || closing_.contains(current) ||
      caller->GetBrowserContext() != &*context_) {
    return Error(Status::kAccessDenied);
  }
  const auto origin = caller->GetPrimaryMainFrame()->GetLastCommittedOrigin();
  static constexpr std::array<std::string_view, 12> methods = {
      "current",  "list",     "create",    "focus",
      "minimize", "maximize", "restore",   "setFullscreen",
      "close",    "setTitle", "getBounds", "setBounds"};
  const std::string method =
      "tuffite.window." +
      std::string(methods.at(static_cast<size_t>(command.operation)));
  if (!manifest.IsShellApiCommandAllowed(origin, method)) {
    return Error(Status::kAccessDenied);
  }
  auto info = [this](Window* window) {
    const auto id = IdFor(window);
    auto entry = std::ranges::find(entries_, id, &Entry::id);
    return Info{id, entry->label,
                window->web_contents()->GetLastCommittedURL().spec(),
                backend_->Read(*window)};
  };
  auto success = [&](Window* window) {
    return Result{.status = Status::kOk, .windows = {info(window)}};
  };
  if (command.operation == Operation::kCurrent) {
    return success(current);
  }
  if (command.operation == Operation::kList) {
    Result result{.status = Status::kOk};
    for (const auto& entry : entries_) {
      result.windows.push_back(info(entry.window));
    }
    return result;
  }
  if (command.operation == Operation::kCreate) {
    const auto& options = command.create;
    if (options.label && (options.label->empty() ||
                          std::ranges::any_of(entries_, [&](const Entry& e) {
                            return e.label == *options.label;
                          }))) {
      return Error(Status::kInvalidArgument);
    }
    auto url = options.url.value_or(manifest.entry_url());
    if (!manifest.IsWindowUrlAllowed(origin, url) ||
        (options.bounds && !IsBoundsAllowed(nullptr, *options.bounds))) {
      return Error(Status::kAccessDenied);
    }
    Window* window = backend_->Create(
        *context_, options.bounds ? options.bounds->size() : gfx::Size());
    if (!window) {
      return Error(Status::kUnavailable);
    }
    WebContentsRoleData::SetRole(window->web_contents(), WebContentsRole::kApp);
    if (options.label) {
      Register(window, *options.label);
    }
    window->LoadURL(url);
    if ((options.bounds &&
         !backend_->Apply(*window, Command{.operation = Operation::kSetBounds,
                                           .bounds = *options.bounds})) ||
        (options.title &&
         !backend_->Apply(*window, Command{.operation = Operation::kSetTitle,
                                           .title = *options.title})) ||
        (options.focused.value_or(true) &&
         !backend_->Apply(*window, Command{.operation = Operation::kFocus}))) {
      backend_->Close(*window);
      return Error(Status::kUnavailable);
    }
    if (!options.focused.value_or(true)) {
      backend_->Apply(*current, Command{.operation = Operation::kFocus});
    }
    return success(window);
  }
  Window* target = command.target ? FindById(*command.target) : current;
  if (!target) {
    return Error(Status::kInvalidArgument);
  }
  if (!manifest.IsShellApiScopeAllowed(
          origin, method, target == current ? "current" : "others")) {
    return Error(Status::kAccessDenied);
  }
  switch (command.operation) {
    case Operation::kClose:
      return {.status = Status::kOk, .close_after_reply = IdFor(target)};
    case Operation::kGetBounds:
      return {.status = Status::kOk, .bounds = backend_->Read(*target).bounds};
    case Operation::kSetBounds:
      if (!IsBoundsAllowed(target, command.bounds)) {
        return Error(Status::kAccessDenied);
      }
      break;
    case Operation::kFocus:
    case Operation::kMinimize:
    case Operation::kMaximize:
    case Operation::kRestore:
    case Operation::kSetFullscreen:
    case Operation::kSetTitle:
      break;
    default:
      return Error(Status::kInvalidArgument);
  }
  const bool succeeded = backend_->Apply(*target, command);
  return succeeded ? success(target) : Error(Status::kUnavailable);
}
}  // namespace tuffite::window
