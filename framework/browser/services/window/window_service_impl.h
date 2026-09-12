// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_SERVICE_IMPL_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_SERVICE_IMPL_H_

#include <cstdint>
#include <optional>
#include <set>
#include <string_view>
#include <vector>

#include "base/containers/span.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/raw_ref.h"
#include "tuffite/framework/browser/services/window/window_service.h"

namespace content {
class WebContents;
class BrowserContext;
}  // namespace content
namespace tuffite {
class AppManifest;
struct AppWindow;
class Window;
}  // namespace tuffite

namespace tuffite::window {

// Browser-process owner for opaque WindowIds and the builtin window routes.
// The registry intentionally stores Shell identity only; native handles never
// cross the ShellAPI boundary.
class WindowBackend;
class StartupWindowsObserver;
class WindowServiceImpl final : public WindowService {
 public:
  WindowServiceImpl(content::BrowserContext&,
                    const AppManifest&,
                    std::unique_ptr<WindowBackend>);
  ~WindowServiceImpl() override;
  void Shutdown() override;
  Window* CreateStartupWindows(const gfx::Size& primary_size) override;
  void Adopt(std::unique_ptr<Window>) override;
  void DestroyWhenIdle(Window*) override;
  void Unregister(Window*) override;
  base::WeakPtr<WindowService> GetWeakPtr() override;

  Result Invoke(const Command&, content::WebContents*) override;
  void Register(Window* window, std::string label) override;
  void CloseById(int64_t id) override;

 private:
  struct Entry {
    int64_t id;
    raw_ptr<Window> window;
    std::string label;
  };

  Window* CreateConfiguredWindow(const AppWindow&, const gfx::Size& size);
  void CreateSecondaryWindows();
  void DestroyWindow(Window*);
  void Reconcile();
  int64_t IdFor(Window* window, std::string label = {});
  Window* FindById(int64_t id);

  const raw_ref<content::BrowserContext> context_;
  const raw_ref<const AppManifest> manifest_;
  std::unique_ptr<WindowBackend> backend_;
  bool stopped_ = false;
  bool startup_created_ = false;
  std::unique_ptr<StartupWindowsObserver> startup_windows_observer_;
  std::vector<std::unique_ptr<Window>> windows_;
  std::set<Window*> closing_;
  std::vector<Entry> entries_;
  int64_t next_id_ = 1;
  base::WeakPtrFactory<WindowServiceImpl> weak_factory_{this};
};

}  // namespace tuffite::window

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_SERVICE_IMPL_H_
