// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/window_service_impl.h"

#include <algorithm>
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
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_observer.h"
#include "tuffite/framework/browser/services/window/platform.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "ui/gfx/geometry/size.h"
#include "url/gurl.h"

namespace tuffite::window {
base::WeakPtr<WindowService> WindowServiceImpl::GetWeakPtr() {
  return weak_factory_.GetWeakPtr();
}

// Owned by WindowService, which detaches this observer before closing windows.
class StartupWindowsObserver final : public content::WebContentsObserver {
 public:
  StartupWindowsObserver(content::WebContents* primary,
                         base::OnceClosure ready,
                         base::OnceClosure closed)
      : content::WebContentsObserver(primary),
        ready_(std::move(ready)),
        closed_(std::move(closed)) {}
  void DidFinishNavigation(content::NavigationHandle* navigation) override {
    if (ready_ && navigation->IsInPrimaryMainFrame() &&
        navigation->HasCommitted() && !navigation->IsErrorPage()) {
      std::move(ready_).Run();
    }
  }
  void WebContentsDestroyed() override {
    ready_.Reset();
    if (closed_) {
      std::move(closed_).Run();
    }
  }

 private:
  base::OnceClosure ready_;
  base::OnceClosure closed_;
};

Window* WindowServiceImpl::CreateConfiguredWindow(const WindowOptions& options,
                                                  const gfx::Size& size) {
  Window* window = Create(size, options.decorations, options.constraints);
  CHECK(window);
  if (options.role) {
    WebContentsRoleData::SetRole(window->web_contents(), *options.role);
  }
  if (!options.label.empty()) {
    Register(window, options.label);
  }
  if (!options.title.empty()) {
    CHECK(SetTitle(window, options.title));
  }
  if (options.position) {
    CHECK(SetBounds(window, gfx::Rect(*options.position, size)));
  }
  if (options.focused) {
    CHECK(Focus(window));
  }
  return window;
}
Window* WindowServiceImpl::CreateStartupWindows(
    std::vector<WindowOptions> windows,
    const gfx::Size& primary_size) {
  CHECK(!stopped_ && !startup_created_ && !windows.empty());
  startup_created_ = true;
  auto primary = std::move(windows.front());
  windows.erase(windows.begin());
  Window* window = CreateConfiguredWindow(
      primary, primary_size.IsEmpty() ? primary.size : primary_size);
  startup_primary_id_ = GetInfo(window)->id;
  startup_windows_ = std::move(windows);
  if (!startup_windows_.empty()) {
    startup_windows_observer_ = std::make_unique<StartupWindowsObserver>(
        window->web_contents(),
        base::BindOnce(&WindowServiceImpl::CreateSecondaryWindows,
                       weak_factory_.GetWeakPtr()),
        base::BindOnce(&WindowServiceImpl::CancelStartupWindows,
                       weak_factory_.GetWeakPtr()));
  }
  // Retain an ID and a weak service, never a raw window in a posted callback.
  // This also leaves the caller time to install smoke/application observers.
  if (!primary.url.is_empty()) {
    base::SingleThreadTaskRunner::GetCurrentDefault()->PostTask(
        FROM_HERE,
        base::BindOnce(
            [](base::WeakPtr<WindowServiceImpl> service, int64_t id, GURL url) {
              if (service) {
                if (auto* window = service->FindById(id)) {
                  window->LoadURL(url);
                }
              }
            },
            weak_factory_.GetWeakPtr(), *startup_primary_id_,
            std::move(primary.url)));
  }
  return window;
}
void WindowServiceImpl::CreateSecondaryWindows() {
  if (stopped_ || !startup_primary_id_ || !FindById(*startup_primary_id_)) {
    CancelStartupWindows();
    return;
  }
  auto windows = std::move(startup_windows_);
  for (const auto& options : windows) {
    if (stopped_) {
      break;
    }
    CreateConfiguredWindow(options, options.size)->LoadURL(options.url);
  }
}
void WindowServiceImpl::CancelStartupWindows() {
  startup_primary_id_.reset();
  startup_windows_.clear();
}

WindowServiceImpl::WindowServiceImpl(content::BrowserContext& context)
    : context_(context) {}
WindowServiceImpl::~WindowServiceImpl() {
  Shutdown();
}
void WindowServiceImpl::Shutdown() {
  if (stopped_) {
    return;
  }
  stopped_ = true;
  startup_windows_observer_.reset();
  CancelStartupWindows();
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
  return IsAvailable(window) ? window : nullptr;
}

bool WindowServiceImpl::IsAvailable(Window* window) const {
  return !stopped_ && window && !closing_.contains(window) &&
         std::ranges::any_of(windows_, [window](const auto& owned) {
           return owned.get() == window;
         });
}
void WindowServiceImpl::CloseById(int64_t id) {
  Close(FindById(id));
}
Window* WindowServiceImpl::Create(const gfx::Size& size,
                                  bool decorations,
                                  const Constraints& constraints) {
  return stopped_ ? nullptr
                  : Window::Create(&*context_, GURL(), size, decorations,
                                   &constraints);
}
std::optional<Info> WindowServiceImpl::GetInfo(Window* window) {
  if (!IsAvailable(window)) {
    return std::nullopt;
  }
  const auto id = IdFor(window);
  const auto entry = std::ranges::find(entries_, id, &Entry::id);
  return Info{id, entry->label,
              window->web_contents()->GetLastCommittedURL().spec(),
              GetState(window)};
}
std::vector<Info> WindowServiceImpl::List() {
  std::vector<Info> result;
  for (const auto& window : windows_) {
    if (auto info = GetInfo(window.get())) {
      result.push_back(std::move(*info));
    }
  }
  return result;
}
bool WindowServiceImpl::Focus(Window* window) {
  return IsAvailable(window) && ::tuffite::window::Focus(window);
}
bool WindowServiceImpl::Minimize(Window* window) {
  return IsAvailable(window) && ::tuffite::window::Minimize(window);
}
bool WindowServiceImpl::Maximize(Window* window) {
  return IsAvailable(window) && ::tuffite::window::Maximize(window);
}
bool WindowServiceImpl::Restore(Window* window) {
  return IsAvailable(window) && ::tuffite::window::Restore(window);
}
bool WindowServiceImpl::SetFullscreen(Window* window, bool fullscreen) {
  return IsAvailable(window) &&
         ::tuffite::window::SetFullscreen(window, fullscreen);
}
bool WindowServiceImpl::SetTitle(Window* window, std::u16string_view title) {
  return IsAvailable(window) && ::tuffite::window::SetTitle(window, title);
}
bool WindowServiceImpl::SetBounds(Window* window, const gfx::Rect& bounds) {
  return IsAvailable(window) && window->constraints().Contains(bounds) &&
         ::tuffite::window::SetBounds(window, bounds);
}
bool WindowServiceImpl::Close(Window* window) {
  if (!IsAvailable(window)) {
    return false;
  }
  window->Close();
  return true;
}
}  // namespace tuffite::window
