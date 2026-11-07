// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/shortcut/shortcut_service_impl.h"

#include <algorithm>
#include <set>
#include <utility>
#include <vector>

#include "base/functional/bind.h"
#include "tuffite/framework/app/services/window/window.h"
#include "tuffite/framework/app/services/window/window_service.h"
#include "ui/views/focus/focus_manager.h"
#include "ui/views/widget/widget.h"
namespace tuffite {
ShortcutServiceImpl::ShortcutServiceImpl(window::WindowService& windows)
    : windows_(windows) {}
ShortcutServiceImpl::~ShortcutServiceImpl() {
  Shutdown();
}
void ShortcutServiceImpl::Shutdown() {
  if (stopped_) {
    return;
  }
  stopped_ = true;
  weak_factory_.InvalidateWeakPtrs();
  if (global_listener_) {
    global_listener_->UnregisterAccelerators(this);
  }
  shortcuts_.clear();
  RebuildShortcuts();
}
void ShortcutServiceImpl::WindowRegistered() {
  if (!stopped_) {
    RebuildShortcuts();
  }
}
void ShortcutServiceImpl::WindowClosed(Window* window, int64_t id) {
  window->SetShortcutHandler({});
  // Remove before invoking retirement callbacks, which may cause reentrant
  // work.
  std::vector<std::pair<std::string, ShortcutCallback>> retired;
  for (const auto& [key, shortcut] : shortcuts_) {
    if (shortcut.scope == id) {
      retired.emplace_back(key, shortcut.callback);
    }
  }
  std::erase_if(shortcuts_,
                [id](const auto& pair) { return pair.second.scope == id; });
  for (const auto& [key, callback] : retired) {
    callback.Run(key, -1);
  }
}
bool ShortcutServiceImpl::RegisterShortcut(std::string id,
                                           const ui::Accelerator& accelerator,
                                           int64_t scope,
                                           ShortcutCallback callback) {
  if (stopped_ || shortcuts_.size() >= 256 || shortcuts_.contains(id) ||
      scope < -1 || (scope > 0 && !windows_->FindById(scope))) {
    return false;
  }
  for (const auto& [key, shortcut] : shortcuts_) {
    if ((shortcut.scope == scope || shortcut.scope == -1 || scope == -1) &&
        shortcut.accelerator == accelerator) {
      return false;
    }
  }
  if (scope == -1) {
    auto* listener = ui::GlobalAcceleratorListener::GetInstance();
    if (!listener || listener->IsRegistrationHandledExternally() ||
        listener->IsShortcutHandlingSuspended() ||
        !listener->RegisterAccelerator(accelerator, this)) {
      return false;
    }
    global_listener_ = listener;
  }
  shortcuts_.emplace(std::move(id),
                     Shortcut{accelerator, scope, std::move(callback)});
  RebuildShortcuts();
  return true;
}
bool ShortcutServiceImpl::IsShortcutRegistered(const std::string& id) const {
  return !stopped_ && shortcuts_.contains(id);
}
void ShortcutServiceImpl::UnregisterShortcuts(int64_t scope) {
  std::vector<std::string> ids;
  for (const auto& [id, shortcut] : shortcuts_) {
    if (shortcut.scope == scope) {
      ids.push_back(id);
    }
  }
  for (const auto& id : ids) {
    UnregisterShortcut(id);
  }
}
bool ShortcutServiceImpl::UnregisterShortcut(const std::string& id) {
  auto found = shortcuts_.find(id);
  if (found == shortcuts_.end()) {
    return false;
  }
  if (found->second.scope == -1) {
    if (auto* listener = global_listener_.get()) {
      listener->UnregisterAccelerator(found->second.accelerator, this);
    }
  }
  auto callback = found->second.callback;
  shortcuts_.erase(found);
  RebuildShortcuts();
  callback.Run(id, -1);
  return true;
}
void ShortcutServiceImpl::RebuildShortcuts() {
  for (const auto& info : windows_->List()) {
    auto* window = windows_->FindById(info.id);
    if (!window) {
      continue;
    }
    window->SetShortcutHandler(
        stopped_ ? base::RepeatingCallback<bool(const ui::Accelerator&)>()
                 : base::BindRepeating(
                       [](base::WeakPtr<ShortcutServiceImpl> service,
                          Window* window, const ui::Accelerator& accelerator) {
                         return service &&
                                service->DispatchShortcut(window, accelerator);
                       },
                       weak_factory_.GetWeakPtr(), window));
    if (!window->widget()) {
      continue;
    }
    auto* manager = window->widget()->GetFocusManager();
    manager->UnregisterAccelerators(window);
    std::set<ui::Accelerator> accelerators;
    for (const auto& [id, shortcut] : shortcuts_) {
      if (shortcut.scope == 0 || shortcut.scope == info.id) {
        accelerators.insert(shortcut.accelerator);
      }
    }
    for (const auto& accelerator : accelerators) {
      manager->RegisterAccelerator(
          accelerator, ui::AcceleratorManager::kHighPriority, window);
    }
  }
}
bool ShortcutServiceImpl::DispatchShortcut(Window* window,
                                           const ui::Accelerator& accelerator) {
  const auto info = windows_->GetInfo(window);
  if (!info || stopped_) {
    return false;
  }
  for (const auto scope : {info->id, int64_t{0}}) {
    for (const auto& [id, shortcut] : shortcuts_) {
      if (shortcut.scope == scope && shortcut.accelerator == accelerator) {
        auto callback = shortcut.callback;
        callback.Run(id, info->id);
        return true;
      }
    }
  }
  return false;
}
void ShortcutServiceImpl::OnKeyPressed(const ui::Accelerator& accelerator) {
  if (stopped_) {
    return;
  }
  for (const auto& [id, shortcut] : shortcuts_) {
    if (shortcut.scope == -1 && shortcut.accelerator == accelerator) {
      auto callback = shortcut.callback;
      callback.Run(id, 0);
      return;
    }
  }
}
void ShortcutServiceImpl::ExecuteCommand(const std::string&,
                                         const std::string&) {}

}  // namespace tuffite
