// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/network_context/network_context_service_factory.h"

#include "base/task/thread_pool.h"
#include "build/build_config.h"
#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "components/os_crypt/async/browser/os_crypt_async.h"
#include "components/prefs/pref_registry_simple.h"
#include "components/prefs/pref_service.h"
#include "components/prefs/pref_service_factory.h"
#include "content/public/browser/browser_context.h"
#include "services/network/public/cpp/cookie_encryption_provider_impl.h"
#include "services/network/public/cpp/transferable_directory.h"
#include "services/network/public/mojom/network_context.mojom.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/network_context/network_context_service.h"

#if BUILDFLAG(IS_MAC)
#include "tuffite/framework/browser/services/network_context/safe_storage_key_provider_mac.h"
#elif BUILDFLAG(IS_WIN)
#include "components/os_crypt/async/browser/dpapi_key_provider.h"
#include "components/os_crypt/async/browser/os_crypt_win.h"
#elif defined(TUFFITE_USE_SECRET_PORTAL)
#include "components/os_crypt/async/browser/secret_portal_key_provider.h"
#endif

namespace tuffite {
namespace {
class NetworkContextServiceImpl final : public NetworkContextService {
 public:
  explicit NetworkContextServiceImpl(content::BrowserContext* context)
      : path_(context->GetPath()), off_the_record_(context->IsOffTheRecord()) {
    if (off_the_record_) {
      return;
    }
    std::vector<std::pair<os_crypt_async::OSCryptAsync::Precedence,
                          std::unique_ptr<os_crypt_async::KeyProvider>>>
        providers;
#if BUILDFLAG(IS_MAC)
    const auto& manifest = BrowserContext::GetFrom(*context).manifest_service();
    providers.emplace_back(0, CreateSafeStorageKeyProvider(
                                  manifest.app_info().identifier,
                                  manifest.app_info().product_name, path_));
#elif BUILDFLAG(IS_WIN) || defined(TUFFITE_USE_SECRET_PORTAL)
    auto registry = base::MakeRefCounted<PrefRegistrySimple>();
#if BUILDFLAG(IS_WIN)
    os_crypt_async::RegisterLocalPrefs(registry.get());
#else
    os_crypt_async::SecretPortalKeyProvider::RegisterLocalPrefs(registry.get());
#endif
    PrefServiceFactory factory;
    if (!off_the_record_) {
      factory.SetUserPrefsFile(
          path_.AppendASCII("Network Local State"),
          base::ThreadPool::CreateSequencedTaskRunner(
              {base::MayBlock(), base::TaskShutdownBehavior::BLOCK_SHUTDOWN})
              .get());
    }
    local_state_ = factory.Create(registry);
#if BUILDFLAG(IS_WIN)
    if (os_crypt_async::Init(local_state_.get())) {
      providers.emplace_back(0,
                             std::make_unique<os_crypt_async::DPAPIKeyProvider>(
                                 local_state_.get()));
    }
#else
    providers.emplace_back(
        0, std::make_unique<os_crypt_async::SecretPortalKeyProvider>(
               local_state_.get()));
#endif
#endif
    has_key_provider_ = !providers.empty();
    crypt_ =
        std::make_unique<os_crypt_async::OSCryptAsync>(std::move(providers));
    cookie_encryption_ =
        std::make_unique<CookieEncryptionProviderImpl>(crypt_.get());
  }

  void Configure(bool in_memory,
                 const base::FilePath& relative_partition_path,
                 network::mojom::NetworkContextParams* params) override {
    params->enable_brotli = true;
    params->enable_zstd = true;
    if (in_memory || off_the_record_) {
      return;
    }
    const auto partition = path_.Append(relative_partition_path);
    params->file_paths = network::mojom::NetworkContextFilePaths::New();
    params->file_paths->data_directory = partition.AppendASCII("Network");
    params->file_paths->http_cache_directory.emplace(
        partition.AppendASCII("Cache"));
    if (has_key_provider_) {
      // Never enable plaintext or known-key fallback when OS key storage fails.
      params->enable_encrypted_cookies = true;
      params->file_paths->cookie_database_name =
          base::FilePath(FILE_PATH_LITERAL("Cookies"));
      params->cookie_encryption_provider = cookie_encryption_->BindNewRemote();
    }
  }

  void Shutdown() override {
    if (local_state_) {
      local_state_->CommitPendingWrite();
    }
  }

 private:
  const base::FilePath path_;
  const bool off_the_record_;
  bool has_key_provider_ = false;
  // The preferences and key provider outlive the Mojo encryption receivers.
  std::unique_ptr<PrefService> local_state_;
  std::unique_ptr<os_crypt_async::OSCryptAsync> crypt_;
  std::unique_ptr<CookieEncryptionProviderImpl> cookie_encryption_;
};
}  // namespace

NetworkContextServiceFactory* NetworkContextServiceFactory::Get() {
  static base::NoDestructor<NetworkContextServiceFactory> instance;
  return instance.get();
}
NetworkContextServiceFactory::NetworkContextServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteNetworkContext",
          BrowserContextDependencyManager::GetInstance()) {}
bool NetworkContextServiceFactory::ServiceIsCreatedWithBrowserContext() const {
  return true;
}

NetworkContextService* NetworkContextServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<NetworkContextService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
NetworkContextServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<NetworkContextServiceImpl>(context);
}
}  // namespace tuffite
