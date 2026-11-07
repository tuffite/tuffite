// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_SHORTCUT_SHORTCUT_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_SHORTCUT_SHORTCUT_SERVICE_IMPL_H_
#include <map>

#include "base/memory/raw_ptr.h"
#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "tuffite/framework/app/services/shortcut/shortcut_service.h"
#include "ui/base/accelerators/global_accelerator_listener/global_accelerator_listener.h"
namespace tuffite {
namespace window {
class WindowService;
}
class ShortcutServiceImpl final
    : public ShortcutService,
      public ui::GlobalAcceleratorListener::Observer {
 public:
  explicit ShortcutServiceImpl(window::WindowService&);
  ~ShortcutServiceImpl() override;
  bool RegisterShortcut(std::string,
                        const ui::Accelerator&,
                        int64_t,
                        ShortcutCallback) override;
  bool IsShortcutRegistered(const std::string&) const override;
  void UnregisterShortcuts(int64_t scope) override;
  bool UnregisterShortcut(const std::string&) override;
  bool DispatchShortcut(Window*, const ui::Accelerator&) override;
  void OnKeyPressed(const ui::Accelerator&) override;
  void ExecuteCommand(const std::string&, const std::string&) override;

  void WindowRegistered() override;
  void WindowClosed(Window*, int64_t id) override;
  void Shutdown() override;

 private:
  struct Shortcut {
    ui::Accelerator accelerator;
    int64_t scope;
    ShortcutCallback callback;
  };
  raw_ptr<ui::GlobalAcceleratorListener> global_listener_ = nullptr;
  std::map<std::string, Shortcut> shortcuts_;
  void RebuildShortcuts();

  const raw_ref<window::WindowService> windows_;
  bool stopped_ = false;
  base::WeakPtrFactory<ShortcutServiceImpl> weak_factory_{this};
};
}  // namespace tuffite
#endif
