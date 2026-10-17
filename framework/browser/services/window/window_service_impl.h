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
class Window;
}  // namespace tuffite

namespace tuffite::window {

// Browser-process owner for opaque WindowIds and the builtin window routes.
// The registry intentionally stores Shell identity only; native handles never
// cross the ShellAPI boundary.
class StartupWindowsObserver;
class WindowServiceImpl final : public WindowService {
 public:
  explicit WindowServiceImpl(content::BrowserContext&);
  ~WindowServiceImpl() override;
  void Shutdown() override;
  Window* Create(const gfx::Size&,
                 bool decorations,
                 const Constraints&) override;
  Window* CreateStartupWindows(std::vector<WindowOptions>,
                               const gfx::Size&) override;
  std::optional<Info> GetInfo(Window*) override;
  std::vector<Info> List() override;
  bool Focus(Window*) override;
  bool Minimize(Window*) override;
  bool Maximize(Window*) override;
  bool Restore(Window*) override;
  bool SetFullscreen(Window*, bool) override;
  bool SetTitle(Window*, std::u16string_view) override;
  bool SetBounds(Window*, const gfx::Rect&) override;
  bool Close(Window*) override;
  Window* FindById(int64_t id) override;
  void Adopt(std::unique_ptr<Window>) override;
  void DestroyWhenIdle(Window*) override;
  void Unregister(Window*) override;
  base::WeakPtr<WindowService> GetWeakPtr() override;

  void Register(Window* window, std::string label) override;
  void CloseById(int64_t id) override;
  void ResolveMessageTarget(
      int64_t,
      content::RenderFrameHost&,
      base::OnceCallback<void(content::RenderFrameHost*)>) override;

 private:
  struct Entry {
    int64_t id;
    raw_ptr<Window> window;
    std::string label;
  };

  bool IsAvailable(Window*) const;
  Window* CreateConfiguredWindow(const WindowOptions&, const gfx::Size&);
  void CreateSecondaryWindows();
  void CancelStartupWindows();
  void DestroyWindow(Window*);
  void Reconcile();
  int64_t IdFor(Window* window, std::string label = {});

  const raw_ref<content::BrowserContext> context_;
  bool stopped_ = false;
  bool startup_created_ = false;
  std::optional<int64_t> startup_primary_id_;
  std::vector<WindowOptions> startup_windows_;
  std::unique_ptr<StartupWindowsObserver> startup_windows_observer_;
  std::vector<std::unique_ptr<Window>> windows_;
  std::set<Window*> closing_;
  std::vector<Entry> entries_;
  int64_t next_id_ = 1;
  base::WeakPtrFactory<WindowServiceImpl> weak_factory_{this};
};

}  // namespace tuffite::window

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WINDOW_WINDOW_SERVICE_IMPL_H_
