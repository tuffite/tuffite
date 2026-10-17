// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_MAIN_DELEGATE_H_
#define TUFFITE_FRAMEWORK_APP_MAIN_DELEGATE_H_

#include <memory>
#include <optional>

#include "build/build_config.h"
#include "content/public/app/content_main_delegate.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"

namespace tuffite {
class AbiHostBridge;
class ApplicationContext;
class ContentBrowserClient;
class ContentClient;
class ContentRendererClient;
class ContentUtilityClient;

class MainDelegate final : public content::ContentMainDelegate {
 public:
  explicit MainDelegate(std::unique_ptr<AbiHostBridge> bridge);
  int Run(int argc, const char* const* argv, void* sandbox_info);
  bool StartUp(std::unique_ptr<ManifestService> manifest);
  void Shutdown();
  AbiHostBridge& bridge() { return *bridge_; }
  ApplicationContext* context() { return context_.get(); }
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
#if BUILDFLAG(IS_MAC)
  static void RegisterNSApplication();
#endif

  std::unique_ptr<AbiHostBridge> bridge_;
  std::unique_ptr<ApplicationContext> context_;
  std::unique_ptr<ContentClient> content_client_;
  std::unique_ptr<ContentBrowserClient> browser_client_;
  std::unique_ptr<ContentRendererClient> renderer_client_;
  std::unique_ptr<ContentUtilityClient> utility_client_;
  bool run_called_ = false;
  bool stopped_ = false;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_MAIN_DELEGATE_H_
