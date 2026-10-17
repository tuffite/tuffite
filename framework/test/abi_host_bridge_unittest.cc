// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/abi_host_bridge.h"

#include <thread>
#include <utility>

#include "base/files/file_util.h"
#include "base/files/scoped_temp_dir.h"
#include "base/functional/bind.h"
#include "base/run_loop.h"
#include "base/strings/string_number_conversions.h"
#include "base/task/single_thread_task_runner.h"
#include "base/test/task_environment.h"
#include "testing/gtest/include/gtest/gtest.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/main_delegate.h"
#include "tuffite/framework/app/process_launch.h"
#include "tuffite/framework/app/services/asset/asset_service.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/manifest/manifest_service_factory.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/services/context_menu/context_menu_service.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
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
  const auto* table = callbacks.query_extension(callbacks.user_data,
                                                TUFFITE_CAPABILITY_BROWSER);
  const auto* browser = reinterpret_cast<const tuffite_browser_host*>(table);
  return *browser->utility->session;
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

TEST(AbiHostBridgeTest, PreservesAbi24ResolverWithoutReadingAbi25Tail) {
  auto application = FakeApplication();
  application.header.struct_size =
      offsetof(tuffite_application, utility_start_with_context);
  application.header.version = {2, 4, 0};
  application.resolve_v8 = [](auto...) { return TUFFITE_STATUS_OK; };
  application.utility_start_with_context = [](auto...) {
    return TUFFITE_STATUS_INTERNAL;
  };
  auto bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  EXPECT_NE(bridge->application().resolve_v8, nullptr);
  EXPECT_EQ(bridge->application().utility_start_with_context, nullptr);
}

TEST(AbiHostBridgeTest, AcceptsAbi25ServiceStartupContext) {
  auto application = FakeApplication();
  application.utility_start_with_context = [](auto...) {
    return TUFFITE_STATUS_OK;
  };
  auto bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  EXPECT_NE(bridge->application().utility_start_with_context, nullptr);
}

TEST(AbiHostBridgeTest, PreservesAbi21HookWithoutReadingAbi23Tail) {
  auto application = FakeApplication();
  application.header.struct_size =
      offsetof(tuffite_application, prepare_process);
  application.header.version = {2, 1, 0};
  application.prepare_browser = [](auto...) { return TUFFITE_STATUS_OK; };
  application.prepare_process = [](auto...) { return TUFFITE_STATUS_INTERNAL; };
  application.utility_runtime = [](auto...) { return TUFFITE_STATUS_INTERNAL; };
  auto bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  EXPECT_NE(bridge->application().prepare_browser, nullptr);
  EXPECT_EQ(bridge->application().prepare_process, nullptr);
  EXPECT_EQ(bridge->application().utility_runtime, nullptr);
}

TEST(AbiHostBridgeTest, AcceptsOriginalApplicationPrefixWithoutStartupHook) {
  auto application = FakeApplication();
  application.header.struct_size =
      offsetof(tuffite_application, prepare_browser);
  application.header.version = {2, 0, 0};
  application.prepare_browser = [](auto...) { return TUFFITE_STATUS_INTERNAL; };
  auto bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  EXPECT_EQ(bridge->application().prepare_browser, nullptr);
}

TEST(AbiHostBridgeTest, FailedStartupCompletesQueuedHostCallOnce) {
  base::test::SingleThreadTaskEnvironment environment;
  state = {};
  auto application = FakeApplication();
  auto bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  bridge->BindBrowserTaskRunner();
  EXPECT_FALSE(bridge->Start("{}"));
  EXPECT_EQ(state.replies, 1);
  EXPECT_EQ(state.status, TUFFITE_STATUS_INVALID_STATE);
  RunPendingBrowserTasks();
  EXPECT_EQ(state.replies, 1);
  bridge.reset();
  RunPendingBrowserTasks();
  EXPECT_EQ(state.replies, 1);
}
TEST(AbiHostBridgeTest, QuiescingRejectsHostCallWithoutTakingReplyOwnership) {
  base::test::SingleThreadTaskEnvironment environment;
  state = {};
  auto application = FakeApplication();
  auto bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  bridge->BindBrowserTaskRunner();
  EXPECT_FALSE(bridge->Start("{}"));
  ReplyState rejected;
  const uint8_t name[] = {'d', 'e', 'm', 'o'};
  EXPECT_EQ(SessionClient().send(SessionClient().user_data, name, sizeof(name),
                                 1, 2, nullptr, 0, &rejected, &Reply),
            TUFFITE_STATUS_INVALID_STATE);
  RunPendingBrowserTasks();
  EXPECT_EQ(rejected.replies, 0);
}
TEST(AbiHostBridgeTest, CapabilityTablesAreStableAndUnknownIdsAreUnsupported) {
  base::test::SingleThreadTaskEnvironment environment;
  auto application = FakeApplication();
  auto bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  bridge->BindBrowserTaskRunner();
  EXPECT_FALSE(bridge->Start("{}"));
  const auto* header = callbacks.query_extension(callbacks.user_data,
                                                 TUFFITE_CAPABILITY_BROWSER);
  ASSERT_NE(header, nullptr);
  EXPECT_EQ(header->struct_size, sizeof(tuffite_browser_host));
  const auto* browser = reinterpret_cast<const tuffite_browser_host*>(header);
  ASSERT_NE(browser->utility, nullptr);
  ASSERT_NE(browser->shell_api, nullptr);
  EXPECT_EQ(browser->utility->header.struct_size, sizeof(tuffite_utility));
  EXPECT_EQ(browser->shell_api->header.struct_size, sizeof(tuffite_shell_api));
  EXPECT_EQ(browser->shell_api->user_data, callbacks.user_data);
  EXPECT_NE(browser->shell_api->invoke, nullptr);
  EXPECT_EQ(callbacks.query_extension(callbacks.user_data,
                                      TUFFITE_CAPABILITY_BROWSER),
            header);
  EXPECT_EQ(callbacks.query_extension(callbacks.user_data, UINT32_MAX),
            nullptr);
  EXPECT_EQ(SessionClient().header.struct_size,
            sizeof(tuffite_utility_session_client));
  EXPECT_EQ(SessionClient().user_data, callbacks.user_data);
}
TEST(AbiHostBridgeTest, RejectsMalformedServiceSessionPrefix) {
  auto application = FakeApplication();
  auto session = *application.utility.session;
  application.utility.session = &session;
  session.header.struct_size = sizeof(session.header);
  EXPECT_EQ(AbiHostBridge::Create(&application), nullptr);
  session.header = tuffite_abi_header_for_size(sizeof(session));
  session.header.reserved = 1;
  EXPECT_EQ(AbiHostBridge::Create(&application), nullptr);
  session.header.reserved = 0;
  session.header.version.major += 1;
  EXPECT_EQ(AbiHostBridge::Create(&application), nullptr);
  application.utility.session = nullptr;
  EXPECT_EQ(AbiHostBridge::Create(&application), nullptr);
}
TEST(AbiHostBridgeTest, QueuedShellApiRepliesOnceWhenStartupFails) {
  base::test::SingleThreadTaskEnvironment environment;
  struct ShellReply {
    int calls = 0;
    int32_t status = -1;
  };
  ShellReply result;
  auto application = FakeApplication();
  application.start = [](const tuffite_host* host, const uint8_t*, size_t,
                         tuffite_handle_t*) {
    callbacks = *host;
    return TUFFITE_STATUS_OK;
  };
  auto bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  bridge->BindBrowserTaskRunner();
  // No runtime handle is published, so Start fails and quiesces pending calls.
  // A dedicated startup hook submits before returning an error.
  static ShellReply* pending;
  pending = &result;
  application.start = [](const tuffite_host* host, const uint8_t*, size_t,
                         tuffite_handle_t*) {
    const auto* browser = reinterpret_cast<const tuffite_browser_host*>(
        host->query_extension(host->user_data, TUFFITE_CAPABILITY_BROWSER));
    const auto* api = browser->shell_api;
    constexpr std::string_view path = "tuffite.remoting.capabilities";
    EXPECT_EQ(api->invoke(api->user_data,
                          reinterpret_cast<const uint8_t*>(path.data()),
                          path.size(), nullptr, 0, pending,
                          [](void* data, int32_t status, tuffite_byte_slice_t) {
                            auto* result = static_cast<ShellReply*>(data);
                            ++result->calls;
                            result->status = status;
                            return TUFFITE_STATUS_OK;
                          }),
              TUFFITE_STATUS_OK);
    return TUFFITE_STATUS_INVALID_STATE;
  };
  bridge = AbiHostBridge::Create(&application);
  ASSERT_TRUE(bridge);
  bridge->BindBrowserTaskRunner();
  EXPECT_FALSE(bridge->Start("{}"));
  RunPendingBrowserTasks();
  EXPECT_EQ(result.calls, 1);
  EXPECT_EQ(result.status, static_cast<int32_t>(rpc::Status::kUnavailable));
  pending = nullptr;
}
TEST(ApplicationServiceTest, IsolatesServicesAcrossApplications) {
  url::ScopedSchemeRegistryForTests registry;
  url::AddStandardScheme("servicetest", url::SCHEME_WITH_HOST);
  // Application services such as remoting create ThreadPool task runners.
  base::test::TaskEnvironment environment;
  base::ScopedTempDir package;
  ASSERT_TRUE(package.CreateUniqueTempDir());
  constexpr std::string_view kManifest = R"({
  "app": {
    "identifier": "com.tuffite.servicetest",
    "security": {
      "capabilities": [{
        "identifier": "main",
        "origins": ["servicetest://main"],
        "shellApi": ["tuffite.path.directories"]
      }]
    },
    "windows": [
      {
        "label": "main",
        "url": "servicetest://main/index.html",
        "width": 800,
        "height": 600
      }
    ],
    "shellApi": {
      "namespace": "servicetest",
      "commandIds": {
        "tuffite.path.directories": "0000000000000000"
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
  }
})";
  ASSERT_TRUE(base::WriteFile(package.GetPath().AppendASCII("tuffite.json"),
                              kManifest));
  std::string error;
  auto manifest = ManifestServiceFactory::Load(package.GetPath(), &error);
  ASSERT_TRUE(manifest) << error;
  auto second_manifest = ManifestServiceFactory::Load(
      package.GetPath(), &error, GURL("http://localhost:5173/"));
  ASSERT_TRUE(second_manifest) << error;

  auto callbacks_a = FakeApplication();
  callbacks_a.start = [](auto...) { return TUFFITE_STATUS_INVALID_STATE; };
  auto callbacks_b = callbacks_a;
  MainDelegate application_a(AbiHostBridge::Create(&callbacks_a));
  MainDelegate application_b(AbiHostBridge::Create(&callbacks_b));
  EXPECT_FALSE(application_a.StartUp(std::move(manifest)));
  EXPECT_FALSE(application_b.StartUp(std::move(second_manifest)));
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
  auto* assets_a = application_a.context()->GetService<AssetService>();
  auto* assets_b = application_b.context()->GetService<AssetService>();
  ASSERT_NE(assets_a, nullptr);
  ASSERT_NE(assets_b, nullptr);
  EXPECT_NE(assets_a, assets_b);

  auto* utility_a = application_a.context()->GetService<UtilityService>();
  auto* utility_b = application_b.context()->GetService<UtilityService>();
  EXPECT_NE(utility_a, utility_b);

  rpc::Router router(shell_api::BuiltinRoutes());
  dispatch::Context native(*application_a.context(),
                           base::UnguessableToken::Create());
  int32_t reply_status = -1;
  rpc::Reply reply =
      base::BindOnce([](int32_t* output, int32_t status,
                        const std::vector<uint8_t>&) { *output = status; },
                     &reply_status);
  EXPECT_TRUE(
      router.Dispatch(native, "tuffite.remoting.capabilities", {}, &reply));
  EXPECT_EQ(reply_status, static_cast<int32_t>(rpc::Status::kOk));
  reply = base::BindOnce([](int32_t* output, int32_t status,
                            const std::vector<uint8_t>&) { *output = status; },
                         &reply_status);
  EXPECT_TRUE(router.Dispatch(native, "tuffite.window.current", {}, &reply));
  EXPECT_EQ(reply_status, static_cast<int32_t>(rpc::Status::kInvalidArgument));

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
  EXPECT_EQ(application_a.context()->GetService<AssetService>(), assets_a);
  application_a.Shutdown();
  EXPECT_EQ(application_a.context(), nullptr);
  EXPECT_FALSE(manifest_weak);
  EXPECT_EQ(
      resource_snapshot.ResolveAssetFile(GURL("servicetest://main/index.html")),
      package.GetPath().AppendASCII("assets").AppendASCII("ui").AppendASCII(
          "index.html"));
  EXPECT_FALSE(resource_snapshot.ResolveAssetFile(
      GURL("https://untrusted.example/index.html")));
  RunPendingBrowserTasks();
}

TEST(ApplicationTest, ShutdownBeforeBrowserStartupIsIdempotent) {
  base::test::SingleThreadTaskEnvironment environment;
  auto application_callbacks = FakeApplication();
  auto bridge = AbiHostBridge::Create(&application_callbacks);
  ASSERT_TRUE(bridge);
  {
    tuffite::MainDelegate application(std::move(bridge));
    EXPECT_EQ(application.context(), nullptr);
    application.Shutdown();
    application.Shutdown();
    EXPECT_EQ(application.context(), nullptr);
  }
  RunPendingBrowserTasks();
}
}  // namespace
}  // namespace tuffite
