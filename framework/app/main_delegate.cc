// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/main_delegate.h"

#include <memory>
#include <vector>

#include "base/base_paths.h"
#include "base/check.h"
#include "base/command_line.h"
#include "base/environment.h"
#include "base/files/file_path.h"
#include "base/files/file_util.h"
#include "base/logging.h"
#include "base/logging/logging_settings.h"
#include "base/path_service.h"
#include "base/process/process.h"
#include "base/process/process_handle.h"
#include "base/strings/string_number_conversions.h"
#include "components/network_session_configurator/common/network_switches.h"
#include "content/public/app/content_main.h"
#include "content/public/common/content_switches.h"
#include "content/public/common/result_codes.h"
#include "crypto/secure_util.h"
#include "crypto/sha2.h"
#include "tuffite/buildflags.h"
#include "tuffite/framework/app/abi_host_bridge.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/content_client.h"
#include "tuffite/framework/app/services/application_dependency_manager.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/manifest/manifest_service_factory.h"
#include "tuffite/framework/browser/content_browser_client.h"
#include "tuffite/framework/common/switches.h"
#include "tuffite/framework/renderer/content_renderer_client.h"
#include "tuffite/framework/utility/content_utility_client.h"
#include "ui/base/resource/resource_bundle.h"
#include "ui/base/ui_base_paths.h"

#if BUILDFLAG(IS_MAC)
#include "base/apple/bundle_locations.h"
#include "base/apple/foundation_util.h"
#endif

#if BUILDFLAG(IS_WIN)
#include <windows.h>

#include "base/win/dark_mode_support.h"
#include "base/win/scoped_handle.h"
#include "base/win/win_util.h"
#include "sandbox/win/src/sandbox_types.h"
#endif

namespace tuffite {
namespace {

constexpr char kDevTokenEnvironment[] = "TUFFITE_DEV_TOKEN";

#if BUILDFLAG(TUFFITE_DEV_MODE)
#if BUILDFLAG(IS_WIN)
bool IsTuffiteCliParent() {
  const base::ProcessId parent_id =
      base::GetParentProcessId(base::GetCurrentProcessHandle());
  if (parent_id == base::kNullProcessId) {
    return false;
  }
  base::Process parent = base::Process::OpenWithAccess(
      parent_id, PROCESS_QUERY_LIMITED_INFORMATION);
  if (!parent.IsValid()) {
    return false;
  }
  std::wstring executable(32768, L'\0');
  DWORD size = static_cast<DWORD>(executable.size());
  if (!::QueryFullProcessImageNameW(parent.Handle(), 0, executable.data(),
                                    &size)) {
    return false;
  }
  executable.resize(size);
  return base::FilePath::CompareEqualIgnoreCase(
      base::FilePath(executable).BaseName().value(),
      FILE_PATH_LITERAL("tuff.exe"));
}
#else
bool IsTuffiteCliParent() {
  const base::ProcessId parent_id =
      base::GetParentProcessId(base::GetCurrentProcessHandle());
  if (parent_id == base::kNullProcessId) {
    return false;
  }
  return base::GetProcessExecutablePath(parent_id).BaseName() ==
         base::FilePath(FILE_PATH_LITERAL("tuff"));
}
#endif
#endif

// Tuffite owns application bootstrap failures. Content's public result-code
// enum is intentionally frozen and no longer exposes MISSING_DATA.
constexpr int kRuntimeBootstrapFailure = 64;

void InitLogging(const base::CommandLine& command_line) {
  base::FilePath log_path = command_line.GetSwitchValuePath(switches::kLogFile);
  const bool is_child_process = command_line.HasSwitch(switches::kProcessType);
#if BUILDFLAG(IS_WIN)
  base::win::ScopedHandle log_handle;
  if (command_line.GetSwitchValueASCII(switches::kEnableLogging) == "handle") {
    uint32_t handle_value = 0;
    if (base::StringToUint(
            command_line.GetSwitchValueNative(switches::kLogFile),
            &handle_value)) {
      HANDLE duplicate = nullptr;
      if (::DuplicateHandle(::GetCurrentProcess(),
                            base::win::Uint32ToHandle(handle_value),
                            ::GetCurrentProcess(), &duplicate, 0, FALSE,
                            DUPLICATE_SAME_ACCESS)) {
        log_handle.Set(duplicate);
      }
    }
  }
#endif
#if BUILDFLAG(IS_MAC)
  // Sandboxed child processes cannot open the package-local log file. Their
  // stderr is inherited by the browser process launcher and remains available
  // without granting an additional Seatbelt file extension.
  if (is_child_process) {
    log_path.clear();
  }
#endif
  if (log_path.empty()) {
#if BUILDFLAG(IS_MAC)
    if (!is_child_process) {
#endif
      base::FilePath executable_directory;
#if BUILDFLAG(IS_MAC)
      if (base::apple::MainBundlePath().MatchesExtension(
              FILE_PATH_LITERAL(".app"))) {
        base::FilePath cache;
        if (base::PathService::Get(base::DIR_CACHE, &cache)) {
          cache = cache.AppendASCII("tuffite").Append(
              base::apple::MainBundlePath().BaseName().RemoveExtension());
          if (base::CreateDirectory(cache)) {
            log_path = cache.AppendASCII("tuffite.log");
          }
        }
      } else
#endif
          if (base::PathService::Get(base::DIR_EXE, &executable_directory)) {
        log_path = executable_directory.AppendASCII("tuffite.log");
      }
#if BUILDFLAG(IS_MAC)
    }
#endif
  }

  logging::LoggingSettings settings;
#if BUILDFLAG(IS_WIN)
  if (log_handle.is_valid()) {
    settings.logging_dest = logging::LOG_TO_ALL;
    settings.log_file = log_handle.release();
  } else
#endif
      if (!log_path.empty()) {
    settings.logging_dest = logging::LOG_TO_ALL;
    settings.log_file_path = log_path.value();
  } else {
    settings.logging_dest = logging::LOG_TO_STDERR;
  }
  settings.delete_old = is_child_process ? logging::APPEND_TO_OLD_LOG_FILE
                                         : logging::DELETE_OLD_LOG_FILE;
  CHECK(logging::InitLogging(settings));
  logging::SetLogItems(/*enable_process_id=*/true,
                       /*enable_thread_id=*/true,
                       /*enable_timestamp=*/true,
                       /*enable_tickcount=*/false);
}

}  // namespace

MainDelegate::MainDelegate(std::unique_ptr<AbiHostBridge> bridge)
    : bridge_(std::move(bridge)) {
  CHECK(bridge_);
}
MainDelegate::~MainDelegate() {
  Shutdown();
}
void MainDelegate::ProcessExiting(const std::string&) {
  Shutdown();
}
bool MainDelegate::StartUp(std::unique_ptr<ManifestService> manifest) {
  CHECK(!stopped_);
  CHECK(!context_);
  context_ =
      std::make_unique<ApplicationContext>(std::move(manifest), *bridge_);
  ApplicationDependencyManager::Get()->CreateServices(context_.get());
  bridge_->BindApplicationContext(context_.get());
  if (browser_client_) {
    browser_client_->BindApplicationServices(context_.get());
  }
  return bridge_->Start(
      context_->GetService<ManifestService>()->SerializeRuntimeConfiguration());
}
void MainDelegate::Shutdown() {
  if (stopped_) {
    return;
  }
  stopped_ = true;
  bridge_->BeginShutdown();
  bridge_->Stop();
  if (browser_client_) {
    browser_client_->BindApplicationServices(nullptr);
  }
  bridge_->BindApplicationContext(nullptr);
  if (context_) {
    ApplicationDependencyManager::Get()->DestroyServices(context_.get());
    context_.reset();
  }
}
int MainDelegate::Run(int argc, const char* const* argv, void* sandbox_info) {
  CHECK(!run_called_);
  CHECK(!stopped_);
  run_called_ = true;
  content::ContentMainParams params(this);
#if BUILDFLAG(IS_WIN)
  base::win::PinUser32();
  base::win::AllowDarkModeForApp(true);
  params.instance = ::GetModuleHandle(nullptr);
  params.sandbox_info =
      static_cast<sandbox::SandboxInterfaceInfo*>(sandbox_info);
#else
  params.argc = argc;
  params.argv = const_cast<const char**>(argv);
#endif
  const int result = content::ContentMain(std::move(params));
  const bool initialization_failed = bridge_->initialization_failed();
  Shutdown();
  return initialization_failed ? kRuntimeBootstrapFailure : result;
}

std::optional<int> MainDelegate::BasicStartupComplete() {
  base::CommandLine* command_line = base::CommandLine::ForCurrentProcess();
  InitLogging(*command_line);
  // Packaged Chromium data lives beside the executable or in bundle Resources.
  // Set this for every process before ICU and
  // V8 initialize their startup data.
  base::FilePath asset_directory;
  if (!base::PathService::Get(base::DIR_EXE, &asset_directory)) {
    return kRuntimeBootstrapFailure;
  }
#if BUILDFLAG(IS_MAC)
  if (base::apple::MainBundlePath().MatchesExtension(
          FILE_PATH_LITERAL(".app"))) {
    asset_directory = base::apple::MainBundlePath()
                          .AppendASCII("Contents")
                          .AppendASCII("Resources");
  }
#endif
  if (!base::PathService::Override(base::DIR_ASSETS, asset_directory)) {
    return kRuntimeBootstrapFailure;
  }
  if (command_line->HasSwitch(switches::kProcessType)) {
    return std::nullopt;
  }
  base::FilePath executable_directory;
  if (!base::PathService::Get(base::DIR_ASSETS, &executable_directory)) {
    return kRuntimeBootstrapFailure;
  }
  base::FilePath executable_path;
  if (!base::PathService::Get(base::FILE_EXE, &executable_path) ||
      executable_path.empty()) {
    return kRuntimeBootstrapFailure;
  }
  // Tuffite uses the business-owned executable for Chromium child processes.
  // Set this explicitly instead of relying on framework-specific PathService
  // registration so sandbox setup always receives a concrete executable path.
  command_line->AppendSwitchPath(switches::kBrowserSubprocessPath,
                                 executable_path);
  std::string manifest_error;
  std::optional<std::string> app_scheme =
      app::ManifestLoader::ReadApplicationScheme(executable_directory,
                                                 &manifest_error);
  if (!app_scheme) {
    LOG(ERROR) << "Unable to load Tuffite application manifest: "
               << manifest_error;
    return kRuntimeBootstrapFailure;
  }
  command_line->AppendSwitchASCII(kAppSchemeSwitch, *app_scheme);
  if (app::ManifestLoader::ReadNetworkDisableHttp2(executable_directory)) {
    command_line->AppendSwitch(switches::kDisableHttp2);
  }
  // Blink reads runtime feature switches before PreBrowserMain(). There is no
  // renderer child command line in single-process mode, so opt the current
  // process into the Tuffite element here. HTMLExplorerElement::IsEnabled still
  // enforces the manifest-derived origin allowlist installed below.
  if (command_line->HasSwitch(switches::kSingleProcess)) {
    std::string features =
        command_line->GetSwitchValueASCII(switches::kEnableBlinkFeatures);
    if (!features.empty()) {
      features.push_back(',');
    }
    features.append("TuffiteExplorer");
    command_line->AppendSwitchASCII(switches::kEnableBlinkFeatures, features);
  }
  return std::nullopt;
}

void MainDelegate::PreSandboxStartup() {
  base::FilePath pak_file;
  CHECK(base::PathService::Get(base::DIR_ASSETS, &pak_file));
  ui::ResourceBundle::InitSharedInstanceWithPakPath(
      pak_file.Append(FILE_PATH_LITERAL("tuffite.pak")));
}

std::optional<int> MainDelegate::PreBrowserMain() {
  if (std::optional<int> result =
          content::ContentMainDelegate::PreBrowserMain()) {
    return result;
  }
  if (base::CommandLine::ForCurrentProcess()->HasSwitch(
          switches::kProcessType)) {
    return std::nullopt;
  }
#if BUILDFLAG(IS_MAC)
  RegisterNSApplication();
#endif
  base::FilePath executable_directory;
  if (!base::PathService::Get(base::DIR_ASSETS, &executable_directory) ||
      !base::PathService::Override(
          ui::DIR_LOCALES,
          executable_directory.AppendASCII("tuffite_locales"))) {
    return kRuntimeBootstrapFailure;
  }
  std::string manifest_error;
  const std::string development_url =
      base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
          kDevUrlSwitch);
  app::StartupEnvironment startup_environment;
  if (!development_url.empty()) {
#if !BUILDFLAG(TUFFITE_DEV_MODE)
    LOG(ERROR) << "Tuffite development URLs are disabled in this build";
    return kRuntimeBootstrapFailure;
#else
    const std::string development_token =
        base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
            kDevTokenSwitch);
    std::unique_ptr<base::Environment> environment =
        base::Environment::Create();
    const std::optional<std::string> inherited_token =
        environment->GetVar(kDevTokenEnvironment);
    environment->UnSetVar(kDevTokenEnvironment);
    std::vector<uint8_t> decoded_token;
    const bool decoded =
        base::HexStringToBytes(development_token, &decoded_token);
    const std::string inherited_digest =
        inherited_token.has_value() ? crypto::SHA256HashString(*inherited_token)
                                    : std::string();
    const bool valid_token =
        decoded && decoded_token.size() == 32 && inherited_token.has_value() &&
        inherited_token->size() == 64 && IsTuffiteCliParent() &&
        crypto::SecureMemEqual(decoded_token,
                               base::as_byte_span(inherited_digest));
    if (!valid_token) {
      LOG(ERROR) << "Tuffite development URL requires a CLI-issued token";
      return kRuntimeBootstrapFailure;
    }
    startup_environment.development_url = GURL(development_url);
#endif
  }
  auto definition =
      app::ManifestLoader::Load(executable_directory, &manifest_error);
  if (!definition) {
    LOG(ERROR) << "Unable to load Tuffite application manifest: "
               << manifest_error;
    return kRuntimeBootstrapFailure;
  }
  app::BrowserOptions browser_options;
  if (const auto setup = bridge().application().setup_browser) {
    const auto status =
        setup(&browser_options.security.devtools.denied_origins,
              [](void* data, tuffite_byte_slice_t origin) -> tuffite_status_t {
                auto& origins = *static_cast<std::vector<std::string>*>(data);
                if (!origin.data || !origin.len || origin.len > 4096 ||
                    origins.size() >= 256) {
                  return TUFFITE_STATUS_INVALID_ARGUMENT;
                }
                origins.emplace_back(reinterpret_cast<const char*>(origin.data),
                                     origin.len);
                return TUFFITE_STATUS_OK;
              });
    if (status != TUFFITE_STATUS_OK) {
      LOG(ERROR) << "Tuffite Browser::setup failed";
      return kRuntimeBootstrapFailure;
    }
  }
  auto configuration = app::ConfigurationAssembler::Build(
      std::move(*definition), startup_environment, browser_options,
      &manifest_error);
  if (!configuration) {
    LOG(ERROR) << "Unable to assemble Tuffite application configuration: "
               << manifest_error;
    return kRuntimeBootstrapFailure;
  }
  auto app_manifest = ManifestServiceFactory::Create(std::move(*configuration));
  // Normal renderer children receive these immutable policy switches from
  // ContentBrowserClient::AppendExtraCommandLineSwitches. Mirror that policy
  // in the shared command line used by a single-process smoke/debug run.
  if (base::CommandLine::ForCurrentProcess()->HasSwitch(
          switches::kSingleProcess)) {
    base::CommandLine* command_line = base::CommandLine::ForCurrentProcess();
    command_line->AppendSwitchASCII(kShellApiOriginsSwitch,
                                    app_manifest->SerializeShellApiOrigins());
    command_line->AppendSwitchASCII(kExplorerOriginsSwitch,
                                    app_manifest->SerializeExplorerOrigins());
  }
  if (!StartUp(std::move(app_manifest))) {
    return kRuntimeBootstrapFailure;
  }
  return std::nullopt;
}

content::ContentClient* MainDelegate::CreateContentClient() {
  content_client_ = std::make_unique<ContentClient>();
  return content_client_.get();
}

content::ContentBrowserClient* MainDelegate::CreateContentBrowserClient() {
  browser_client_ = std::make_unique<ContentBrowserClient>(bridge());
  browser_client_->BindApplicationServices(context());
  return browser_client_.get();
}

content::ContentRendererClient* MainDelegate::CreateContentRendererClient() {
  renderer_client_ = std::make_unique<ContentRendererClient>();
  return renderer_client_.get();
}

content::ContentUtilityClient* MainDelegate::CreateContentUtilityClient() {
  utility_client_ =
      std::make_unique<ContentUtilityClient>(bridge().application());
  return utility_client_.get();
}

}  // namespace tuffite
