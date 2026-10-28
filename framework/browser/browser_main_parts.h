// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_BROWSER_MAIN_PARTS_H_
#define TUFFITE_FRAMEWORK_BROWSER_BROWSER_MAIN_PARTS_H_

#include <memory>

#include "base/functional/callback.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "build/build_config.h"
#include "content/public/browser/browser_main_parts.h"

namespace base {
class RunLoop;
}

namespace content {
class BrowserContext;
}

namespace display {
class Screen;
#if BUILDFLAG(IS_MAC)
class ScopedNativeScreen;
#endif
}  // namespace display
namespace views {
class LayoutProvider;
class ViewsDelegate;
}  // namespace views
#if !BUILDFLAG(IS_MAC)
namespace wm {
class WMState;
}  // namespace wm
#endif

namespace tuffite {
namespace abi {
class Application;
class Host;
}  // namespace abi
class ApplicationContext;
class BrowserTestObserver;
class BrowserContext;

class BrowserMainParts final : public content::BrowserMainParts {
 public:
  BrowserMainParts(abi::Application& application,
                   abi::Host& host,
                   base::RepeatingClosure prepare_browser,
                   base::RepeatingClosure begin_shutdown,
                   base::RepeatingClosure shutdown);
  ~BrowserMainParts() override;

 private:
  void ToolkitInitialized() override;
  int PreMainMessageLoopRun() override;
  void WillRunMainMessageLoop(
      std::unique_ptr<base::RunLoop>& run_loop) override;
  void PostMainMessageLoopRun() override;
  void InitializeMessageLoopContext();
  void OnBrowserInitialized(bool ok);
  base::WeakPtr<BrowserContext> startup_context_;

  const raw_ref<abi::Application> application_;
  const raw_ref<abi::Host> host_;
  const base::RepeatingClosure prepare_browser_;
  const base::RepeatingClosure begin_shutdown_;
  const base::RepeatingClosure shutdown_;
  std::unique_ptr<BrowserContext> browser_context_;
  std::unique_ptr<views::ViewsDelegate> views_delegate_;
  std::unique_ptr<views::LayoutProvider> layout_provider_;
#if BUILDFLAG(IS_MAC)
  std::unique_ptr<display::ScopedNativeScreen> screen_;
#else
  std::unique_ptr<display::Screen> screen_;
  std::unique_ptr<wm::WMState> wm_state_;
#endif
  std::unique_ptr<BrowserTestObserver> smoke_test_observer_;
  std::unique_ptr<BrowserTestObserver> window_surface_test_observer_;
  base::RepeatingClosure quit_;
  base::WeakPtrFactory<BrowserMainParts> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_BROWSER_MAIN_PARTS_H_
