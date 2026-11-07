// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_SHORTCUT_SHORTCUT_SERVICE_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_SHORTCUT_SHORTCUT_SERVICE_H_
#include <cstdint>
#include <string>

#include "base/functional/callback.h"
#include "components/keyed_service/core/keyed_service.h"
#include "ui/base/accelerators/accelerator.h"
namespace tuffite {
class Window;
class ShortcutService : public KeyedService {
 public:
  using ShortcutCallback = base::RepeatingCallback<void(std::string, int64_t)>;
  // scope: positive WindowId, 0 application, -1 system.
  virtual bool RegisterShortcut(std::string,
                                const ui::Accelerator&,
                                int64_t scope,
                                ShortcutCallback) = 0;
  virtual bool IsShortcutRegistered(const std::string&) const = 0;
  virtual void UnregisterShortcuts(int64_t scope) = 0;
  virtual bool UnregisterShortcut(const std::string&) = 0;
  virtual bool DispatchShortcut(Window*, const ui::Accelerator&) = 0;

  virtual void WindowRegistered() = 0;
  virtual void WindowClosed(Window*, int64_t id) = 0;
};
}  // namespace tuffite
#endif
