// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_MAIN_DELEGATE_H_
#define TUFFITE_FRAMEWORK_APP_MAIN_DELEGATE_H_

#include <memory>
#include <optional>
#include <string>

#include "build/build_config.h"
#include "content/public/app/content_main_delegate.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"

namespace tuffite {
namespace abi {
class Application;
class Host;
}  // namespace abi
class ApplicationContext;
class ContentBrowserClient;
class ContentClient;
class ContentRendererClient;
class ContentUtilityClient;

class MainDelegate final : public content::ContentMainDelegate {
 public:
  explicit MainDelegate(std::unique_ptr<abi::Application> application);
  int Run(int argc, const char* const* argv, void* sandbox_info);
  bool StartUp(std::unique_ptr<ManifestService> manifest);
  void Shutdown();
  abi::Application& application() { return *application_; }
  abi::Host& host() { return *host_; }
  void BindBrowserTaskRunner();
  void BeginShutdown();
  ApplicationContext* context();
  ~MainDelegate() override;

  std::optional<int> BasicStartupComplete() override;
  void PreSandboxStartup() override;
  void ProcessExiting(const std::string& process_type) override;
  std::optional<int> PreBrowserMain() override;
  content::ContentClient* CreateContentClient() override;
  content::ContentBrowserClient* CreateContentBrowserClient() override;
  content::ContentRendererClient* CreateContentRendererClient() override;
  content::ContentUtilityClient* CreateContentUtilityClient() override;

 private:
  enum class ShutdownPhase {
    kReady,
    kQuiescing,
    kQuiesced,
    kStopping,
    kStopped,
  };

#if BUILDFLAG(IS_MAC)
  static void RegisterNSApplication();
#endif

  // Declaration order keeps Application alive while Host's event sink exists.
  std::unique_ptr<abi::Application> application_;
  std::unique_ptr<abi::Host> host_;
  std::unique_ptr<ApplicationContext> context_;
  std::unique_ptr<ContentClient> content_client_;
  std::unique_ptr<ContentBrowserClient> browser_client_;
  std::unique_ptr<ContentRendererClient> renderer_client_;
  std::unique_ptr<ContentUtilityClient> utility_client_;
  bool run_called_ = false;
  ShutdownPhase shutdown_phase_ = ShutdownPhase::kReady;
  bool shutdown_requested_ = false;
  std::string runtime_configuration_;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_MAIN_DELEGATE_H_
