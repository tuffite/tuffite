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
#include "base/functional/bind.h"
#include "base/logging.h"
#include "base/memory/raw_ptr.h"
#include "base/run_loop.h"
#include "base/strings/utf_string_conversions.h"
#include "tuffite/buildflags.h"
#include "tuffite/framework/app/abi/application.h"
#include "tuffite/framework/app/abi/host.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/profile/profile_manager.h"
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
#include "tuffite/framework/common/switches.h"
#if BUILDFLAG(IS_MAC)
#include "content/public/browser/context_factory.h"
#endif
#include "content/public/browser/child_process_security_policy.h"
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

BrowserMainParts::BrowserMainParts(abi::Application& application,
                                   abi::Host& host,
                                   base::RepeatingClosure prepare_browser,
                                   base::RepeatingClosure begin_shutdown,
                                   base::RepeatingClosure shutdown)
    : application_(application),
      host_(host),
      prepare_browser_(std::move(prepare_browser)),
      begin_shutdown_(std::move(begin_shutdown)),
      shutdown_(std::move(shutdown)) {}
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
  // Application resources use ordinary web origins. Allow requests across the
  // declared internal schemes while retaining Chromium's per-process commit
  // grants. Origin policy and resource loaders still authorize each operation.
  auto* security = content::ChildProcessSecurityPolicy::GetInstance();
  for (const auto& scheme : ApplicationSchemes()) {
    if (!security->IsWebSafeScheme(scheme)) {
      security->RegisterWebSafeIsolatedScheme(scheme);
    }
  }
  const ManifestService* manifest =
      host_->application_context()->GetService<ManifestService>();
  CHECK(manifest);
  base::FilePath profile =
      base::CommandLine::ForCurrentProcess()->GetSwitchValuePath(
          "user-data-dir");
  if (profile.empty()) {
    const auto data = manifest->ResolveBaseDirectory("data");
    CHECK(data);
    profile = data->AppendASCII("profile");
  }
  if (application_->table().start_profile) {
    auto* profiles = host_->application_context()->GetService<ProfileManager>();
    if (!profiles->Configure(*application_, profile)) {
      return 64;
    }
    prepare_browser_.Run();
    application_->InitializeBrowser(base::BindOnce(
        &BrowserMainParts::OnBrowserInitialized, weak_factory_.GetWeakPtr()));
    return 0;
  }
  CHECK(base::CreateDirectory(profile));
  browser_context_ = std::make_unique<BrowserContext>(
      std::move(profile), *host_->application_context(), *application_);
  startup_context_ = browser_context_->GetWeakPtr();
  DevToolsServiceFactory::GetForBrowserContext(startup_context_.get())
      ->StartIfRequested(startup_context_.get());
  host_->BindBrowserContext(browser_context_->GetWeakPtr());
  prepare_browser_.Run();
  application_->InitializeBrowser(base::BindOnce(
      &BrowserMainParts::OnBrowserInitialized, weak_factory_.GetWeakPtr()));
  return 0;
}

void BrowserMainParts::WillRunMainMessageLoop(
    std::unique_ptr<base::RunLoop>& run_loop) {
  quit_ = run_loop->QuitClosure();
  Window::SetQuitClosure(quit_);
  if (application_->initialization_failed()) {
    quit_.Run();
  }
}

void BrowserMainParts::PostMainMessageLoopRun() {
  weak_factory_.InvalidateWeakPtrs();
  window_surface_test_observer_.reset();
  smoke_test_observer_.reset();
  if (auto* devtools = startup_context_
                           ? DevToolsServiceFactory::GetForBrowserContext(
                                 startup_context_.get(), false)
                           : nullptr) {
    devtools->Shutdown();
  }
  begin_shutdown_.Run();
  Window::SetQuitClosure({});
  Window::Shutdown();
  if (application_->table().start_profile) {
    host_->application_context()->GetService<ProfileManager>()->Shutdown();
  }
  startup_context_ = nullptr;
  browser_context_.reset();
  shutdown_.Run();
  screen_.reset();
#if !BUILDFLAG(IS_MAC)
  wm_state_.reset();
#endif
  layout_provider_.reset();
  views_delegate_.reset();
}

void BrowserMainParts::OnBrowserInitialized(bool ok) {
  if (!ok) {
    LOG(ERROR) << "Tuffite Browser initialization failed";
    if (quit_) {
      quit_.Run();
    }
    return;
  }
  if (application_->table().start_profile) {
    host_->application_context()->GetService<ProfileManager>()->Open(
        "default",
        base::BindOnce(
            [](base::WeakPtr<BrowserMainParts> self, BrowserContext* context) {
              if (!self) {
                return;
              }
              if (!context) {
                LOG(ERROR) << "Unable to open startup profile";
                if (self->quit_) {
                  self->quit_.Run();
                }
                return;
              }
              self->startup_context_ = context->GetWeakPtr();
              DevToolsServiceFactory::GetForBrowserContext(context)
                  ->StartIfRequested(context);
              self->InitializeMessageLoopContext();
            },
            weak_factory_.GetWeakPtr()));
  } else {
    InitializeMessageLoopContext();
  }
}

void BrowserMainParts::InitializeMessageLoopContext() {
  const ManifestService* manifest =
      host_->application_context()->GetService<ManifestService>();
  CHECK(manifest);
  const base::CommandLine* command_line =
      base::CommandLine::ForCurrentProcess();
  const app::AppWindow& primary = manifest->startup_windows().front();
  const gfx::Size primary_size = command_line->HasSwitch(kSmokeTestSwitch)
                                     ? gfx::Size(1100, 1000)
                                     : gfx::Size(primary.width, primary.height);
  std::vector<window::WindowOptions> startup_windows;
  startup_windows.reserve(manifest->startup_windows().size());
  for (const auto& configured : manifest->startup_windows()) {
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
      window::WindowServiceFactory::GetForBrowserContext(startup_context_.get())
          ->CreateStartupWindows(*startup_context_, std::move(startup_windows),
                                 primary_size);

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
