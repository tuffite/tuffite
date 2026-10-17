// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/abi_host_bridge.h"

#include <array>
#include <string>
#include <utility>

#include "base/check.h"
#include "base/check_op.h"
#include "base/containers/span.h"
#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/logging.h"
#include "base/memory/ptr_util.h"
#include "base/task/sequenced_task_runner.h"
#include "base/task/thread_pool.h"
#include "base/time/time.h"
#include "tuffite/components/dispatch/common/codec.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/utility/host_runtime.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service_factory.h"

namespace tuffite {
namespace {

static_assert(sizeof(tuffite_handle_t) == sizeof(uint64_t));
static_assert(offsetof(tuffite_host, header) == 0);
static_assert(offsetof(tuffite_utility, header) == 0);
static_assert(offsetof(tuffite_shell_api, header) == 0);
static_assert(offsetof(tuffite_browser_host, header) == 0);
static_assert(offsetof(tuffite_utility_session_client, header) == 0);
static_assert(offsetof(tuffite_application, header) == 0);
static_assert(offsetof(tuffite_application, utility) >
              offsetof(tuffite_application, stop));

constexpr size_t kMaxApplicationEventNameBytes = 256;
constexpr size_t kMaxApplicationEventPayloadBytes = 1024 * 1024;
constexpr size_t kMaxUtilityServiceNameBytes = 128;
constexpr size_t kMaxUtilitySessionMessageBytes = 1024 * 1024;
constexpr size_t kMaxShellApiMessageBytes = 16 * 1024 * 1024;

bool CopyFfiBytes(const uint8_t* data,
                  size_t length,
                  size_t limit,
                  std::vector<uint8_t>* output) {
  if (!output || length > limit || (!data && length != 0)) {
    return false;
  }
  if (length == 0) {
    output->clear();
    return true;
  }
  // SAFETY: The native ABI producer supplies the declared readable byte range
  // for this callback. Copy it before returning; never retain the raw pointer.
  const base::span<const uint8_t> bytes =
      UNSAFE_BUFFERS(base::span<const uint8_t>(data, length));
  output->assign(bytes.begin(), bytes.end());
  return true;
}

bool CopyFfiName(const uint8_t* data, size_t length, std::string* output) {
  std::vector<uint8_t> bytes;
  if (length == 0 ||
      !CopyFfiBytes(data, length, kMaxUtilityServiceNameBytes, &bytes)) {
    return false;
  }
  output->assign(bytes.begin(), bytes.end());
  return output->find('\0') == std::string::npos;
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

std::unique_ptr<AbiHostBridge> AbiHostBridge::Create(
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
  // ABI 2.0 providers expose only the original prefix. Never read their tail.
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
  };
  const tuffite_abi_version startup_version = {2, 1, 0};
  if (application->header.struct_size >=
          offsetof(tuffite_application, prepare_process) &&
      tuffite_abi_is_compatible(application->header.version, startup_version)) {
    snapshot.prepare_browser = application->prepare_browser;
  }
  const tuffite_abi_version process_version = {2, 3, 0};
  if (application->header.struct_size >=
          offsetof(tuffite_application, resolve_v8) &&
      tuffite_abi_is_compatible(application->header.version, process_version)) {
    snapshot.prepare_process = application->prepare_process;
    snapshot.utility_runtime = application->utility_runtime;
  }
  const tuffite_abi_version resolver_version = {2, 4, 0};
  if (application->header.struct_size >=
          offsetof(tuffite_application, utility_start_with_context) &&
      tuffite_abi_is_compatible(application->header.version,
                                resolver_version)) {
    snapshot.resolve_v8 = application->resolve_v8;
  }
  const tuffite_abi_version context_version = {2, 5, 0};
  if (application->header.struct_size >= sizeof(tuffite_application) &&
      tuffite_abi_is_compatible(application->header.version, context_version)) {
    snapshot.utility_start_with_context =
        application->utility_start_with_context;
  }
  return base::WrapUnique(new AbiHostBridge(snapshot));
}

AbiHostBridge::AbiHostBridge(const tuffite_application& application)
    : shell_api_router_(
          std::make_unique<rpc::Router>(shell_api::BuiltinRoutes())),
      application_(application) {
  utility_session_ = {
      .header =
          tuffite_abi_header_for_size(sizeof(tuffite_utility_session_client)),
      .user_data = this,
      .create = &AbiHostBridge::CreateUtilityHostSession,
      .send = &AbiHostBridge::SendUtilityHostSession,
      .acknowledge = &AbiHostBridge::AcknowledgeUtilityHostSession,
      .cancel = &AbiHostBridge::CancelUtilityHostSession,
      .close = &AbiHostBridge::CloseUtilityHostSession,
  };
  utility_ = {.header = tuffite_abi_header_for_size(sizeof(tuffite_utility)),
              .session = &utility_session_};
  shell_api_ = {
      .header = tuffite_abi_header_for_size(sizeof(tuffite_shell_api)),
      .user_data = this,
      .invoke = &AbiHostBridge::InvokeShellApi};
  browser_host_ = {
      .header = tuffite_abi_header_for_size(sizeof(tuffite_browser_host)),
      .shell_api = &shell_api_,
      .utility = &utility_,
  };
  DETACH_FROM_SEQUENCE(browser_sequence_checker_);
}

const tuffite_abi_header* AbiHostBridge::QueryExtension(void* user_data,
                                                        uint32_t capability) {
  auto* bridge = static_cast<AbiHostBridge*>(user_data);
  if (!bridge) {
    return nullptr;
  }
  switch (capability) {
    case TUFFITE_CAPABILITY_BROWSER:
      return &bridge->browser_host_.header;
    // Preserve the ABI 2.0 IDs for older consumers.
    case TUFFITE_CAPABILITY_SHELL_API:
      return &bridge->shell_api_.header;
    case TUFFITE_CAPABILITY_UTILITY:
      return &bridge->utility_.header;
    default:
      return nullptr;
  }
}

AbiHostBridge::~AbiHostBridge() {
  Stop();
}

bool AbiHostBridge::Start(std::string_view configuration) {
  if (!accepting_requests_) {
    return false;
  }
  CHECK_EQ(runtime_.load(), 0u);
  runtime_start_ = application_.start;
  runtime_dispatch_ = application_.dispatch;
  runtime_stop_ = application_.stop;
  const tuffite_host framework = {
      .header = tuffite_abi_header_for_size(sizeof(tuffite_host)),
      .user_data = this,
      .post_event = &AbiHostBridge::PostEvent,
      .post_task = &AbiHostBridge::PostTask,
      .query_extension = &AbiHostBridge::QueryExtension,
  };
  tuffite_handle_t runtime = 0;
  const tuffite_status_t status = runtime_start_(
      &framework, reinterpret_cast<const uint8_t*>(configuration.data()),
      configuration.size(), &runtime);
  if (status != TUFFITE_STATUS_OK || runtime == 0) {
    LOG(ERROR) << "Tuffite Runtime start failed: " << StatusName(status) << " ("
               << status << ")";
    Stop();
    return false;
  }
  runtime_.store(runtime);
  if (application_context_) {
    ConfigureUtilityHostRuntime(
        *application_context_->GetService<UtilityService>(),
        *application_context_->GetService<ManifestService>(), application_,
        runtime);
  }
  return true;
}

void AbiHostBridge::BindBrowserTaskRunner() {
  CHECK(base::SequencedTaskRunner::HasCurrentDefault());
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  browser_task_runner_ = base::SequencedTaskRunner::GetCurrentDefault();
  weak_this_ = weak_factory_.GetWeakPtr();
}

bool AbiHostBridge::Dispatch(std::vector<uint8_t> event,
                             DispatchCallback callback,
                             const base::UnguessableToken& document) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  const tuffite_handle_t runtime = runtime_.load();
  const auto decoded_request = rpc::DecodeRequest(event);
  if (!runtime || !runtime_dispatch_ || !browser_task_runner_ ||
      !decoded_request) {
    std::move(callback).Run(false, {});
    return false;
  }
  // Runtime correlation uses an id owned by the bridge. The codec validates
  // the complete frame before allowing the in-place rewrite.
  const uint64_t caller_id = decoded_request->request_id;
  uint64_t request_id;
  // Bound browser-side state even when the host finishes faster than the UI can
  // consume replies. Zero is reserved, and IDs must never wrap and collide.
  if (!accepting_requests_ || pending_.size() >= 256 || next_request_id_ == 0) {
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
              &AbiHostBridge::CompletePending, weak_this_, request_id, false,
              ErrorBytes("ShellAPI request timed out after 30 seconds")));
  CHECK(rpc::ReplaceRequestId(&event, request_id));
  const tuffite_status_t status =
      runtime_dispatch_(runtime, event.data(), event.size());
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

void AbiHostBridge::CancelDocument(const base::UnguessableToken& document) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  std::erase_if(pending_, [&](const auto& entry) {
    return entry.second.document == document;
  });
}

void AbiHostBridge::BeginShutdown() {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  std::unordered_map<uint64_t, base::OnceClosure> rejected;
  {
    base::AutoLock lock(host_calls_lock_);
    accepting_requests_ = false;
    rejected.swap(queued_host_calls_);
  }
  FailPending();
  auto pending_shell_api = std::move(pending_shell_api_);
  pending_shell_api_.clear();
  for (auto& [id, pending] : pending_shell_api) {
    pending.reply(pending.data, static_cast<int32_t>(rpc::Status::kUnavailable),
                  {.data = nullptr, .len = 0});
  }
  for (auto& [id, reply] : rejected) {
    if (reply) {
      std::move(reply).Run();
    }
  }
}

void AbiHostBridge::Stop() {
  if (browser_task_runner_) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  }
  BeginShutdown();
  weak_factory_.InvalidateWeakPtrs();
  const tuffite_handle_t runtime = runtime_.exchange(0);
  if (runtime && runtime_stop_) {
    const tuffite_status_t status = runtime_stop_(runtime);
    if (status != TUFFITE_STATUS_OK) {
      LOG(ERROR) << "Tuffite Runtime stop failed: " << status;
    }
  }
  runtime_start_ = nullptr;
  runtime_dispatch_ = nullptr;
  runtime_stop_ = nullptr;
}

// Admission and posting share a lock: quiescing cannot miss a call posted by
// a host worker. Rejected posts retain caller ownership; accepted queued calls
// receive their failure reply before host shutdown joins workers.
tuffite_status_t AbiHostBridge::PostHostCall(base::OnceClosure task,
                                             base::OnceClosure rejected) {
  base::AutoLock lock(host_calls_lock_);
  if (!accepting_requests_ || !browser_task_runner_ ||
      next_host_call_id_ == 0) {
    return TUFFITE_STATUS_INVALID_STATE;
  }
  const uint64_t id = next_host_call_id_++;
  queued_host_calls_.emplace(id, std::move(rejected));
  if (!browser_task_runner_->PostTask(
          FROM_HERE, base::BindOnce(&AbiHostBridge::RunHostCall, weak_this_, id,
                                    std::move(task)))) {
    queued_host_calls_.erase(id);
    return TUFFITE_STATUS_INVALID_STATE;
  }
  return TUFFITE_STATUS_OK;
}
void AbiHostBridge::RunHostCall(uint64_t id, base::OnceClosure task) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  {
    base::AutoLock lock(host_calls_lock_);
    if (!queued_host_calls_.erase(id)) {
      return;
    }
  }
  std::move(task).Run();
}

// static
bool AbiHostBridge::ReadResponseRequestId(base::span<const uint8_t> bytes,
                                          uint64_t* request_id) {
  const auto response = rpc::DecodeResponse(bytes);
  if (!response) {
    return false;
  }
  *request_id = response->request_id;
  return true;
}

void AbiHostBridge::CompletePending(uint64_t request_id,
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

void AbiHostBridge::FailPending() {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  std::unordered_map<uint64_t, PendingRequest> to_fail;
  to_fail.swap(pending_);
  for (auto& entry : to_fail) {
    std::move(entry.second.callback).Run(false, {});
  }
}

void AbiHostBridge::BroadcastApplicationEvent(std::string name,
                                              std::vector<uint8_t> payload) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  if (!application_context_) {
    return;
  }
  ShellApiServiceFactory::BroadcastEvent(*application_context_, std::move(name),
                                         std::move(payload));
}

// static
tuffite_status_t AbiHostBridge::PostEvent(void* user_data,
                                          tuffite_handle_t runtime,
                                          tuffite_runtime_event_kind_t kind,
                                          tuffite_byte_slice_t payload) {
  auto* bridge = static_cast<AbiHostBridge*>(user_data);
  if (!bridge || runtime == 0) {
    return TUFFITE_STATUS_INVALID_HANDLE;
  }
  if (kind == TUFFITE_RUNTIME_EVENT_STARTED) {
    tuffite_handle_t expected = 0;
    if (!bridge->runtime_.compare_exchange_strong(expected, runtime) &&
        expected != runtime) {
      return TUFFITE_STATUS_INVALID_HANDLE;
    }
    return TUFFITE_STATUS_OK;
  }
  const tuffite_handle_t active_runtime = bridge->runtime_.load();
  if (active_runtime == 0 || active_runtime != runtime) {
    return TUFFITE_STATUS_INVALID_HANDLE;
  }
  VLOG(1) << "Tuffite Runtime event kind=" << kind
          << " payload_bytes=" << payload.len;
  if (kind == TUFFITE_RUNTIME_EVENT_STOPPING) {
    // Stop() fails pending callbacks on the browser sequence before calling
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
    if (!bridge->browser_task_runner_) {
      return TUFFITE_STATUS_INVALID_STATE;
    }
    // SAFETY: The native ABI producer supplies the declared readable byte range
    // for this callback. Copy it before returning; never retain the raw
    // pointer.
    const base::span<const uint8_t> bytes =
        UNSAFE_BUFFERS(base::span<const uint8_t>(payload.data, payload.len));
    std::vector<uint8_t> body(bytes.begin(), bytes.end());
    bridge->browser_task_runner_->PostTask(
        FROM_HERE,
        base::BindOnce(&AbiHostBridge::CompletePending, bridge->weak_this_,
                       request_id, true, std::move(body)));
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
    if (!bridge->browser_task_runner_) {
      return TUFFITE_STATUS_INVALID_STATE;
    }
    bridge->browser_task_runner_->PostTask(
        FROM_HERE, base::BindOnce(&AbiHostBridge::BroadcastApplicationEvent,
                                  bridge->weak_this_, std::move(name),
                                  std::move(event_payload)));
  }
  return TUFFITE_STATUS_OK;
}

// static
tuffite_status_t AbiHostBridge::PostTask(void* user_data,
                                         tuffite_task_priority_t priority,
                                         uint8_t may_block,
                                         void* task_data,
                                         tuffite_task_run_fn run,
                                         tuffite_task_destroy_fn destroy) {
  if (!user_data || !task_data || !run || !destroy || may_block > 1) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  if (!static_cast<AbiHostBridge*>(user_data)->accepting_requests_) {
    return TUFFITE_STATUS_INVALID_STATE;
  }
  base::TaskPriority task_priority;
  switch (priority) {
    case TUFFITE_TASK_PRIORITY_BEST_EFFORT:
      task_priority = base::TaskPriority::BEST_EFFORT;
      break;
    case TUFFITE_TASK_PRIORITY_USER_VISIBLE:
      task_priority = base::TaskPriority::USER_VISIBLE;
      break;
    case TUFFITE_TASK_PRIORITY_USER_BLOCKING:
      task_priority = base::TaskPriority::USER_BLOCKING;
      break;
    default:
      return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  const base::TaskTraits traits =
      may_block ? base::TaskTraits(task_priority, base::MayBlock(),
                                   base::TaskShutdownBehavior::BLOCK_SHUTDOWN)
                : base::TaskTraits(task_priority,
                                   base::TaskShutdownBehavior::BLOCK_SHUTDOWN);
  const bool posted = base::ThreadPool::PostTask(
      FROM_HERE, traits,
      base::BindOnce(
          [](void* task_data, tuffite_task_run_fn run,
             tuffite_task_destroy_fn destroy) {
            run(task_data);
            destroy(task_data);
          },
          task_data, run, destroy));
  if (!posted) {
    return TUFFITE_STATUS_INVALID_STATE;
  }
  return TUFFITE_STATUS_OK;
}

// static
tuffite_status_t AbiHostBridge::CreateUtilityHostSession(
    void* user_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    const uint8_t* init,
    size_t init_len,
    uint32_t initial_event_credits,
    size_t max_event_bytes,
    void* session_data,
    tuffite_utility_host_event_fn event,
    tuffite_utility_host_closed_fn closed,
    void* create_data,
    tuffite_utility_host_create_reply_fn created) {
  auto* bridge = static_cast<AbiHostBridge*>(user_data);
  std::string service_name;
  std::vector<uint8_t> initialization;
  if (!bridge || !bridge->browser_task_runner_ || !session_data || !event ||
      !closed || !create_data || !created || session_id == 0 ||
      max_event_bytes > kMaxUtilitySessionMessageBytes ||
      !CopyFfiName(name, name_len, &service_name) ||
      !CopyFfiBytes(init, init_len, kMaxUtilitySessionMessageBytes,
                    &initialization)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return bridge->PostHostCall(
      base::BindOnce(
          [](AbiHostBridge* bridge, std::string service_name,
             uint64_t session_id, std::vector<uint8_t> initialization,
             uint32_t initial_event_credits, uint32_t max_event_bytes,
             void* session_data, tuffite_utility_host_event_fn event,
             tuffite_utility_host_closed_fn closed, void* create_data,
             tuffite_utility_host_create_reply_fn created) {
            bridge->application_context()
                ->GetService<UtilityService>()
                ->CreateUtilitySession(
                    std::move(service_name), session_id,
                    std::move(initialization), initial_event_credits,
                    max_event_bytes,
                    base::BindRepeating(
                        [](void* session_data,
                           tuffite_utility_host_event_fn event,
                           uint64_t sequence, uint32_t kind,
                           std::vector<uint8_t> payload) {
                          return event(session_data, sequence, kind,
                                       {.data = payload.data(),
                                        .len = payload.size()}) ==
                                 TUFFITE_STATUS_OK;
                        },
                        session_data, event),
                    base::BindOnce(
                        [](void* session_data,
                           tuffite_utility_host_closed_fn closed,
                           int32_t status, uint64_t last_sequence,
                           std::vector<uint8_t> summary) {
                          closed(
                              session_data, status, last_sequence,
                              {.data = summary.data(), .len = summary.size()});
                        },
                        session_data, closed),
                    base::BindOnce(
                        [](void* create_data,
                           tuffite_utility_host_create_reply_fn created,
                           int32_t status) { created(create_data, status); },
                        create_data, created));
          },
          bridge, std::move(service_name), session_id,
          std::move(initialization), initial_event_credits,
          static_cast<uint32_t>(max_event_bytes), session_data, event, closed,
          create_data, created),
      base::BindOnce(
          [](void* data, tuffite_utility_host_create_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE);
          },
          create_data, created));
}

// static
tuffite_status_t AbiHostBridge::SendUtilityHostSession(
    void* user_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    uint64_t command_id,
    const uint8_t* command,
    size_t command_len,
    void* reply_data,
    tuffite_utility_reply_fn reply) {
  auto* bridge = static_cast<AbiHostBridge*>(user_data);
  std::string service_name;
  std::vector<uint8_t> command_bytes;
  if (!bridge || !bridge->browser_task_runner_ || !reply_data || !reply ||
      session_id == 0 || command_id == 0 ||
      !CopyFfiName(name, name_len, &service_name) ||
      !CopyFfiBytes(command, command_len, kMaxUtilitySessionMessageBytes,
                    &command_bytes)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return bridge->PostHostCall(
      base::BindOnce(
          [](AbiHostBridge* bridge, std::string service_name,
             uint64_t session_id, uint64_t command_id,
             std::vector<uint8_t> command, void* reply_data,
             tuffite_utility_reply_fn reply) {
            bridge->application_context()
                ->GetService<UtilityService>()
                ->SendUtilitySession(
                    service_name, session_id, command_id, std::move(command),
                    base::BindOnce(
                        [](void* reply_data, tuffite_utility_reply_fn reply,
                           int32_t status, std::vector<uint8_t> response) {
                          reply(reply_data, status,
                                {.data = response.data(),
                                 .len = response.size()});
                        },
                        reply_data, reply));
          },
          bridge, std::move(service_name), session_id, command_id,
          std::move(command_bytes), reply_data, reply),
      base::BindOnce(
          [](void* data, tuffite_utility_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE,
                  {.data = nullptr, .len = 0});
          },
          reply_data, reply));
}

// static
tuffite_status_t AbiHostBridge::AcknowledgeUtilityHostSession(
    void* user_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    uint64_t committed_sequence,
    uint32_t credits) {
  auto* bridge = static_cast<AbiHostBridge*>(user_data);
  std::string service_name;
  if (!bridge || !bridge->browser_task_runner_ || session_id == 0 ||
      !CopyFfiName(name, name_len, &service_name)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return bridge->PostHostCall(
      base::BindOnce(
          [](AbiHostBridge* bridge, std::string name, uint64_t session_id,
             uint64_t committed_sequence, uint32_t credits) {
            bridge->application_context()
                ->GetService<UtilityService>()
                ->AcknowledgeUtilitySession(name, session_id,
                                            committed_sequence, credits);
          },
          bridge, std::move(service_name), session_id, committed_sequence,
          credits),
      base::OnceClosure());
}

// static
tuffite_status_t AbiHostBridge::CancelUtilityHostSession(void* user_data,
                                                         const uint8_t* name,
                                                         size_t name_len,
                                                         uint64_t session_id,
                                                         uint64_t command_id) {
  auto* bridge = static_cast<AbiHostBridge*>(user_data);
  std::string service_name;
  if (!bridge || !bridge->browser_task_runner_ || session_id == 0 ||
      command_id == 0 || !CopyFfiName(name, name_len, &service_name)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return bridge->PostHostCall(
      base::BindOnce(
          [](AbiHostBridge* bridge, std::string name, uint64_t session_id,
             uint64_t command_id) {
            bridge->application_context()
                ->GetService<UtilityService>()
                ->CancelUtilitySessionCommand(name, session_id, command_id);
          },
          bridge, std::move(service_name), session_id, command_id),
      base::OnceClosure());
}

// static
tuffite_status_t AbiHostBridge::CloseUtilityHostSession(
    void* user_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    uint32_t reason,
    void* reply_data,
    tuffite_utility_reply_fn reply) {
  auto* bridge = static_cast<AbiHostBridge*>(user_data);
  std::string service_name;
  if (!bridge || !bridge->browser_task_runner_ || !reply_data || !reply ||
      session_id == 0 || !CopyFfiName(name, name_len, &service_name)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return bridge->PostHostCall(
      base::BindOnce(
          [](AbiHostBridge* bridge, std::string service_name,
             uint64_t session_id, uint32_t reason, void* reply_data,
             tuffite_utility_reply_fn reply) {
            bridge->application_context()
                ->GetService<UtilityService>()
                ->CloseUtilitySession(
                    service_name, session_id, reason,
                    base::BindOnce(
                        [](void* reply_data, tuffite_utility_reply_fn reply,
                           int32_t status, std::vector<uint8_t> response) {
                          reply(reply_data, status,
                                {.data = response.data(),
                                 .len = response.size()});
                        },
                        reply_data, reply));
          },
          bridge, std::move(service_name), session_id, reason, reply_data,
          reply),
      base::BindOnce(
          [](void* data, tuffite_utility_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE,
                  {.data = nullptr, .len = 0});
          },
          reply_data, reply));
}

void AbiHostBridge::CompleteShellApi(uint64_t id,
                                     int32_t status,
                                     const std::vector<uint8_t>& response) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  auto found = pending_shell_api_.find(id);
  if (found == pending_shell_api_.end()) {
    return;
  }
  const auto pending = found->second;
  pending_shell_api_.erase(found);
  pending.reply(pending.data, status,
                {.data = response.data(), .len = response.size()});
}

// static
tuffite_status_t AbiHostBridge::InvokeShellApi(
    void* user_data,
    const uint8_t* channel,
    size_t channel_len,
    const uint8_t* request,
    size_t request_len,
    void* reply_data,
    tuffite_shell_api_reply_fn reply) {
  auto* bridge = static_cast<AbiHostBridge*>(user_data);
  std::string path;
  std::vector<uint8_t> request_bytes;
  if (!bridge || !bridge->browser_task_runner_ || !reply_data || !reply ||
      !CopyFfiName(channel, channel_len, &path) ||
      !CopyFfiBytes(request, request_len, kMaxShellApiMessageBytes,
                    &request_bytes)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return bridge->PostHostCall(
      base::BindOnce(
          [](AbiHostBridge* bridge, std::string path,
             std::vector<uint8_t> request, void* data,
             tuffite_shell_api_reply_fn reply) {
            const uint64_t id = bridge->next_shell_api_id_++;
            bridge->pending_shell_api_.emplace(id,
                                               PendingShellApi{data, reply});
            rpc::Reply complete = base::BindOnce(
                &AbiHostBridge::CompleteShellApi, bridge->weak_this_, id);
            if (!bridge->application_context()) {
              std::move(complete).Run(
                  static_cast<int32_t>(rpc::Status::kUnavailable), {});
              return;
            }
            dispatch::Context context(*bridge->application_context(),
                                      bridge->native_owner_);
            if (!bridge->shell_api_router_->Dispatch(context, path, request,
                                                     &complete)) {
              std::move(complete).Run(
                  static_cast<int32_t>(rpc::Status::kInvalidArgument), {});
            }
          },
          bridge, std::move(path), std::move(request_bytes), reply_data, reply),
      base::BindOnce(
          [](void* data, tuffite_shell_api_reply_fn reply) {
            reply(data, static_cast<int32_t>(rpc::Status::kUnavailable),
                  {.data = nullptr, .len = 0});
          },
          reply_data, reply));
}

}  // namespace tuffite
