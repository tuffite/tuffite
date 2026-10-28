// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/abi/application.h"

#include <algorithm>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

#include "base/check.h"
#include "base/check_op.h"
#include "base/compiler_specific.h"
#include "base/containers/span.h"
#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/logging.h"
#include "base/memory/ptr_util.h"
#include "base/task/sequenced_task_runner.h"
#include "base/task/thread_pool.h"
#include "base/time/time.h"
#include "tuffite/components/dispatch/browser/rpc.h"
#include "tuffite/components/dispatch/common/codec.h"
#include "tuffite/components/dispatch/common/status.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/utility/utility_service.h"

namespace tuffite::abi {
namespace {
static_assert(offsetof(tuffite_application, header) == 0);
constexpr size_t kMaxApplicationEventNameBytes = 256;
constexpr size_t kMaxApplicationEventPayloadBytes = 1024 * 1024;
struct ModuleResolutionReply {
  UtilityResponseCallback callback;
  scoped_refptr<base::SequencedTaskRunner> runner;
  std::optional<base::FilePath> asset_path;
};

void ReadModuleAsset(std::optional<base::FilePath> path,
                     UtilityResponseCallback callback) {
  base::ThreadPool::PostTaskAndReplyWithResult(
      FROM_HERE, {base::MayBlock()},
      base::BindOnce(
          [](std::optional<base::FilePath> path) -> std::optional<std::string> {
            std::string source;
            if (!path ||
                !base::ReadFileToStringWithMaxSize(*path, &source,
                                                   512 * 1024) ||
                source.empty()) {
              return std::nullopt;
            }
            return source;
          },
          std::move(path)),
      base::BindOnce(
          [](UtilityResponseCallback callback,
             std::optional<std::string> source) {
            std::vector<uint8_t> bytes;
            if (source) {
              bytes.assign(source->begin(), source->end());
            }
            std::move(callback).Run(
                source ? TUFFITE_STATUS_OK : TUFFITE_STATUS_INVALID_ARGUMENT,
                std::move(bytes));
          },
          std::move(callback)));
}

tuffite_status_t CaptureModuleResolution(void* data,
                                         tuffite_status_t status,
                                         tuffite_byte_slice_t source) {
  std::unique_ptr<ModuleResolutionReply> pending(
      static_cast<ModuleResolutionReply*>(data));
  std::vector<uint8_t> bytes;
  if (status == TUFFITE_STATUS_OK && source.data && source.len > 0 &&
      source.len <= 512 * 1024) {
    // SAFETY: The native ABI producer supplies the declared readable byte range
    // for this callback. Copy it before returning; never retain the raw
    // pointer.
    auto view = UNSAFE_BUFFERS(base::span(source.data, source.len));
    bytes.assign(view.begin(), view.end());
  } else if (status == TUFFITE_STATUS_OK) {
    status = TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  pending->runner->PostTask(
      FROM_HERE, base::BindOnce(
                     [](UtilityResponseCallback callback,
                        std::optional<base::FilePath> path,
                        tuffite_status_t status, std::vector<uint8_t> bytes) {
                       if (status == TUFFITE_STATUS_NOT_FOUND) {
                         ReadModuleAsset(std::move(path), std::move(callback));
                       } else {
                         std::move(callback).Run(status, std::move(bytes));
                       }
                     },
                     std::move(pending->callback),
                     std::move(pending->asset_path), status, std::move(bytes)));
  return TUFFITE_STATUS_OK;
}

void ConfigureUtilityHostRuntime(UtilityService& service,
                                 const ManifestService& manifest,
                                 const tuffite_application& application,
                                 tuffite_handle_t runtime) {
  if (application.resolve_utility_route) {
    service.SetUtilityInterfaceResolver(base::BindRepeating(
        [](tuffite_resolve_utility_route_fn resolve,
           std::string_view name) -> std::string {
          tuffite_byte_slice_t owner{};
          if (resolve(
                  {reinterpret_cast<const uint8_t*>(name.data()), name.size()},
                  &owner) != TUFFITE_STATUS_OK ||
              !owner.data || owner.len == 0 || owner.len > 128) {
            return {};
          }
          // SAFETY: The ABI producer returns application-lifetime readable
          // bytes.
          const auto bytes = UNSAFE_BUFFERS(base::span(owner.data, owner.len));
          if (std::ranges::find(bytes, uint8_t{0}) != bytes.end()) {
            return {};
          }
          return std::string(bytes.begin(), bytes.end());
        },
        application.resolve_utility_route));
  }
  service.SetUtilityModuleResolver(base::BindRepeating(
      [](asset::AssetConfiguration assets, tuffite_resolve_v8_fn resolve,
         tuffite_handle_t runtime, std::string name, uint64_t session_id,
         std::string url, UtilityResponseCallback callback) {
        if (!resolve) {
          ReadModuleAsset(assets.ResolveAssetFile(GURL(url)),
                          std::move(callback));
          return;
        }
        auto pending =
            std::make_unique<ModuleResolutionReply>(ModuleResolutionReply{
                std::move(callback),
                base::SequencedTaskRunner::GetCurrentDefault(),
                assets.ResolveAssetFile(GURL(url))});
        // Release before calling: an accepted reply may arrive
        // immediately on a worker.
        auto* context = pending.release();
        const auto status = resolve(
            runtime, session_id,
            {reinterpret_cast<const uint8_t*>(name.data()), name.size()},
            {reinterpret_cast<const uint8_t*>(url.data()), url.size()}, context,
            &CaptureModuleResolution);
        if (status != TUFFITE_STATUS_OK) {
          pending.reset(context);
          std::move(pending->callback).Run(status, {});
        }
      },
      manifest.assets(), application.resolve_v8, runtime));
  service.SetUtilityModuleCanceller(base::BindRepeating(
      [](tuffite_cancel_v8_session_fn cancel, tuffite_handle_t runtime,
         std::string name, uint64_t session_id) {
        if (cancel) {
          cancel(runtime,
                 {reinterpret_cast<const uint8_t*>(name.data()), name.size()},
                 session_id);
        }
      },
      application.cancel_v8_session, runtime));
  const tuffite_utility_is_registered_fn utility_is_registered =
      application.utility.is_registered;
  service.SetUtilityRegistrationChecker(base::BindRepeating(
      [](tuffite_utility_is_registered_fn checker, std::string_view name) {
        return checker && checker(reinterpret_cast<const uint8_t*>(name.data()),
                                  name.size()) == TUFFITE_STATUS_OK;
      },
      utility_is_registered));
}

std::vector<uint8_t> ErrorBytes(std::string_view message) {
  return std::vector<uint8_t>(message.begin(), message.end());
}

const char* StatusName(tuffite_status_t status) {
  switch (status) {
    case TUFFITE_STATUS_OK:
      return "OK";
    case TUFFITE_STATUS_INVALID_ARGUMENT:
      return "INVALID_ARGUMENT";
    case TUFFITE_STATUS_INCOMPATIBLE_ABI:
      return "INCOMPATIBLE_ABI";
    case TUFFITE_STATUS_INVALID_HANDLE:
      return "INVALID_HANDLE";
    case TUFFITE_STATUS_INVALID_STATE:
      return "INVALID_STATE";
    case TUFFITE_STATUS_RESOURCE_EXHAUSTED:
      return "RESOURCE_EXHAUSTED";
    case TUFFITE_STATUS_INTERNAL:
      return "INTERNAL";
    default:
      return "UNKNOWN";
  }
}

}  // namespace

std::unique_ptr<Application> Application::Create(
    const tuffite_application* application) {
  if (!application ||
      application->header.struct_size <
          offsetof(tuffite_application, prepare_browser) ||
      !tuffite_abi_is_compatible(application->header.version,
                                 tuffite_abi_baseline_version()) ||
      application->header.reserved != 0 || !application->start ||
      !application->dispatch || !application->stop ||
      !application->utility.is_registered || !application->utility.start ||
      !application->utility.invoke || !application->utility.cancel ||
      !application->utility.stop || !application->utility.session ||
      application->utility.session->header.struct_size <
          sizeof(tuffite_utility_session_service) ||
      !tuffite_abi_is_compatible(application->utility.session->header.version,
                                 tuffite_abi_baseline_version()) ||
      application->utility.session->header.reserved != 0 ||
      !application->utility.session->create ||
      !application->utility.session->send ||
      !application->utility.session->acknowledge ||
      !application->utility.session->cancel ||
      !application->utility.session->close) {
    return nullptr;
  }
  // Providers may expose only a prefix. Never read beyond their declared size.
  tuffite_application snapshot = {
      .header = application->header,
      .start = application->start,
      .dispatch = application->dispatch,
      .stop = application->stop,
      .utility = application->utility,
      .prepare_browser = nullptr,
      .prepare_process = nullptr,
      .utility_runtime = nullptr,
      .resolve_v8 = nullptr,
      .utility_start_with_context = nullptr,
      .resolve_utility_route = nullptr,
      .setup_browser = nullptr,
      .initialize_browser = nullptr,
      .cancel_v8_session = nullptr,
      .start_profile = nullptr,
      .command_scope = nullptr,
      .configuration = {},
  };
  const tuffite_abi_version startup_version = tuffite_abi_baseline_version();
  if (application->header.struct_size >=
          offsetof(tuffite_application, prepare_process) &&
      tuffite_abi_is_compatible(application->header.version, startup_version)) {
    snapshot.prepare_browser = application->prepare_browser;
  }
  const tuffite_abi_version process_version = tuffite_abi_baseline_version();
  if (application->header.struct_size >=
          offsetof(tuffite_application, resolve_v8) &&
      tuffite_abi_is_compatible(application->header.version, process_version)) {
    snapshot.prepare_process = application->prepare_process;
    snapshot.utility_runtime = application->utility_runtime;
  }
  const tuffite_abi_version resolver_version = tuffite_abi_baseline_version();
  if (application->header.struct_size >=
          offsetof(tuffite_application, utility_start_with_context) &&
      tuffite_abi_is_compatible(application->header.version,
                                resolver_version)) {
    snapshot.resolve_v8 = application->resolve_v8;
  }
  const tuffite_abi_version context_version = tuffite_abi_baseline_version();
  if (application->header.struct_size >=
          offsetof(tuffite_application, resolve_utility_route) &&
      tuffite_abi_is_compatible(application->header.version, context_version)) {
    snapshot.utility_start_with_context =
        application->utility_start_with_context;
  }
  if (application->header.struct_size >=
      offsetof(tuffite_application, setup_browser)) {
    snapshot.resolve_utility_route = application->resolve_utility_route;
  }
  if (application->header.struct_size >=
      offsetof(tuffite_application, initialize_browser)) {
    snapshot.setup_browser = application->setup_browser;
  }
  if (application->header.struct_size >=
      offsetof(tuffite_application, cancel_v8_session)) {
    snapshot.initialize_browser = application->initialize_browser;
  }
  if (application->header.struct_size >=
      offsetof(tuffite_application, start_profile)) {
    snapshot.cancel_v8_session = application->cancel_v8_session;
  }
  if (application->header.struct_size >=
      offsetof(tuffite_application, command_scope)) {
    snapshot.start_profile = application->start_profile;
  }
  if (application->header.struct_size >= sizeof(tuffite_application)) {
    snapshot.command_scope = application->command_scope;
    snapshot.configuration = application->configuration;
  }
  return base::WrapUnique(new Application(snapshot));
}

Application::Application(const tuffite_application& application)
    : application_(application) {
  DETACH_FROM_SEQUENCE(browser_sequence_checker_);
}

Application::~Application() {
  Stop();
}

bool Application::Start(const tuffite_host& host, ApplicationContext& context) {
  CHECK_EQ(runtime_.load(), 0u);
  CHECK(!stopped_);
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  {
    base::AutoLock lock(event_lock_);
    accepting_events_ = true;
  }
  const auto& configuration =
      context.GetService<ManifestService>()->SerializeRuntimeConfiguration();
  tuffite_handle_t runtime = 0;
  const auto status = application_.start(
      &host, reinterpret_cast<const uint8_t*>(configuration.data()),
      configuration.size(), &runtime);
  if (status != TUFFITE_STATUS_OK || runtime == 0) {
    LOG(ERROR) << "Tuffite Runtime start failed: " << StatusName(status) << " ("
               << status << ")";
    return false;
  }
  {
    base::AutoLock lock(event_lock_);
    const auto published = runtime_.load();
    if (published != 0 && published != runtime) {
      LOG(ERROR) << "Tuffite Runtime published a different startup handle";
      return false;
    }
    runtime_.store(runtime);
  }
  browser_initialized_ = !application_.initialize_browser;
  ConfigureUtilityHostRuntime(*context.GetService<UtilityService>(),
                              *context.GetService<ManifestService>(),
                              application_, runtime);
  return true;
}

bool Application::StartProfile(Application& parent,
                               const tuffite_host& host,
                               ApplicationContext& context,
                               const std::string& profile_id,
                               UtilityService& utilities) {
  if (!application_.start_profile || !parent.runtime()) {
    return false;
  }
  parent_ = &parent;
  {
    base::AutoLock lock(event_lock_);
    accepting_events_ = true;
  }
  const auto& configuration =
      context.GetService<ManifestService>()->SerializeRuntimeConfiguration();
  tuffite_handle_t runtime = 0;
  const auto status = application_.start_profile(
      parent.runtime(), &host,
      {reinterpret_cast<const uint8_t*>(configuration.data()),
       configuration.size()},
      {reinterpret_cast<const uint8_t*>(profile_id.data()), profile_id.size()},
      &runtime);
  if (status != TUFFITE_STATUS_OK || !runtime) {
    return false;
  }
  runtime_.store(runtime);
  browser_initialized_ = !application_.initialize_browser;
  ConfigureUtilityHostRuntime(utilities, *context.GetService<ManifestService>(),
                              application_, runtime);
  return true;
}

void Application::InitializeBrowser(base::OnceCallback<void(bool)> callback) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  if (!accepting_requests_ || !runtime_.load() || !browser_task_runner_) {
    std::move(callback).Run(false);
    return;
  }
  if (!application_.initialize_browser) {
    std::move(callback).Run(true);
    return;
  }
  struct Pending {
    scoped_refptr<base::SequencedTaskRunner> runner;
    base::OnceCallback<void(bool)> completion;
  };
  auto* pending =
      new Pending{browser_task_runner_,
                  base::BindOnce(
                      [](base::WeakPtr<Application> application,
                         base::OnceCallback<void(bool)> callback, bool ok) {
                        if (!application || !application->accepting_requests_) {
                          return;
                        }
                        application->browser_initialized_ = ok;
                        application->initialization_failed_ = !ok;
                        std::move(callback).Run(ok);
                      },
                      weak_this_, std::move(callback))};
  const auto status = application_.initialize_browser(
      runtime_.load(), pending,
      [](void* data, tuffite_status_t status, tuffite_byte_slice_t) {
        std::unique_ptr<Pending> pending(static_cast<Pending*>(data));
        pending->runner->PostTask(FROM_HERE,
                                  base::BindOnce(std::move(pending->completion),
                                                 status == TUFFITE_STATUS_OK));
        return TUFFITE_STATUS_OK;
      });
  if (status != TUFFITE_STATUS_OK) {
    std::unique_ptr<Pending> rejected(pending);
    rejected->runner->PostTask(
        FROM_HERE, base::BindOnce(std::move(rejected->completion), false));
  }
}

void Application::BindBrowserTaskRunner() {
  CHECK(base::SequencedTaskRunner::HasCurrentDefault());
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  const auto weak = weak_factory_.GetWeakPtr();
  base::AutoLock lock(event_lock_);
  browser_task_runner_ = base::SequencedTaskRunner::GetCurrentDefault();
  weak_this_ = weak;
}

bool Application::Dispatch(std::vector<uint8_t> event,
                           DispatchCallback callback,
                           const base::UnguessableToken& document) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  const tuffite_handle_t runtime = runtime_.load();
  const auto decoded_request = rpc::DecodeRequest(event);
  // Forwarding an application command must still respect the originating
  // profile's lifecycle admission, not only the parent's admission.
  if (!runtime || !browser_task_runner_ || !decoded_request ||
      !accepting_requests_ || !browser_initialized_) {
    std::move(callback).Run(false, {});
    return false;
  }
  if (decoded_request && application_.command_scope) {
    const auto scope = application_.command_scope(decoded_request->command_id);
    if (parent_ && scope == 1) {
      return parent_->Dispatch(std::move(event), std::move(callback), document);
    }
    if (scope != (parent_ ? 2u : 1u)) {
      std::move(callback).Run(false, {});
      return false;
    }
  }
  // Runtime correlation uses an id owned by the host. The codec validates
  // the complete frame before allowing the in-place rewrite.
  const uint64_t caller_id = decoded_request->request_id;
  uint64_t request_id;
  // Bound browser-side state even when the host finishes faster than the UI can
  // consume replies. Zero is reserved, and IDs must never wrap and collide.
  if (!accepting_requests_ || !browser_initialized_ || pending_.size() >= 256 ||
      next_request_id_ == 0) {
    request_id = 0;
  } else {
    request_id = next_request_id_++;
    pending_.emplace(request_id,
                     PendingRequest{std::move(callback), caller_id, document,
                                    std::make_unique<base::OneShotTimer>()});
  }
  if (request_id == 0) {
    std::move(callback).Run(false, {});
    return false;
  }
  pending_.at(request_id)
      .timeout->Start(
          FROM_HERE, base::Seconds(30),
          base::BindOnce(
              &Application::CompletePending, weak_this_, request_id, false,
              ErrorBytes("ShellAPI request timed out after 30 seconds")));
  CHECK(rpc::ReplaceRequestId(&event, request_id));
  const tuffite_status_t status =
      application_.dispatch(runtime, event.data(), event.size());
  if (status != TUFFITE_STATUS_OK) {
    // Retrieve and fail the callback we just registered so the pipe does not
    // leak. If the response happens to race here, whichever site wins the
    // lookup runs the callback.
    DispatchCallback failed;
    {
      auto it = pending_.find(request_id);
      if (it != pending_.end()) {
        failed = std::move(it->second.callback);
        pending_.erase(it);
      }
    }
    if (failed) {
      std::move(failed).Run(
          false,
          ErrorBytes("Runtime rejected the ShellAPI request before execution"));
    }
    return false;
  }
  return true;
}

void Application::CancelDocument(const base::UnguessableToken& document) {
  if (parent_) {
    parent_->CancelDocument(document);
  }
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  std::erase_if(pending_, [&](const auto& entry) {
    return entry.second.document == document;
  });
}

bool Application::ReadResponseRequestId(base::span<const uint8_t> bytes,
                                        uint64_t* request_id) {
  const auto response = rpc::DecodeResponse(bytes);
  if (!response) {
    return false;
  }
  *request_id = response->request_id;
  return true;
}

void Application::CompletePending(uint64_t request_id,
                                  bool ok,
                                  std::vector<uint8_t> response) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  DispatchCallback callback;
  auto it = pending_.find(request_id);
  if (it == pending_.end()) {
    return;
  }
  callback = std::move(it->second.callback);
  if (ok && !rpc::ReplaceResponseRequestId(&response, it->second.caller_id)) {
    ok = false;
    response.clear();
  }
  pending_.erase(it);
  std::move(callback).Run(ok, std::move(response));
}

void Application::FailPending() {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  std::unordered_map<uint64_t, PendingRequest> to_fail;
  to_fail.swap(pending_);
  for (auto& entry : to_fail) {
    std::move(entry.second.callback).Run(false, {});
  }
}

void Application::Quiesce() {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  accepting_requests_ = false;
  FailPending();
}

void Application::StopAsync(base::OnceClosure complete) {
  if (stopped_) {
    std::move(complete).Run();
    return;
  }
  stopped_ = true;
  Quiesce();
  weak_factory_.InvalidateWeakPtrs();
  tuffite_handle_t runtime;
  {
    base::AutoLock lock(event_lock_);
    accepting_events_ = false;
    runtime = runtime_.exchange(0);
  }
  stop_complete_.Reset();
  const auto stop = application_.stop;
  base::ThreadPool::PostTaskAndReply(
      FROM_HERE, {base::MayBlock(), base::TaskPriority::USER_BLOCKING},
      base::BindOnce(
          [](tuffite_application_stop_fn stop, tuffite_handle_t runtime,
             base::WaitableEvent* complete) {
            if (runtime) {
              stop(runtime);
            }
            complete->Signal();
          },
          stop, runtime, &stop_complete_),
      std::move(complete));
}

void Application::Stop() {
  if (stopped_) {
    stop_complete_.Wait();
    return;
  }
  stopped_ = true;
  Quiesce();
  weak_factory_.InvalidateWeakPtrs();
  tuffite_handle_t runtime;
  {
    base::AutoLock lock(event_lock_);
    accepting_events_ = false;
    runtime = runtime_.exchange(0);
  }
  if (runtime) {
    const auto status = application_.stop(runtime);
    if (status != TUFFITE_STATUS_OK) {
      LOG(ERROR) << "Tuffite Runtime stop failed: " << status;
    }
  }
}

void Application::PublishApplicationEvent(std::string name,
                                          std::vector<uint8_t> payload) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  if (accepting_requests_ && application_event_) {
    application_event_.Run(std::move(name), std::move(payload));
  }
}

tuffite_status_t Application::OnEvent(tuffite_handle_t runtime,
                                      tuffite_runtime_event_kind_t kind,
                                      tuffite_byte_slice_t payload) {
  scoped_refptr<base::SequencedTaskRunner> runner;
  base::WeakPtr<Application> weak;
  {
    base::AutoLock lock(event_lock_);
    if (!accepting_events_) {
      return TUFFITE_STATUS_INVALID_STATE;
    }
    if (runtime == 0) {
      return TUFFITE_STATUS_INVALID_HANDLE;
    }
    if (kind == TUFFITE_RUNTIME_EVENT_STARTED) {
      tuffite_handle_t expected = 0;
      if (!runtime_.compare_exchange_strong(expected, runtime) &&
          expected != runtime) {
        return TUFFITE_STATUS_INVALID_HANDLE;
      }
      return TUFFITE_STATUS_OK;
    }
    if (runtime_.load() != runtime) {
      return TUFFITE_STATUS_INVALID_HANDLE;
    }
    runner = browser_task_runner_;
    weak = weak_this_;
  }
  VLOG(1) << "Tuffite Runtime event kind=" << kind
          << " payload_bytes=" << payload.len;
  if (kind == TUFFITE_RUNTIME_EVENT_STOPPING) {
    // Quiesce() fails pending callbacks on the browser sequence before calling
    // Runtime's synchronous stop function. The event remains an ordering
    // marker for non-browser frameworks.
    return TUFFITE_STATUS_OK;
  } else if (kind == TUFFITE_RUNTIME_EVENT_COMMAND_RESPONSE) {
    // Parse the request_id from the response header and dispatch the
    // matching pending callback on the browser task runner. Do not run the
    // callback on this thread because host worker threads are not the
    // browser sequence.
    uint64_t request_id = 0;
    // SAFETY: The native ABI producer supplies the declared readable byte range
    // for this callback. Copy it before returning; never retain the raw
    // pointer.
    if (!payload.data || !ReadResponseRequestId(UNSAFE_BUFFERS(base::span(
                                                    payload.data, payload.len)),
                                                &request_id)) {
      return TUFFITE_STATUS_INVALID_ARGUMENT;
    }
    if (!runner) {
      return TUFFITE_STATUS_INVALID_STATE;
    }
    // SAFETY: The native ABI producer supplies the declared readable byte range
    // for this callback. Copy it before returning; never retain the raw
    // pointer.
    const base::span<const uint8_t> bytes =
        UNSAFE_BUFFERS(base::span<const uint8_t>(payload.data, payload.len));
    std::vector<uint8_t> body(bytes.begin(), bytes.end());
    if (!runner->PostTask(FROM_HERE,
                          base::BindOnce(&Application::CompletePending, weak,
                                         request_id, true, std::move(body)))) {
      return TUFFITE_STATUS_INVALID_STATE;
    }
  } else if (kind == TUFFITE_RUNTIME_EVENT_APPLICATION_EVENT) {
    if (!payload.data || payload.len < sizeof(uint16_t)) {
      return TUFFITE_STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: The native ABI producer supplies the declared readable byte range
    // for this callback. Copy it before returning; never retain the raw
    // pointer.
    const base::span<const uint8_t> bytes =
        UNSAFE_BUFFERS(base::span<const uint8_t>(payload.data, payload.len));
    const uint16_t name_length =
        bytes[0] | (static_cast<uint16_t>(bytes[1]) << 8);
    if (name_length == 0 || name_length > kMaxApplicationEventNameBytes ||
        name_length > bytes.size() - sizeof(uint16_t) ||
        bytes.size() - sizeof(uint16_t) - name_length >
            kMaxApplicationEventPayloadBytes) {
      return TUFFITE_STATUS_INVALID_ARGUMENT;
    }
    std::string name(bytes.begin() + sizeof(uint16_t),
                     bytes.begin() + sizeof(uint16_t) + name_length);
    std::vector<uint8_t> event_payload(
        bytes.begin() + sizeof(uint16_t) + name_length, bytes.end());
    if (!runner) {
      return TUFFITE_STATUS_INVALID_STATE;
    }
    if (!runner->PostTask(
            FROM_HERE,
            base::BindOnce(&Application::PublishApplicationEvent, weak,
                           std::move(name), std::move(event_payload)))) {
      return TUFFITE_STATUS_INVALID_STATE;
    }
  }
  return TUFFITE_STATUS_OK;
}

}  // namespace tuffite::abi
