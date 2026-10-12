// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/browser_main_parts.h"

#include <optional>
#include <string>
#include <utility>
#include <vector>

#include "base/command_line.h"
#include "base/files/file_path.h"
#include "base/files/file_util.h"
#include "base/memory/raw_ptr.h"
#include "base/run_loop.h"
#include "base/strings/utf_string_conversions.h"
#include "tuffite/buildflags.h"
#include "tuffite/framework/app/abi_host_bridge.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"
#include "tuffite/framework/browser/services/devtools/devtools_service_factory.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service_factory.h"
#include "tuffite/framework/browser/services/window/platform.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "tuffite/framework/browser/services/window/window_service.h"
#include "tuffite/framework/browser/services/window/window_service_factory.h"
#include "tuffite/framework/browser/test_support/browser_test_observers.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/common/app_manifest.h"
#include "tuffite/framework/common/constants.h"
#if BUILDFLAG(IS_MAC)
#include "content/public/browser/context_factory.h"
#endif
#include "content/public/browser/navigation_entry.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_observer.h"
#include "ui/display/screen.h"
#include "ui/views/buildflags.h"
#include "ui/views/layout/layout_provider.h"
#include "ui/views/views_delegate.h"
#include "ui/views/widget/widget.h"
#if BUILDFLAG(ENABLE_DESKTOP_AURA)
#include "ui/views/widget/desktop_aura/desktop_native_widget_aura.h"
#include "ui/views/widget/native_widget_aura.h"
#endif
#if !BUILDFLAG(IS_MAC)
#include "ui/views/widget/desktop_aura/desktop_screen.h"
#include "ui/wm/core/wm_state.h"
#endif
#include "url/gurl.h"

namespace tuffite {
namespace {

class TuffiteViewsDelegate final : public views::ViewsDelegate {
 public:
#if BUILDFLAG(IS_MAC)
  ui::ContextFactory* GetContextFactory() override {
    return content::GetContextFactory();
  }
#endif

  void OnBeforeWidgetInit(
      views::Widget::InitParams* params,
      views::internal::NativeWidgetDelegate* delegate) override {
#if BUILDFLAG(ENABLE_DESKTOP_AURA)
    if (params->native_widget) {
      return;
    }
    if (params->parent &&
        params->type != views::Widget::InitParams::TYPE_MENU &&
        params->type != views::Widget::InitParams::TYPE_TOOLTIP) {
      params->native_widget = new views::NativeWidgetAura(delegate);
    } else {
      params->native_widget = new views::DesktopNativeWidgetAura(delegate);
    }
#endif
  }
};

}  // namespace

BrowserMainParts::BrowserMainParts(AbiHostBridge& bridge,
                                   ApplicationContext& application)
    : bridge_(bridge), application_(application) {}
BrowserMainParts::~BrowserMainParts() = default;

void BrowserMainParts::ToolkitInitialized() {
  if (!views::ViewsDelegate::GetInstance()) {
    views_delegate_ = std::make_unique<TuffiteViewsDelegate>();
  }
  if (!views::LayoutProvider::Get()) {
    // Views menus initialize typography through the process-wide layout
    // provider. Content frameworks do not get Chrome's provider implicitly, so
    // install the default provider before an Explorer can open a context menu.
    layout_provider_ = std::make_unique<views::LayoutProvider>();
  }
#if BUILDFLAG(IS_MAC)
  screen_ = std::make_unique<display::ScopedNativeScreen>();
#else
  wm_state_ = std::make_unique<wm::WMState>();
  if (!display::Screen::HasScreen()) {
    screen_ = views::CreateDesktopScreen();
  }
#endif
}

int BrowserMainParts::PreMainMessageLoopRun() {
  const AppManifest* manifest = &application_->manifest();
  CHECK(manifest);
  base::FilePath profile =
      base::CommandLine::ForCurrentProcess()->GetSwitchValuePath(
          "user-data-dir");
  if (profile.empty()) {
    const auto data = manifest->ResolveBaseDirectory("data");
    CHECK(data);
    profile = data->AppendASCII("profile");
  }
  CHECK(base::CreateDirectory(profile));
  browser_context_ =
      std::make_unique<BrowserContext>(std::move(profile), *application_);
  DevToolsServiceFactory::GetForBrowserContext(browser_context_.get())
      ->StartIfRequested(browser_context_.get());
  InitializeMessageLoopContext();
  return 0;
}

void BrowserMainParts::WillRunMainMessageLoop(
    std::unique_ptr<base::RunLoop>& run_loop) {
  Window::SetQuitClosure(run_loop->QuitClosure());
}

void BrowserMainParts::PostMainMessageLoopRun() {
  window_surface_test_observer_.reset();
  smoke_test_observer_.reset();
  if (auto* devtools = DevToolsServiceFactory::GetForBrowserContext(
          browser_context_.get(), false)) {
    devtools->Shutdown();
  }
  bridge_->BeginShutdown();
  application_->GetService<UtilityService>()->Shutdown();
  Window::SetQuitClosure({});
  Window::Shutdown();
  browser_context_.reset();
  screen_.reset();
#if !BUILDFLAG(IS_MAC)
  wm_state_.reset();
#endif
  layout_provider_.reset();
  views_delegate_.reset();
}

void BrowserMainParts::InitializeMessageLoopContext() {
  bridge_->BindBrowserTaskRunner();
  const AppManifest* manifest = &application_->manifest();
  CHECK(manifest);
  const base::CommandLine* command_line =
      base::CommandLine::ForCurrentProcess();
  const AppWindow& primary = manifest->windows().front();
  const gfx::Size primary_size = command_line->HasSwitch(kSmokeTestSwitch)
                                     ? gfx::Size(1100, 1000)
                                     : gfx::Size(primary.width, primary.height);
  std::vector<window::WindowOptions> startup_windows;
  startup_windows.reserve(manifest->windows().size());
  for (const auto& configured : manifest->windows()) {
    startup_windows.push_back({
        .label = configured.label,
        .url = configured.url,
        .title = base::UTF8ToUTF16(configured.title),
        .size = gfx::Size(configured.width, configured.height),
        .position =
            configured.x && configured.y
                ? std::make_optional(gfx::Point(*configured.x, *configured.y))
                : std::nullopt,
        .focused = configured.focused,
        .decorations = configured.decorations,
        .constraints = {.minimum_size = gfx::Size(configured.min_width,
                                                  configured.min_height),
                        .maximum_size = gfx::Size(configured.max_width,
                                                  configured.max_height)},
        .role = WebContentsRole::kApp,
    });
  }
  Window* primary_window =
      window::WindowServiceFactory::GetForBrowserContext(browser_context_.get())
          ->CreateStartupWindows(std::move(startup_windows), primary_size);

  if (command_line->HasSwitch(kSmokeTestSwitch)) {
    base::FilePath marker = command_line->GetSwitchValuePath(kSmokeTestSwitch);
    if (!marker.empty()) {
      smoke_test_observer_ =
          CreateSmokeTestObserver(primary_window->web_contents(), marker);
    }
  }
  if (command_line->HasSwitch(kWindowSurfaceTestSwitch)) {
    base::FilePath marker =
        command_line->GetSwitchValuePath(kWindowSurfaceTestSwitch);
    if (!marker.empty()) {
      window_surface_test_observer_ = CreateWindowSurfaceTestObserver(
          primary_window->web_contents(), primary_window, std::move(marker));
    }
  }
}

}  // namespace tuffite
