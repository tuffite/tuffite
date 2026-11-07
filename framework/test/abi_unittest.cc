// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <array>
#include <optional>
#include <thread>
#include <utility>

#include "base/files/file_util.h"
#include "base/files/scoped_temp_dir.h"
#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/memory/raw_ptr.h"
#include "base/run_loop.h"
#include "base/strings/string_number_conversions.h"
#include "base/task/single_thread_task_runner.h"
#include "base/test/task_environment.h"
#include "testing/gtest/include/gtest/gtest.h"
#include "tuffite/components/dispatch/browser/rpc.h"
#include "tuffite/components/dispatch/common/codec.h"
#include "tuffite/framework/app/abi/application.h"
#include "tuffite/framework/app/abi/host.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/main_delegate.h"
#include "tuffite/framework/app/process_launch.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/manifest/manifest_service_factory.h"
#include "tuffite/framework/app/services/remoting/remoting_service.h"
#include "tuffite/framework/app/services/shortcut/shortcut_service.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/app/services/utility/utility_service_factory.h"
#include "tuffite/framework/app/services/window/window_service.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"
#include "tuffite/framework/browser/services/isolated/isolated_service_impl.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service_factory.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service.h"
#include "url/url_util.h"

namespace tuffite {
class ShellApiServiceFactoryTestPeer {
 public:
  static void Clear() { ShellApiServiceFactory::Get()->services_.clear(); }
  static void Register(content::BrowserContext* browser_context,
                       ApplicationContext& application,
                       base::WeakPtr<ShellApiService> service) {
    ShellApiServiceFactory::Get()->services_.insert_or_assign(
        browser_context,
        ShellApiServiceFactory::RegisteredService{&application, service});
  }
};

namespace {
std::unique_ptr<ManifestService> LoadManifest(
    const base::FilePath& root,
    std::string* error,
    const app::StartupEnvironment& environment = {},
    const app::BrowserOptions& options = {}) {
  auto definition = app::ManifestLoader::Load(root, error);
  if (!definition) {
    return nullptr;
  }
  auto configuration = app::ConfigurationAssembler::Build(
      std::move(*definition), environment, options, error);
  return configuration
             ? ManifestServiceFactory::Create(std::move(*configuration))
             : nullptr;
}

constexpr std::string_view kTestManifest = R"({
  "app": {
    "identifier": "com.tuffite.servicetest",
    "schemes": ["servicetest"],
    "windows": [
      {
        "label": "main",
        "url": "servicetest://main/index.html",
        "width": 800,
        "height": 600
      }
    ],
    "shellApi": {
      "commandIds": {
        "tuffite.Path.Directories": "0000000000000000"
      }
    },
    "network": {},
    "assets": [
      {
        "origins": [
          "servicetest://main"
        ],
        "paths": [
          "/*"
        ],
        "directory": "ui"
      }
    ]
  },
  "browser": {
    "rules": [{
      "identifier": "main",
      "origins": ["servicetest://main"],
      "shellApi": ["tuffite.Path.Directories"]
    }]
  }
})";

std::unique_ptr<ManifestService> TestManifest() {
  base::ScopedTempDir package;
  if (!package.CreateUniqueTempDir() ||
      !base::WriteFile(package.GetPath().AppendASCII("tuffite.json"),
                       kTestManifest)) {
    ADD_FAILURE() << "Could not create test manifest";
    return nullptr;
  }
  std::string error;
  auto manifest = LoadManifest(package.GetPath(), &error);
  EXPECT_TRUE(manifest) << error;
  return manifest;
}

class AbiTest : public testing::Test {
 protected:
  void SetUp() override {
    url::AddStandardScheme("servicetest", url::SCHEME_WITH_HOST);
  }
  url::ScopedSchemeRegistryForTests registry_;
};

// Version-zero empty Mojom request struct: num_bytes=8, version=0.

void RunPendingBrowserTasks() {
  // Observe a barrier after previously posted browser tasks. Negative
  // assertions below must also hold after those tasks have had a chance to run.
  base::RunLoop barrier;
  base::SingleThreadTaskRunner::GetCurrentDefault()->PostTask(
      FROM_HERE, barrier.QuitClosure());
  barrier.Run();
}

struct ReplyState {
  int replies = 0;
  tuffite_status_t status = TUFFITE_STATUS_OK;
};
tuffite_host callbacks;
ReplyState state;

class RecordingShellApiService final : public ShellApiService {
 public:
  bool IsAuthorized(content::RenderFrameHost&) const override { return true; }
  bool IsEventAuthorized(content::RenderFrameHost&) const override {
    return true;
  }
  void Dispatch(content::RenderFrameHost&,
                const std::string&,
                const std::vector<uint8_t>&,
                const base::UnguessableToken&,
                Reply) override {}
  void DispatchRequest(content::RenderFrameHost&,
                       uint64_t,
                       const std::string&,
                       const std::vector<uint8_t>&,
                       const base::UnguessableToken&,
                       Reply) override {}
  void CancelRequest(const base::UnguessableToken&, uint64_t) override {}
  void CancelDocument(const base::UnguessableToken&) override {}
  void Subscribe(const base::UnguessableToken&, Event) override {}
  base::WeakPtr<ShellApiService> GetWeakPtr() override {
    return weak_factory_.GetWeakPtr();
  }
  void Broadcast(const std::string& name,
                 const std::vector<uint8_t>& bytes) override {
    ++broadcasts;
    last_name = name;
    last_bytes = bytes;
  }

  int broadcasts = 0;
  std::string last_name;
  std::vector<uint8_t> last_bytes;

 private:
  base::WeakPtrFactory<RecordingShellApiService> weak_factory_{this};
};
tuffite_status_t Reply(void* data,
                       tuffite_status_t status,
                       tuffite_byte_slice_t) {
  auto* reply = static_cast<ReplyState*>(data);
  ++reply->replies;
  reply->status = status;
  return TUFFITE_STATUS_OK;
}

const tuffite_utility_session_client& SessionClient() {
  const auto* table = callbacks.query_extension(
      callbacks.user_data, TUFFITE_CAPABILITY_UTILITY_SESSION);
  return *reinterpret_cast<const tuffite_utility_session_client*>(table);
}

tuffite_status_t FailingStart(const tuffite_host* framework,
                              const uint8_t*,
                              size_t,
                              tuffite_handle_t*) {
  callbacks = *framework;
  const uint8_t name[] = {'d', 'e', 'm', 'o'};
  tuffite_status_t accepted = TUFFITE_STATUS_INVALID_STATE;
  std::thread worker([&] {
    accepted =
        SessionClient().send(SessionClient().user_data, name, sizeof(name), 1,
                             1, nullptr, 0, &state, &Reply);
  });
  worker.join();
  EXPECT_EQ(accepted, TUFFITE_STATUS_OK);
  return TUFFITE_STATUS_INVALID_STATE;
}

tuffite_application FakeApplication() {
  static const tuffite_utility_session_service session = {
      .header =
          tuffite_abi_header_for_size(sizeof(tuffite_utility_session_service)),
      .create = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
      .send = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
      .acknowledge = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
      .cancel = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
      .close = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
  };
  return {
      .header = tuffite_abi_header_for_size(sizeof(tuffite_application)),
      .start = &FailingStart,
      .dispatch = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
      .stop = [](auto...) { return TUFFITE_STATUS_OK; },
      .utility =
          {
              .is_registered =
                  [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
              .start = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
              .invoke = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
              .cancel = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; },
              .stop = [](auto...) { return TUFFITE_STATUS_OK; },
              .session = &session,
          },
      .prepare_browser = nullptr,
  };
}

TEST(BrowserLaunchTest, PreparesBrowserButSkipsAllChildren) {
  auto application = FakeApplication();
  application.prepare_browser = [](const tuffite_byte_slice_t* args,
                                   size_t count, void* context,
                                   tuffite_launch_argument_fn emit) {
    for (size_t i = 0; i < count; ++i) {
      // SAFETY: PrepareProcessLaunch owns the argument array and keeps its
      // entries alive throughout this synchronous test callback.
      auto status = emit(context, UNSAFE_BUFFERS(args[i]));
      if (status != TUFFITE_STATUS_OK) {
        return status;
      }
    }
    const uint8_t extra[] = {'-', '-', 'b', 'u', 's', 'i', 'n', 'e', 's', 's'};
    return emit(context, {extra, sizeof(extra)});
  };
  std::vector<std::string> browser = {"app", "--original"};
  ASSERT_TRUE(PrepareProcessLaunch(application, &browser));
  EXPECT_EQ(browser,
            (std::vector<std::string>{"app", "--original", "--business"}));
  for (const auto* role : {"utility", "renderer", "gpu-process"}) {
    std::vector<std::string> child = {"app", std::string("--type=") + role};
    auto original = child;
    EXPECT_TRUE(PrepareProcessLaunch(application, &child));
    EXPECT_EQ(child, original);
  }
}

TEST(BrowserLaunchTest, RejectsRoleChangesAndPreservesOriginalArguments) {
  auto application = FakeApplication();
  application.prepare_browser = [](const tuffite_byte_slice_t* args, size_t,
                                   void* context,
                                   tuffite_launch_argument_fn emit) {
    // SAFETY: PrepareProcessLaunch owns the argument array and keeps its
    // entries alive throughout this synchronous test callback.
    if (emit(context, UNSAFE_BUFFERS(args[0])) != TUFFITE_STATUS_OK) {
      return TUFFITE_STATUS_INTERNAL;
    }
    const uint8_t role[] = "--type=utility";
    return emit(context, {role, sizeof(role) - 1});
  };
  std::vector<std::string> args = {"app", "--original"};
  EXPECT_FALSE(PrepareProcessLaunch(application, &args));
  EXPECT_EQ(args, (std::vector<std::string>{"app", "--original"}));
  application.prepare_browser = nullptr;
  EXPECT_TRUE(PrepareProcessLaunch(application, &args));
}

TEST(ProcessLaunchTest, PreparesEveryRoleAndPrefersTheProcessHook) {
  auto application = FakeApplication();
  application.prepare_browser = [](auto...) { return TUFFITE_STATUS_INTERNAL; };
  application.prepare_process = [](uint32_t kind,
                                   const tuffite_byte_slice_t* args,
                                   size_t count, void* context,
                                   tuffite_launch_argument_fn emit) {
    std::string role = "--observed-role=" + base::NumberToString(kind);
    for (size_t i = 0; i < count; ++i) {
      // SAFETY: PrepareProcessLaunch owns the argument array and keeps its
      // entries alive throughout this synchronous test callback.
      auto status = emit(context, UNSAFE_BUFFERS(args[i]));
      if (status != TUFFITE_STATUS_OK) {
        return status;
      }
    }
    return emit(context,
                {reinterpret_cast<const uint8_t*>(role.data()), role.size()});
  };
  uint32_t kind = 0;
  for (const auto* role :
       {"", "utility", "renderer", "gpu-process", "other-child"}) {
    std::vector<std::string> args = {"app"};
    if (*role) {
      args.push_back(std::string("--type=") + role);
    }
    ASSERT_TRUE(PrepareProcessLaunch(application, &args));
    EXPECT_EQ(args.back(), "--observed-role=" + base::NumberToString(kind++));
  }
  application.prepare_process = [](uint32_t, const tuffite_byte_slice_t* args,
                                   size_t, void* context,
                                   tuffite_launch_argument_fn emit) {
    // SAFETY: PrepareProcessLaunch owns the argument array and keeps its
    // entries alive throughout this synchronous test callback.
    emit(context, UNSAFE_BUFFERS(args[0]));
    const uint8_t role[] = "--type=renderer";
    return emit(context, {role, sizeof(role) - 1});
  };
  std::vector<std::string> args = {"app", "--type=utility"};
  EXPECT_FALSE(PrepareProcessLaunch(application, &args));
  EXPECT_EQ(args, (std::vector<std::string>{"app", "--type=utility"}));
}

TEST_F(AbiTest, PreservesResolverWithoutReadingStartupContextTail) {
  auto application = FakeApplication();
  application.header.struct_size =
      offsetof(tuffite_application, utility_start_with_context);
  application.header.version = tuffite_abi_current_version();
  application.resolve_v8 = [](auto...) { return TUFFITE_STATUS_OK; };
  application.utility_start_with_context = [](auto...) {
    return TUFFITE_STATUS_INTERNAL;
  };
  auto host = abi::Application::Create(&application);
  ASSERT_TRUE(host);
  EXPECT_NE(host->table().resolve_v8, nullptr);
  EXPECT_EQ(host->table().utility_start_with_context, nullptr);
}

TEST_F(AbiTest, OptionalUtilityRouteTailPreservesStartupContext) {
  auto application = FakeApplication();
  application.utility_start_with_context = [](auto...) {
    return TUFFITE_STATUS_OK;
  };
  application.resolve_utility_route = [](auto...) {
    return TUFFITE_STATUS_NOT_FOUND;
  };
  auto host = abi::Application::Create(&application);
  ASSERT_TRUE(host);
  EXPECT_NE(host->table().resolve_utility_route, nullptr);
  application.header.struct_size =
      offsetof(tuffite_application, resolve_utility_route);
  host = abi::Application::Create(&application);
  ASSERT_TRUE(host);
  EXPECT_NE(host->table().utility_start_with_context, nullptr);
  EXPECT_EQ(host->table().resolve_utility_route, nullptr);
}

TEST_F(AbiTest, OptionalSetupTailPreservesOlderApplicationTables) {
  auto application = FakeApplication();
  application.resolve_utility_route = [](auto...) {
    return TUFFITE_STATUS_NOT_FOUND;
  };
  application.setup_browser = [](void*, tuffite_launch_argument_fn) {
    return TUFFITE_STATUS_OK;
  };
  auto host = abi::Application::Create(&application);
  ASSERT_TRUE(host);
  EXPECT_NE(host->table().setup_browser, nullptr);
  application.header.struct_size = offsetof(tuffite_application, setup_browser);
  host = abi::Application::Create(&application);
  ASSERT_TRUE(host);
  EXPECT_EQ(host->table().setup_browser, nullptr);
  EXPECT_NE(host->table().resolve_utility_route, nullptr);
}

TEST_F(AbiTest, ShortcutRegistrationHasScopeAndDuplicateOwnership) {
  base::test::TaskEnvironment environment;
  auto table = FakeApplication();
  MainDelegate delegate(abi::Application::Create(&table));
  delegate.BindBrowserTaskRunner();
  ASSERT_TRUE(delegate.StartUp(TestManifest()));
  auto* shortcuts = delegate.context()->GetService<ShortcutService>();
  auto callback = base::BindRepeating([](std::string, int64_t) {});
  ui::Accelerator key(ui::VKEY_P, ui::EF_CONTROL_DOWN | ui::EF_SHIFT_DOWN);
  EXPECT_TRUE(shortcuts->RegisterShortcut("first", key, 0, callback));
  EXPECT_FALSE(shortcuts->RegisterShortcut("duplicate", key, 0, callback));
  EXPECT_FALSE(
      shortcuts->RegisterShortcut("missing-window", key, 999, callback));
  EXPECT_TRUE(shortcuts->IsShortcutRegistered("first"));
  shortcuts->UnregisterShortcuts(0);
  EXPECT_FALSE(shortcuts->IsShortcutRegistered("first"));
  EXPECT_FALSE(shortcuts->UnregisterShortcut("first"));
  EXPECT_TRUE(shortcuts->RegisterShortcut("again", key, 0, callback));
  delegate.Shutdown();
}

TEST_F(AbiTest, BrowserInitializationCompletesOnBrowserSequence) {
  base::test::TaskEnvironment environment;
  auto application = FakeApplication();
  application.start = [](const tuffite_host*, const uint8_t*, size_t,
                         tuffite_handle_t* runtime) {
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  application.initialize_browser = [](tuffite_handle_t runtime, void* data,
                                      tuffite_utility_reply_fn reply) {
    EXPECT_EQ(runtime, 1u);
    std::thread worker(
        [=] { reply(data, TUFFITE_STATUS_OK, {.data = nullptr, .len = 0}); });
    worker.join();
    return TUFFITE_STATUS_OK;
  };
  auto host =
      std::make_unique<MainDelegate>(abi::Application::Create(&application));
  ASSERT_TRUE(host);
  host->BindBrowserTaskRunner();
  ASSERT_TRUE(host->StartUp(TestManifest()));
  bool completed = false;
  host->application().InitializeBrowser(base::BindOnce(
      [](bool* completed, bool ok) {
        EXPECT_TRUE(ok);
        *completed = true;
      },
      &completed));
  EXPECT_FALSE(completed);
  environment.RunUntilIdle();
  EXPECT_TRUE(completed);
}

TEST_F(AbiTest, BrowserInitializationFailureIsReported) {
  base::test::TaskEnvironment environment;
  auto application = FakeApplication();
  application.start = [](const tuffite_host*, const uint8_t*, size_t,
                         tuffite_handle_t* runtime) {
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  application.initialize_browser = [](auto...) {
    return TUFFITE_STATUS_INVALID_STATE;
  };
  auto host =
      std::make_unique<MainDelegate>(abi::Application::Create(&application));
  ASSERT_TRUE(host);
  host->BindBrowserTaskRunner();
  ASSERT_TRUE(host->StartUp(TestManifest()));
  bool completed = false;
  host->application().InitializeBrowser(base::BindOnce(
      [](bool* completed, bool ok) {
        EXPECT_FALSE(ok);
        *completed = true;
      },
      &completed));
  environment.RunUntilIdle();
  EXPECT_TRUE(completed);
}

TEST_F(AbiTest, RejectsIncompatibleAbiMajor) {
  auto application = FakeApplication();
  application.header.version.major ^= 1;
  EXPECT_FALSE(abi::Application::Create(&application));
}

TEST_F(AbiTest, AcceptsServiceStartupContext) {
  auto application = FakeApplication();
  application.utility_start_with_context = [](auto...) {
    return TUFFITE_STATUS_OK;
  };
  auto host = abi::Application::Create(&application);
  ASSERT_TRUE(host);
  EXPECT_NE(host->table().utility_start_with_context, nullptr);
}

TEST_F(AbiTest, PreservesBrowserHookWithoutReadingProcessTail) {
  auto application = FakeApplication();
  application.header.struct_size =
      offsetof(tuffite_application, prepare_process);
  application.header.version = tuffite_abi_current_version();
  application.prepare_browser = [](auto...) { return TUFFITE_STATUS_OK; };
  application.prepare_process = [](auto...) { return TUFFITE_STATUS_INTERNAL; };
  application.utility_runtime = [](auto...) { return TUFFITE_STATUS_INTERNAL; };
  auto host = abi::Application::Create(&application);
  ASSERT_TRUE(host);
  EXPECT_NE(host->table().prepare_browser, nullptr);
  EXPECT_EQ(host->table().prepare_process, nullptr);
  EXPECT_EQ(host->table().utility_runtime, nullptr);
}

TEST_F(AbiTest, AcceptsOriginalApplicationPrefixWithoutStartupHook) {
  auto application = FakeApplication();
  application.header.struct_size =
      offsetof(tuffite_application, prepare_browser);
  application.header.version = tuffite_abi_current_version();
  application.prepare_browser = [](auto...) { return TUFFITE_STATUS_INTERNAL; };
  auto host = abi::Application::Create(&application);
  ASSERT_TRUE(host);
  EXPECT_EQ(host->table().prepare_browser, nullptr);
}

// Hold an accepted ABI reply until service shutdown, without launching a
// process.
class PendingUtility final : public UtilityService {
 public:
  explicit PendingUtility(std::vector<std::string>& order) : order_(order) {}
  ~PendingUtility() override { order_->push_back("service destroyed"); }
  void SetUtilityRegistrationChecker(UtilityRegistrationChecker) override {}
  bool IsRegisteredUtility(std::string_view) const override { return true; }
  void SetUtilityInterfaceResolver(UtilityInterfaceResolver) override {}
  std::string ResolveUtilityInterface(std::string_view) const override {
    return {};
  }
  void InvokeUtilityService(const base::UnguessableToken&,
                            uint64_t,
                            std::string,
                            std::vector<uint8_t>,
                            UtilityResponseCallback) override {}
  void CancelUtilityService(const base::UnguessableToken&,
                            std::string,
                            uint64_t) override {}
  void CancelUtilityOwner(const base::UnguessableToken&) override {}
  void CreateUtilitySession(std::string,
                            uint64_t,
                            std::vector<uint8_t>,
                            uint32_t,
                            uint32_t,
                            UtilitySessionEventCallback,
                            UtilitySessionClosedCallback,
                            base::OnceCallback<void(int32_t)>) override {}
  void SendUtilitySession(std::string_view,
                          uint64_t,
                          uint64_t,
                          std::vector<uint8_t>,
                          UtilityResponseCallback callback) override {
    pending_ = std::move(callback);
  }
  void AcknowledgeUtilitySession(std::string_view,
                                 uint64_t,
                                 uint64_t,
                                 uint32_t) override {}
  void CancelUtilitySessionCommand(std::string_view,
                                   uint64_t,
                                   uint64_t) override {}
  void CloseUtilitySession(std::string_view,
                           uint64_t,
                           uint32_t,
                           UtilityResponseCallback) override {}
  void Shutdown() override {
    if (stopped_) {
      return;
    }
    stopped_ = true;
    order_->push_back("service shutdown");
    if (pending_) {
      std::move(pending_).Run(TUFFITE_STATUS_INVALID_STATE, {});
    }
  }

 private:
  const raw_ref<std::vector<std::string>> order_;
  UtilityResponseCallback pending_;
  bool stopped_ = false;
};

TEST_F(AbiTest, CompletesServiceRepliesBeforeStoppingRust) {
  base::test::TaskEnvironment environment;
  std::vector<std::string> order;
  static std::vector<std::string>* observed;
  observed = &order;
  auto application = FakeApplication();
  application.start = [](const tuffite_host* framework, const uint8_t*, size_t,
                         tuffite_handle_t* runtime) {
    callbacks = *framework;
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  application.stop = [](tuffite_handle_t runtime) {
    EXPECT_EQ(runtime, 1u);
    observed->push_back("runtime stop");
    ReplyState rejected;
    const uint8_t name[] = {'d', 'e', 'm', 'o'};
    EXPECT_EQ(
        SessionClient().send(SessionClient().user_data, name, sizeof(name), 1,
                             2, nullptr, 0, &rejected, &Reply),
        TUFFITE_STATUS_INVALID_STATE);
    EXPECT_EQ(rejected.replies, 0);
    // Cleanup tasks remain accepted until the synchronous stop completes.
    static int task_data;
    EXPECT_EQ(callbacks.post_task(
                  callbacks.user_data, TUFFITE_TASK_PRIORITY_USER_VISIBLE, 0,
                  &task_data, [](void*) {}, [](void*) {}),
              TUFFITE_STATUS_OK);
    return TUFFITE_STATUS_OK;
  };
  auto host =
      std::make_unique<MainDelegate>(abi::Application::Create(&application));
  ASSERT_TRUE(host);
  host->BindBrowserTaskRunner();
  ASSERT_TRUE(host->StartUp(TestManifest()));
  auto context = host->context()->GetWeakPtr();
  auto manifest = host->context()->GetService<ManifestService>()->GetWeakPtr();
  UtilityServiceFactory::Get()->SetTestingFactoryForApplication(
      host->context(),
      base::BindOnce(
          [](std::vector<std::string>* order,
             ApplicationContext&) -> std::unique_ptr<UtilityService> {
            return std::make_unique<PendingUtility>(*order);
          },
          &order));
  const uint8_t name[] = {'d', 'e', 'm', 'o'};
  ASSERT_EQ(
      SessionClient().send(
          SessionClient().user_data, name, sizeof(name), 1, 1, nullptr, 0,
          &order,
          [](void* data, tuffite_status_t status, tuffite_byte_slice_t) {
            EXPECT_EQ(status, TUFFITE_STATUS_INVALID_STATE);
            static_cast<std::vector<std::string>*>(data)->push_back("reply");
            return TUFFITE_STATUS_OK;
          }),
      TUFFITE_STATUS_OK);
  environment.RunUntilIdle();
  EXPECT_TRUE(order.empty());
  host->Shutdown();
  EXPECT_EQ(order,
            (std::vector<std::string>{"service shutdown", "reply",
                                      "service destroyed", "runtime stop"}));
  EXPECT_FALSE(context);
  EXPECT_FALSE(manifest);
  EXPECT_EQ(host->context(), nullptr);
  EXPECT_FALSE(host->application().is_running());
  host->Shutdown();
  host.reset();
  environment.RunUntilIdle();
  EXPECT_EQ(order.size(), 4u);
  observed = nullptr;
}

TEST_F(AbiTest,
       ApplicationScopeDrainsUtilitiesAndRetainsServicesDuringRustStop) {
  base::test::TaskEnvironment environment;
  std::vector<std::string> order;
  static std::vector<std::string>* observed;
  base::WeakPtr<const ManifestService> manifest;
  static base::WeakPtr<const ManifestService>* observed_manifest;
  observed_manifest = &manifest;
  observed = &order;
  auto application = FakeApplication();
  application.start = [](const tuffite_host* framework, const uint8_t*, size_t,
                         tuffite_handle_t* runtime) {
    callbacks = *framework;
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  application.start_profile = [](tuffite_handle_t, const tuffite_host*,
                                 tuffite_byte_slice_t, tuffite_byte_slice_t,
                                 tuffite_handle_t*) {
    return TUFFITE_STATUS_INVALID_STATE;
  };
  application.stop = [](tuffite_handle_t) {
    EXPECT_EQ(*observed,
              (std::vector<std::string>{"service shutdown", "reply"}));
    EXPECT_TRUE(*observed_manifest);
    observed->push_back("runtime stop");
    return TUFFITE_STATUS_OK;
  };
  MainDelegate host(abi::Application::Create(&application));
  host.BindBrowserTaskRunner();
  ASSERT_TRUE(host.StartUp(TestManifest()));
  manifest = host.context()->GetService<ManifestService>()->GetWeakPtr();
  UtilityServiceFactory::Get()->SetTestingFactoryForApplication(
      host.context(),
      base::BindOnce(
          [](std::vector<std::string>* events,
             ApplicationContext&) -> std::unique_ptr<UtilityService> {
            return std::make_unique<PendingUtility>(*events);
          },
          &order));
  const uint8_t name[] = {'d', 'e', 'm', 'o'};
  ASSERT_EQ(
      SessionClient().send(
          SessionClient().user_data, name, sizeof(name), 1, 1, nullptr, 0,
          &order,
          [](void* data, tuffite_status_t status, tuffite_byte_slice_t) {
            EXPECT_EQ(status, TUFFITE_STATUS_INVALID_STATE);
            static_cast<std::vector<std::string>*>(data)->push_back("reply");
            return TUFFITE_STATUS_OK;
          }),
      TUFFITE_STATUS_OK);
  environment.RunUntilIdle();
  EXPECT_TRUE(order.empty());
  host.Shutdown();
  EXPECT_EQ(order,
            (std::vector<std::string>{"service shutdown", "reply",
                                      "runtime stop", "service destroyed"}));
  EXPECT_FALSE(manifest);
  host.Shutdown();
  EXPECT_EQ(order.size(), 4u);
  observed = nullptr;
  observed_manifest = nullptr;
}

TEST_F(AbiTest, FailedStartupStopsPublishedRuntimeAfterServices) {
  base::test::TaskEnvironment environment;
  auto manifest = TestManifest();
  ASSERT_TRUE(manifest);
  auto manifest_weak = manifest->GetWeakPtr();
  static const base::WeakPtr<const ManifestService>* observed_manifest;
  static int stops;
  observed_manifest = &manifest_weak;
  stops = 0;
  auto application = FakeApplication();
  application.start = [](const tuffite_host* framework, const uint8_t*, size_t,
                         tuffite_handle_t*) {
    callbacks = *framework;
    EXPECT_EQ(framework->post_event(framework->user_data, 7,
                                    TUFFITE_RUNTIME_EVENT_STARTED, {}),
              TUFFITE_STATUS_OK);
    return TUFFITE_STATUS_INTERNAL;
  };
  application.stop = [](tuffite_handle_t runtime) {
    EXPECT_EQ(runtime, 7u);
    EXPECT_FALSE(*observed_manifest);
    ++stops;
    return TUFFITE_STATUS_OK;
  };
  auto host =
      std::make_unique<MainDelegate>(abi::Application::Create(&application));
  ASSERT_TRUE(host);
  EXPECT_FALSE(host->StartUp(std::move(manifest)));
  EXPECT_EQ(host->context(), nullptr);
  EXPECT_FALSE(host->application().is_running());
  EXPECT_EQ(stops, 1);
  host->Shutdown();
  host.reset();
  EXPECT_EQ(stops, 1);
  observed_manifest = nullptr;
}

TEST_F(AbiTest, ShutdownReentryDrainsAcceptedRepliesBeforeRustStop) {
  base::test::TaskEnvironment environment;
  struct ShutdownState {
    raw_ptr<MainDelegate> delegate = nullptr;
    int replies = 0;
    int stops = 0;
  } shutdown;
  static ShutdownState* observed;
  observed = &shutdown;
  auto table = FakeApplication();
  table.start = [](const tuffite_host* host, const uint8_t*, size_t,
                   tuffite_handle_t* runtime) {
    callbacks = *host;
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  table.stop = [](tuffite_handle_t runtime) {
    EXPECT_EQ(runtime, 1u);
    EXPECT_EQ(observed->replies, 2);
    ++observed->stops;
    // Service teardown and Rust stop may also reenter the owner.
    observed->delegate->Shutdown();
    return TUFFITE_STATUS_OK;
  };
  MainDelegate delegate(abi::Application::Create(&table));
  shutdown.delegate = &delegate;
  delegate.BindBrowserTaskRunner();
  ASSERT_TRUE(delegate.StartUp(TestManifest()));
  const uint8_t name[] = {'d', 'e', 'm', 'o'};
  for (uint64_t id : {1u, 2u}) {
    ASSERT_EQ(
        SessionClient().send(
            SessionClient().user_data, name, sizeof(name), 1, id, nullptr, 0,
            &shutdown,
            [](void* data, tuffite_status_t status, tuffite_byte_slice_t) {
              auto& shutdown = *static_cast<ShutdownState*>(data);
              EXPECT_EQ(status, TUFFITE_STATUS_INVALID_STATE);
              EXPECT_EQ(shutdown.stops, 0);
              if (++shutdown.replies == 1) {
                shutdown.delegate->Shutdown();
              }
              return TUFFITE_STATUS_OK;
            }),
        TUFFITE_STATUS_OK);
  }
  // The calls are accepted but still queued when closing begins.
  delegate.BeginShutdown();
  EXPECT_EQ(shutdown.replies, 2);
  EXPECT_EQ(shutdown.stops, 1);
  EXPECT_EQ(delegate.context(), nullptr);
  environment.RunUntilIdle();
  delegate.Shutdown();
  EXPECT_EQ(shutdown.replies, 2);
  EXPECT_EQ(shutdown.stops, 1);
  observed = nullptr;
}

TEST_F(AbiTest, RuntimeEventsRequireActiveApplicationLifetime) {
  base::test::TaskEnvironment environment;
  static int stops;
  stops = 0;
  auto table = FakeApplication();
  table.start = [](const tuffite_host* host, const uint8_t*, size_t,
                   tuffite_handle_t* runtime) {
    callbacks = *host;
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  table.stop = [](tuffite_handle_t runtime) {
    EXPECT_EQ(runtime, 1u);
    ++stops;
    // Even synchronous events emitted by stop cannot republish a handle.
    EXPECT_EQ(callbacks.post_event(callbacks.user_data, 1,
                                   TUFFITE_RUNTIME_EVENT_STARTED, {}),
              TUFFITE_STATUS_INVALID_STATE);
    return TUFFITE_STATUS_OK;
  };
  MainDelegate delegate(abi::Application::Create(&table));
  const auto& host = delegate.host().table();
  EXPECT_EQ(
      host.post_event(host.user_data, 1, TUFFITE_RUNTIME_EVENT_STARTED, {}),
      TUFFITE_STATUS_INVALID_STATE);
  delegate.BindBrowserTaskRunner();
  ASSERT_TRUE(delegate.StartUp(TestManifest()));
  EXPECT_EQ(
      host.post_event(host.user_data, 1, TUFFITE_RUNTIME_EVENT_STARTED, {}),
      TUFFITE_STATUS_OK);
  delegate.Shutdown();
  EXPECT_EQ(
      host.post_event(host.user_data, 7, TUFFITE_RUNTIME_EVENT_STARTED, {}),
      TUFFITE_STATUS_INVALID_STATE);
  EXPECT_FALSE(delegate.application().is_running());
  delegate.Shutdown();
  EXPECT_EQ(stops, 1);
}

TEST_F(AbiTest, StartupRejectsMismatchingPublishedHandle) {
  base::test::TaskEnvironment environment;
  static int stops;
  stops = 0;
  auto table = FakeApplication();
  table.start = [](const tuffite_host* host, const uint8_t*, size_t,
                   tuffite_handle_t* runtime) {
    EXPECT_EQ(
        host->post_event(host->user_data, 7, TUFFITE_RUNTIME_EVENT_STARTED, {}),
        TUFFITE_STATUS_OK);
    *runtime = 9;
    return TUFFITE_STATUS_OK;
  };
  table.stop = [](tuffite_handle_t runtime) {
    EXPECT_EQ(runtime, 7u);
    ++stops;
    return TUFFITE_STATUS_OK;
  };
  MainDelegate delegate(abi::Application::Create(&table));
  EXPECT_FALSE(delegate.StartUp(TestManifest()));
  EXPECT_EQ(delegate.context(), nullptr);
  EXPECT_FALSE(delegate.application().is_running());
  delegate.Shutdown();
  EXPECT_EQ(stops, 1);
}

TEST_F(AbiTest, BrowserInitializationRequiresBoundTaskRunner) {
  base::test::TaskEnvironment environment;
  auto table = FakeApplication();
  table.start = [](const tuffite_host*, const uint8_t*, size_t,
                   tuffite_handle_t* runtime) {
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  table.initialize_browser = [](tuffite_handle_t, void*,
                                tuffite_utility_reply_fn) {
    ADD_FAILURE() << "Initialization must wait for the browser task runner";
    return TUFFITE_STATUS_INTERNAL;
  };
  MainDelegate delegate(abi::Application::Create(&table));
  ASSERT_TRUE(delegate.StartUp(TestManifest()));
  bool completed = false;
  delegate.application().InitializeBrowser(base::BindOnce(
      [](bool* completed, bool ok) {
        EXPECT_FALSE(ok);
        *completed = true;
      },
      &completed));
  EXPECT_TRUE(completed);
  delegate.Shutdown();
}

TEST_F(AbiTest, BackendScopesRouteAndCancelWithTheirOriginatingDocument) {
  base::test::TaskEnvironment environment;
  std::vector<uint64_t> runtimes;
  static std::vector<uint64_t>* dispatched;
  dispatched = &runtimes;
  auto table = FakeApplication();
  table.start = [](const tuffite_host*, const uint8_t*, size_t,
                   tuffite_handle_t* out) {
    *out = 1;
    return TUFFITE_STATUS_OK;
  };
  table.start_profile = [](tuffite_handle_t, const tuffite_host*,
                           tuffite_byte_slice_t, tuffite_byte_slice_t,
                           tuffite_handle_t* out) {
    *out = 2;
    return TUFFITE_STATUS_OK;
  };
  table.command_scope = [](uint64_t command) -> uint32_t {
    return command == 1 || command == 2 ? static_cast<uint32_t>(command) : 0;
  };
  table.dispatch = [](tuffite_handle_t runtime, const uint8_t*, size_t) {
    dispatched->push_back(runtime);
    return TUFFITE_STATUS_OK;
  };
  MainDelegate parent(abi::Application::Create(&table));
  parent.BindBrowserTaskRunner();
  ASSERT_TRUE(parent.StartUp(TestManifest()));
  auto child = abi::Application::Create(&table);
  abi::Host child_host(base::BindRepeating(&abi::Application::OnEvent,
                                           base::Unretained(child.get())));
  child_host.BindBrowserTaskRunner();
  child_host.BindApplicationContext(parent.context());
  child->BindBrowserTaskRunner();
  auto utilities = CreateUtilityService();
  ASSERT_TRUE(child->StartProfile(parent.application(), child_host.table(),
                                  *parent.context(), "work", *utilities));
  const auto document = base::UnguessableToken::Create();
  int completed = 0;
  bool rejected_framework = false;
  EXPECT_FALSE(child->Dispatch(
      rpc::EncodeRequest({.command_id = UINT64_MAX}),
      base::BindOnce([](bool* rejected, bool ok,
                        std::vector<uint8_t>) { *rejected = !ok; },
                     &rejected_framework),
      document));
  EXPECT_TRUE(rejected_framework);
  for (uint64_t command : {1u, 2u}) {
    auto request =
        rpc::EncodeRequest({.request_id = command, .command_id = command});
    EXPECT_TRUE(child->Dispatch(
        std::move(request),
        base::BindOnce(
            [](int* completed, bool, std::vector<uint8_t>) { ++*completed; },
            &completed),
        document));
  }
  EXPECT_EQ(runtimes, (std::vector<uint64_t>{1, 2}));
  int rejected = 0;
  auto rejection = [&rejected] {
    return base::BindOnce(
        [](int* count, bool ok, std::vector<uint8_t>) {
          EXPECT_FALSE(ok);
          ++*count;
        },
        &rejected);
  };
  EXPECT_FALSE(parent.application().Dispatch(
      rpc::EncodeRequest({.request_id = 3, .command_id = 2}), rejection(),
      document));
  EXPECT_FALSE(
      child->Dispatch(rpc::EncodeRequest({.request_id = 4, .command_id = 99}),
                      rejection(), document));
  EXPECT_EQ(rejected, 2);
  EXPECT_EQ(runtimes.size(), 2u);
  child->CancelDocument(document);
  auto response = rpc::EncodeResponse({.request_id = 1});
  const tuffite_byte_slice_t bytes{response.data(), response.size()};
  EXPECT_EQ(parent.application().OnEvent(
                1, TUFFITE_RUNTIME_EVENT_COMMAND_RESPONSE, bytes),
            TUFFITE_STATUS_OK);
  EXPECT_EQ(child->OnEvent(2, TUFFITE_RUNTIME_EVENT_COMMAND_RESPONSE, bytes),
            TUFFITE_STATUS_OK);
  environment.RunUntilIdle();
  EXPECT_EQ(completed, 0);
  child->Quiesce();
  EXPECT_FALSE(
      child->Dispatch(rpc::EncodeRequest({.request_id = 5, .command_id = 1}),
                      rejection(), document));
  EXPECT_EQ(rejected, 3);
  EXPECT_EQ(runtimes.size(), 2u);
  child_host.Close();
  utilities->Shutdown();
  child->Stop();
  child_host.Shutdown();
  EXPECT_EQ(child_host.table().post_event(
                child_host.table().user_data, 2,
                TUFFITE_RUNTIME_EVENT_COMMAND_RESPONSE, bytes),
            TUFFITE_STATUS_INVALID_STATE);
}

TEST_F(AbiTest, ApplicationEventsCrossHostBoundaryAndStopAtQuiesce) {
  base::test::TaskEnvironment environment;
  auto table = FakeApplication();
  table.start = [](const tuffite_host* framework, const uint8_t*, size_t,
                   tuffite_handle_t* runtime) {
    callbacks = *framework;
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  MainDelegate application(abi::Application::Create(&table));
  application.BindBrowserTaskRunner();
  ASSERT_TRUE(application.StartUp(TestManifest()));
  RecordingShellApiService shell;
  ShellApiServiceFactoryTestPeer::Clear();
  ShellApiServiceFactoryTestPeer::Register(
      reinterpret_cast<content::BrowserContext*>(1), *application.context(),
      shell.GetWeakPtr());
  std::vector<uint8_t> event = {4, 0, 'd', 'e', 'm', 'o', 9};
  EXPECT_EQ(callbacks.post_event(callbacks.user_data, 2,
                                 TUFFITE_RUNTIME_EVENT_APPLICATION_EVENT,
                                 {event.data(), event.size()}),
            TUFFITE_STATUS_INVALID_HANDLE);
  EXPECT_EQ(callbacks.post_event(callbacks.user_data, 1,
                                 TUFFITE_RUNTIME_EVENT_APPLICATION_EVENT,
                                 {event.data(), event.size()}),
            TUFFITE_STATUS_OK);
  // Native payload ownership ends at post_event return, before UI dispatch.
  event.assign(event.size(), 0);
  environment.RunUntilIdle();
  EXPECT_EQ(shell.broadcasts, 1);
  EXPECT_EQ(shell.last_name, "demo");
  EXPECT_EQ(shell.last_bytes, (std::vector<uint8_t>{9}));
  application.BeginShutdown();
  event = {4, 0, 'd', 'e', 'm', 'o', 8};
  EXPECT_EQ(callbacks.post_event(callbacks.user_data, 1,
                                 TUFFITE_RUNTIME_EVENT_APPLICATION_EVENT,
                                 {event.data(), event.size()}),
            TUFFITE_STATUS_OK);
  environment.RunUntilIdle();
  EXPECT_EQ(shell.broadcasts, 1);
  ShellApiServiceFactoryTestPeer::Clear();
  application.Shutdown();
}

TEST_F(AbiTest, FailedStartupCompletesQueuedHostCallOnce) {
  base::test::SingleThreadTaskEnvironment environment;
  state = {};
  auto application = FakeApplication();
  auto host =
      std::make_unique<MainDelegate>(abi::Application::Create(&application));
  ASSERT_TRUE(host);
  host->BindBrowserTaskRunner();
  EXPECT_FALSE(host->StartUp(TestManifest()));
  EXPECT_EQ(host->context(), nullptr);
  EXPECT_EQ(state.replies, 1);
  EXPECT_EQ(state.status, TUFFITE_STATUS_INVALID_STATE);
  RunPendingBrowserTasks();
  EXPECT_EQ(state.replies, 1);
  host.reset();
  RunPendingBrowserTasks();
  EXPECT_EQ(state.replies, 1);
}

TEST_F(AbiTest, QuiescingRejectsHostCallWithoutTakingReplyOwnership) {
  base::test::SingleThreadTaskEnvironment environment;
  state = {};
  auto application = FakeApplication();
  auto host =
      std::make_unique<MainDelegate>(abi::Application::Create(&application));
  ASSERT_TRUE(host);
  host->BindBrowserTaskRunner();
  EXPECT_FALSE(host->StartUp(TestManifest()));
  ReplyState rejected;
  const uint8_t name[] = {'d', 'e', 'm', 'o'};
  EXPECT_EQ(SessionClient().send(SessionClient().user_data, name, sizeof(name),
                                 1, 2, nullptr, 0, &rejected, &Reply),
            TUFFITE_STATUS_INVALID_STATE);
  RunPendingBrowserTasks();
  EXPECT_EQ(rejected.replies, 0);
}

TEST_F(AbiTest, CapabilityTablesAreStableAndUnknownIdsAreUnsupported) {
  base::test::SingleThreadTaskEnvironment environment;
  auto application = FakeApplication();
  auto host =
      std::make_unique<MainDelegate>(abi::Application::Create(&application));
  ASSERT_TRUE(host);
  host->BindBrowserTaskRunner();
  EXPECT_FALSE(host->StartUp(TestManifest()));
  const auto* header = callbacks.query_extension(
      callbacks.user_data, TUFFITE_CAPABILITY_UTILITY_SESSION);
  ASSERT_NE(header, nullptr);
  EXPECT_EQ(header->struct_size, sizeof(tuffite_utility_session_client));
  const auto* isolated_header = callbacks.query_extension(
      callbacks.user_data, TUFFITE_CAPABILITY_ISOLATED);
  ASSERT_NE(isolated_header, nullptr);
  EXPECT_EQ(isolated_header->struct_size, sizeof(tuffite_isolated));
  const auto* isolated =
      reinterpret_cast<const tuffite_isolated*>(isolated_header);
  EXPECT_NE(isolated->apply, nullptr);
  EXPECT_EQ(isolated->user_data, callbacks.user_data);
  const auto* interfaces = callbacks.query_extension(
      callbacks.user_data, TUFFITE_CAPABILITY_UTILITY_INTERFACES);
  ASSERT_NE(interfaces, nullptr);
  EXPECT_EQ(interfaces->struct_size, sizeof(tuffite_abi_header));
  EXPECT_EQ(callbacks.query_extension(callbacks.user_data,
                                      TUFFITE_CAPABILITY_UTILITY_SESSION),
            header);
  EXPECT_EQ(callbacks.query_extension(callbacks.user_data, UINT32_MAX),
            nullptr);
  EXPECT_EQ(SessionClient().header.struct_size,
            sizeof(tuffite_utility_session_client));
  EXPECT_EQ(SessionClient().user_data, callbacks.user_data);
}

TEST_F(AbiTest, RemotingCapabilityCancelsQueuedCallsOnceAndRejectsAfterClose) {
  base::test::SingleThreadTaskEnvironment environment;
  abi::Host host(base::BindRepeating(
      [](tuffite_handle_t, tuffite_runtime_event_kind_t, tuffite_byte_slice_t) {
        return TUFFITE_STATUS_OK;
      }));
  host.BindBrowserTaskRunner();
  const auto& table = host.table();
  const auto* header =
      table.query_extension(table.user_data, TUFFITE_CAPABILITY_REMOTING);
  ASSERT_NE(header, nullptr);
  ASSERT_EQ(header->struct_size, sizeof(tuffite_remoting));
  const auto* remoting = reinterpret_cast<const tuffite_remoting*>(header);
  struct Result {
    int count = 0;
    int32_t status = 0;
  } result;
  auto reply = [](void* data, int32_t status,
                  tuffite_byte_slice_t) -> tuffite_status_t {
    auto& result = *static_cast<Result*>(data);
    ++result.count;
    result.status = status;
    return TUFFITE_STATUS_OK;
  };
  EXPECT_EQ(remoting->capabilities(remoting->user_data, {}, &result, reply),
            TUFFITE_STATUS_OK);
  host.Close();
  EXPECT_EQ(result.count, 1);
  EXPECT_EQ(result.status, static_cast<int32_t>(rpc::Status::kUnavailable));
  environment.RunUntilIdle();
  EXPECT_EQ(result.count, 1);
  EXPECT_EQ(remoting->capabilities(remoting->user_data, {}, &result, reply),
            TUFFITE_STATUS_INVALID_STATE);
  EXPECT_EQ(result.count, 1);
}

TEST_F(AbiTest,
       IsolatedCapabilityRejectsUnboundContextAndCancelsQueuedCallsOnce) {
  base::test::SingleThreadTaskEnvironment environment;
  auto application = FakeApplication();
  application.start = [](const tuffite_host* framework, const uint8_t*, size_t,
                         tuffite_handle_t* runtime) {
    callbacks = *framework;
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  auto host =
      std::make_unique<MainDelegate>(abi::Application::Create(&application));
  ASSERT_TRUE(host);
  host->BindBrowserTaskRunner();
  ASSERT_TRUE(host->StartUp(TestManifest()));
  const auto* header = callbacks.query_extension(callbacks.user_data,
                                                 TUFFITE_CAPABILITY_ISOLATED);
  ASSERT_NE(header, nullptr);
  auto table = *reinterpret_cast<const tuffite_isolated*>(header);
  struct Result {
    int calls = 0;
    tuffite_status_t status = TUFFITE_STATUS_OK;
  } result;
  auto reply = [](void* data, tuffite_status_t status,
                  uint64_t generation) -> tuffite_status_t {
    auto* completion = static_cast<Result*>(data);
    ++completion->calls;
    completion->status = status;
    EXPECT_EQ(generation, 0u);
    return TUFFITE_STATUS_OK;
  };
  const uint8_t request[] = {'{', '}'};
  EXPECT_EQ(
      table.apply(table.user_data, {.data = request, .len = sizeof(request)},
                  &result, reply),
      TUFFITE_STATUS_OK);
  EXPECT_EQ(result.calls, 0);
  environment.RunUntilIdle();
  EXPECT_EQ(result.calls, 1);
  EXPECT_EQ(result.status, TUFFITE_STATUS_INVALID_STATE);
  result.calls = 0;
  EXPECT_EQ(
      table.apply(table.user_data, {.data = request, .len = sizeof(request)},
                  &result, reply),
      TUFFITE_STATUS_OK);
  host->BeginShutdown();
  EXPECT_EQ(result.calls, 1);
  environment.RunUntilIdle();
  EXPECT_EQ(result.calls, 1);
  EXPECT_EQ(
      table.apply(table.user_data, {.data = request, .len = sizeof(request)},
                  &result, reply),
      TUFFITE_STATUS_INVALID_STATE);
  EXPECT_EQ(result.calls, 1);
}

TEST_F(AbiTest, RejectsMalformedServiceSessionPrefix) {
  auto application = FakeApplication();
  auto session = *application.utility.session;
  application.utility.session = &session;
  session.header.struct_size = sizeof(session.header);
  EXPECT_EQ(abi::Application::Create(&application), nullptr);
  session.header = tuffite_abi_header_for_size(sizeof(session));
  session.header.reserved = 1;
  EXPECT_EQ(abi::Application::Create(&application), nullptr);
  session.header.reserved = 0;
  session.header.version.major += 1;
  EXPECT_EQ(abi::Application::Create(&application), nullptr);
  application.utility.session = nullptr;
  EXPECT_EQ(abi::Application::Create(&application), nullptr);
}

TEST(ApplicationServiceTest, IsolatesServicesAcrossApplications) {
  url::ScopedSchemeRegistryForTests registry;
  url::AddStandardScheme("servicetest", url::SCHEME_WITH_HOST);
  // Application services such as remoting create ThreadPool task runners.
  base::test::TaskEnvironment environment;
  base::ScopedTempDir package;
  ASSERT_TRUE(package.CreateUniqueTempDir());

  ASSERT_TRUE(base::WriteFile(package.GetPath().AppendASCII("tuffite.json"),
                              kTestManifest));
  std::string error;
  auto manifest = LoadManifest(package.GetPath(), &error);
  ASSERT_TRUE(manifest) << error;
  auto second_manifest =
      LoadManifest(package.GetPath(), &error,
                   {.development_url = GURL("http://localhost:5173/")});
  ASSERT_TRUE(second_manifest) << error;

  auto callbacks_a = FakeApplication();
  callbacks_a.start = [](const tuffite_host*, const uint8_t*, size_t,
                         tuffite_handle_t* runtime) {
    *runtime = 1;
    return TUFFITE_STATUS_OK;
  };
  auto callbacks_b = callbacks_a;
  MainDelegate application_a(abi::Application::Create(&callbacks_a));
  MainDelegate application_b(abi::Application::Create(&callbacks_b));
  ASSERT_TRUE(application_a.StartUp(std::move(manifest)));
  ASSERT_TRUE(application_b.StartUp(std::move(second_manifest)));
  ASSERT_NE(application_a.context(), nullptr);
  ASSERT_NE(application_b.context(), nullptr);
  EXPECT_NE(application_a.context()->GetService<ManifestService>(),
            application_b.context()->GetService<ManifestService>());
  auto* manifest_a =
      application_a.context()->GetService<ManifestService>(false);
  auto* manifest_b =
      application_b.context()->GetService<ManifestService>(false);
  ASSERT_NE(manifest_a, nullptr);
  ASSERT_NE(manifest_b, nullptr);
  {
    IsolatedServiceImpl first(*manifest_a);
    IsolatedServiceImpl second(*manifest_a);
    auto registry_request = *base::JSONReader::ReadDict(
        R"({"op":"register","name":"dynamic","source":"snapshot","origins":["https://example.com"]})",
        base::JSON_PARSE_RFC);
    const auto registry_origin =
        url::Origin::Create(GURL("https://example.com"));
    ASSERT_TRUE(first.Apply(registry_request));
    EXPECT_EQ(first.Source("dynamic", registry_origin), "snapshot");
    EXPECT_FALSE(second.Source("dynamic", registry_origin));
    auto client = first.GetWeakPtr();
    ASSERT_TRUE(client);
    first.Shutdown();
    EXPECT_FALSE(client);
    EXPECT_FALSE(first.Source("dynamic", registry_origin));
    EXPECT_FALSE(first.Apply(registry_request));
    EXPECT_TRUE(second.Apply(registry_request));
    first.Shutdown();
  }
  EXPECT_EQ(manifest_a->app_info().identifier, "com.tuffite.servicetest");
  EXPECT_EQ(manifest_a->startup_windows().front().url,
            GURL("servicetest://main/index.html"));
  EXPECT_EQ(manifest_b->startup_windows().front().url,
            GURL("http://localhost:5173/"));
  const auto dev_origin = url::Origin::Create(GURL("http://localhost:5173/"));
  EXPECT_FALSE(manifest_a->IsShellApiOriginAllowed(dev_origin));
  EXPECT_TRUE(manifest_b->IsShellApiOriginAllowed(dev_origin));
  EXPECT_FALSE(manifest_b->IsShellApiOriginAllowed(
      url::Origin::Create(GURL("https://untrusted.example/"))));
  auto resource_snapshot = manifest_a->assets();
  auto manifest_weak = manifest_a->GetWeakPtr();

  auto* utility_a = application_a.context()->GetService<UtilityService>();
  auto* utility_b = application_b.context()->GetService<UtilityService>();
  EXPECT_NE(utility_a, utility_b);
  EXPECT_NE(application_a.context()->GetService<remoting::RemotingService>(),
            application_b.context()->GetService<remoting::RemotingService>());
  EXPECT_EQ(application_a.context()
                ->GetService<remoting::RemotingService>()
                ->GetCapabilities()
                .status,
            remoting::Status::kOk);

  // Native Remoting reaches the typed application service without a frame or
  // builtin renderer router. Malformed wire bodies still fail closed.
  {
    abi::Host native(base::BindRepeating(
        [](tuffite_handle_t, tuffite_runtime_event_kind_t,
           tuffite_byte_slice_t) { return TUFFITE_STATUS_OK; }));
    native.BindBrowserTaskRunner();
    native.BindApplicationContext(application_a.context());
    const auto& table = native.table();
    const auto* remoting = reinterpret_cast<const tuffite_remoting*>(
        table.query_extension(table.user_data, TUFFITE_CAPABILITY_REMOTING));
    ASSERT_NE(remoting, nullptr);
    struct Result {
      int count = 0;
      int32_t status = -1;
      size_t length = 0;
    } result;
    auto reply = [](void* data, int32_t status,
                    tuffite_byte_slice_t bytes) -> tuffite_status_t {
      auto& result = *static_cast<Result*>(data);
      ++result.count;
      result.status = status;
      result.length = bytes.len;
      return TUFFITE_STATUS_OK;
    };
    constexpr std::array<uint8_t, 8> empty = {8, 0, 0, 0, 0, 0, 0, 0};
    EXPECT_EQ(
        remoting->capabilities(remoting->user_data,
                               {empty.data(), empty.size()}, &result, reply),
        TUFFITE_STATUS_OK);
    environment.RunUntilIdle();
    EXPECT_EQ(result.count, 1);
    EXPECT_EQ(result.status, static_cast<int32_t>(rpc::Status::kOk));
    EXPECT_GT(result.length, 8u);
    EXPECT_EQ(remoting->capabilities(remoting->user_data, {}, &result, reply),
              TUFFITE_STATUS_OK);
    environment.RunUntilIdle();
    EXPECT_EQ(result.count, 2);
    EXPECT_EQ(result.status,
              static_cast<int32_t>(rpc::Status::kInvalidArgument));
    EXPECT_EQ(result.length, 0u);
  }

  RecordingShellApiService shell_a;
  RecordingShellApiService shell_b;
  ShellApiServiceFactoryTestPeer::Clear();
  ShellApiServiceFactoryTestPeer::Register(
      reinterpret_cast<content::BrowserContext*>(1), *application_a.context(),
      shell_a.GetWeakPtr());
  ShellApiServiceFactoryTestPeer::Register(
      reinterpret_cast<content::BrowserContext*>(2), *application_b.context(),
      shell_b.GetWeakPtr());
  ShellApiServiceFactory::BroadcastEvent(*application_a.context(),
                                         "application-a", {1, 2, 3});
  EXPECT_EQ(shell_a.broadcasts, 1);
  EXPECT_EQ(shell_a.last_name, "application-a");
  EXPECT_EQ(shell_a.last_bytes, (std::vector<uint8_t>{1, 2, 3}));
  EXPECT_EQ(shell_b.broadcasts, 0);
  ShellApiServiceFactoryTestPeer::Clear();

  application_b.Shutdown();
  EXPECT_EQ(application_b.context(), nullptr);
  EXPECT_EQ(application_a.context()->GetService<UtilityService>(), utility_a);
  application_a.Shutdown();
  EXPECT_EQ(application_a.context(), nullptr);
  EXPECT_FALSE(manifest_weak);
  const auto asset =
      resource_snapshot.ResolveAssetFile(GURL("servicetest://main/index.html"));
  ASSERT_TRUE(asset);
  EXPECT_EQ(asset->NormalizePathSeparators(), package.GetPath()
                                                  .AppendASCII("assets")
                                                  .AppendASCII("ui")
                                                  .AppendASCII("index.html")
                                                  .NormalizePathSeparators());
  EXPECT_FALSE(resource_snapshot.ResolveAssetFile(
      GURL("https://untrusted.example/index.html")));
  RunPendingBrowserTasks();
}

TEST(ApplicationTest, ShutdownBeforeBrowserStartupIsIdempotent) {
  base::test::SingleThreadTaskEnvironment environment;
  auto application_callbacks = FakeApplication();
  auto host = abi::Application::Create(&application_callbacks);
  ASSERT_TRUE(host);
  {
    tuffite::MainDelegate application(std::move(host));
    EXPECT_EQ(application.context(), nullptr);
    application.Shutdown();
    application.Shutdown();
    EXPECT_EQ(application.context(), nullptr);
  }
  RunPendingBrowserTasks();
}

}  // namespace
}  // namespace tuffite
