// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/abi/host.h"

#include <algorithm>
#include <functional>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <utility>

#include "base/check.h"
#include "base/compiler_specific.h"
#include "base/containers/span.h"
#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/logging.h"
#include "base/strings/string_number_conversions.h"
#include "base/task/sequenced_task_runner.h"
#include "base/task/thread_pool.h"
#include "tuffite/components/dispatch/browser/rpc.h"
#include "tuffite/components/dispatch/common/status.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/profile/profile_manager.h"
#include "tuffite/framework/app/services/profile/profile_service_factory.h"
#include "tuffite/framework/app/services/remoting/remoting_protocol.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/isolated/isolated_service_factory.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service.h"
#include "tuffite/framework/browser/services/shell_api/shell_api_service_factory.h"

namespace tuffite::abi {
namespace {
static_assert(offsetof(tuffite_host, header) == 0);
static_assert(offsetof(tuffite_utility_session_client, header) == 0);
static_assert(offsetof(tuffite_isolated, header) == 0);
constexpr size_t kMaxUtilitySessionMessageBytes = 1024 * 1024;
constexpr size_t kMaxRemotingMessageBytes = 16 * 1024 * 1024;
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
  if (length == 0 || !CopyFfiBytes(data, length, 128, &bytes)) {
    return false;
  }
  output->assign(bytes.begin(), bytes.end());
  return output->find('\0') == std::string::npos;
}

}  // namespace

Host::Host(EventHandler events) : events_(std::move(events)) {
  CHECK(events_);
  InitializeAbi();
  DETACH_FROM_SEQUENCE(browser_sequence_checker_);
}

Host::~Host() {
  Shutdown();
}

void Host::BindBrowserTaskRunner() {
  CHECK(base::SequencedTaskRunner::HasCurrentDefault());
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  const auto weak = weak_factory_.GetWeakPtr();
  base::AutoLock lock(host_calls_lock_);
  browser_task_runner_ = base::SequencedTaskRunner::GetCurrentDefault();
  weak_this_ = weak;
}

void Host::BindApplicationContext(ApplicationContext* context) {
  application_context_ = context ? context->GetWeakPtr() : nullptr;
}

Host::ApplicationEvent Host::application_event() {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  return base::BindRepeating(&Host::BroadcastApplicationEvent, weak_this_);
}

void Host::Close() {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  std::unordered_map<uint64_t, base::OnceClosure> rejected;
  {
    base::AutoLock lock(host_calls_lock_);
    accepting_requests_ = false;
    rejected.swap(queued_host_calls_);
  }
  auto pending = std::move(pending_remoting_);
  pending_remoting_.clear();
  for (const auto& [id, call] : pending) {
    call.reply(call.data, static_cast<int32_t>(rpc::Status::kUnavailable), {});
  }
  for (auto& [id, reply] : rejected) {
    if (reply) {
      std::move(reply).Run();
    }
  }
}

void Host::Shutdown() {
  if (!accepting_tasks_) {
    return;
  }
  Close();
  {
    base::AutoLock lock(host_calls_lock_);
    accepting_tasks_ = false;
  }
  application_context_.reset();
  browser_context_.reset();
  weak_factory_.InvalidateWeakPtrs();
}

tuffite_status_t Host::PostEvent(void* data,
                                 tuffite_handle_t runtime,
                                 tuffite_runtime_event_kind_t kind,
                                 tuffite_byte_slice_t payload) {
  auto* host = static_cast<Host*>(data);
  if (!host) {
    return TUFFITE_STATUS_INVALID_HANDLE;
  }
  // A retired Host keeps its ABI tables, but its runtime callback target may
  // already be destroyed. Shutdown joins admissions under this same lock.
  base::AutoLock lock(host->host_calls_lock_);
  if (!host->accepting_tasks_) {
    return TUFFITE_STATUS_INVALID_STATE;
  }
  return host->events_.Run(runtime, kind, payload);
}

UtilityService* Host::utility_service() const {
  return browser_context_ ? &browser_context_->utility_service()
                          : application_context_->GetService<UtilityService>();
}

template <typename Method, auto Handler>
tuffite_status_t Host::CallRemoting(void* data,
                                    tuffite_byte_slice_t request,
                                    void* reply_data,
                                    tuffite_remoting_reply_fn reply) {
  auto* host = static_cast<Host*>(data);
  std::vector<uint8_t> bytes;
  if (!host || !reply_data || !reply ||
      !CopyFfiBytes(request.data, request.len, kMaxRemotingMessageBytes,
                    &bytes)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return host->PostHostCall(
      base::BindOnce(
          [](Host* host, std::vector<uint8_t> bytes, void* data,
             tuffite_remoting_reply_fn reply) {
            if (!host->application_context_ || host->next_remoting_id_ == 0) {
              reply(data, static_cast<int32_t>(rpc::Status::kUnavailable), {});
              return;
            }
            const uint64_t id = host->next_remoting_id_++;
            host->pending_remoting_.emplace(id, PendingRemoting{data, reply});
            Method::Call(
                bytes,
                base::BindOnce(&Host::CompleteRemoting, host->weak_this_, id),
                base::BindOnce(
                    Handler,
                    std::ref(*host->application_context_
                                  ->GetService<remoting::RemotingService>())));
          },
          host, std::move(bytes), reply_data, reply),
      base::BindOnce(
          [](void* data, tuffite_remoting_reply_fn reply) {
            reply(data, static_cast<int32_t>(rpc::Status::kUnavailable), {});
          },
          reply_data, reply));
}
void Host::CompleteRemoting(uint64_t id,
                            int32_t status,
                            const std::vector<uint8_t>& bytes) {
  auto found = pending_remoting_.find(id);
  if (found == pending_remoting_.end()) {
    return;
  }
  const auto call = found->second;
  pending_remoting_.erase(found);
  call.reply(call.data, status, {.data = bytes.data(), .len = bytes.size()});
}

void Host::InitializeAbi() {
  remoting_ = {
      .header = tuffite_abi_header_for_size(sizeof(tuffite_remoting)),
      .user_data = this,
      .capabilities = &Host::CallRemoting<
          shell_api::generated::Remoting::CapabilitiesMethod,
          &remoting::Protocol::Capabilities>,
      .start = &Host::CallRemoting<shell_api::generated::Remoting::StartMethod,
                                   &remoting::Protocol::Start>,
      .process_signal = &Host::CallRemoting<
          shell_api::generated::Remoting::ProcessSignalMethod,
          &remoting::Protocol::ProcessSignal>,
      .take_signals =
          &Host::CallRemoting<shell_api::generated::Remoting::TakeSignalsMethod,
                              &remoting::Protocol::TakeSignals>,
      .send_input =
          &Host::CallRemoting<shell_api::generated::Remoting::SendInputMethod,
                              &remoting::Protocol::SendInput>,
      .inject_input =
          &Host::CallRemoting<shell_api::generated::Remoting::InjectInputMethod,
                              &remoting::Protocol::InjectInput>,
      .stop = &Host::CallRemoting<shell_api::generated::Remoting::StopMethod,
                                  &remoting::Protocol::Stop>,
      .stats = &Host::CallRemoting<shell_api::generated::Remoting::StatsMethod,
                                   &remoting::Protocol::Stats>,
      .capture =
          &Host::CallRemoting<shell_api::generated::Remoting::CaptureMethod,
                              &remoting::Protocol::Capture>,
  };
  contexts_ = {.header = tuffite_abi_header_for_size(sizeof(tuffite_contexts)),
               .user_data = this,
               .apply = &Host::ApplyContexts};
  host_ = {.header = tuffite_abi_header_for_size(sizeof(tuffite_host)),
           .user_data = this,
           .post_event = &Host::PostEvent,
           .post_task = &Host::PostTask,
           .query_extension = &Host::QueryExtension};
  utility_session_ = {
      .header =
          tuffite_abi_header_for_size(sizeof(tuffite_utility_session_client)),
      .user_data = this,
      .create = &Host::CreateUtilityHostSession,
      .send = &Host::SendUtilityHostSession,
      .acknowledge = &Host::AcknowledgeUtilityHostSession,
      .cancel = &Host::CancelUtilityHostSession,
      .close = &Host::CloseUtilityHostSession,
  };
  isolated_ = {.header = tuffite_abi_header_for_size(sizeof(tuffite_isolated)),
               .user_data = this,
               .apply = &Host::ApplyIsolated};
}

const tuffite_abi_header* Host::QueryExtension(void* user_data,
                                               uint32_t capability) {
  auto* host = static_cast<Host*>(user_data);
  if (!host) {
    return nullptr;
  }
  switch (capability) {
    case TUFFITE_CAPABILITY_REMOTING:
      return &host->remoting_.header;
    case TUFFITE_CAPABILITY_CONTEXTS:
      return &host->contexts_.header;
    case TUFFITE_CAPABILITY_ISOLATED:
      return &host->isolated_.header;
    case TUFFITE_CAPABILITY_UTILITY_SESSION:
      return &host->utility_session_.header;
    case TUFFITE_CAPABILITY_UTILITY_INTERFACES: {
      static const auto header =
          tuffite_abi_header_for_size(sizeof(tuffite_abi_header));
      return &header;
    }
    default:
      return nullptr;
  }
}

tuffite_status_t Host::PostTask(void* user_data,
                                tuffite_task_priority_t priority,
                                uint8_t may_block,
                                void* task_data,
                                tuffite_task_run_fn run,
                                tuffite_task_destroy_fn destroy) {
  if (!user_data || !task_data || !run || !destroy || may_block > 1) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  if (!static_cast<Host*>(user_data)->accepting_tasks_) {
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

// Admission and posting share a lock so quiescing cannot miss an accepted call.
// Rejected submissions retain caller ownership; accepted calls receive a reply.

tuffite_status_t Host::PostHostCall(base::OnceClosure task,
                                    base::OnceClosure rejected) {
  base::AutoLock lock(host_calls_lock_);
  if (!accepting_requests_ || !browser_task_runner_ ||
      next_host_call_id_ == 0) {
    return TUFFITE_STATUS_INVALID_STATE;
  }
  const uint64_t id = next_host_call_id_++;
  queued_host_calls_.emplace(id, std::move(rejected));
  if (!browser_task_runner_->PostTask(
          FROM_HERE, base::BindOnce(&Host::RunHostCall, weak_this_, id,
                                    std::move(task)))) {
    queued_host_calls_.erase(id);
    return TUFFITE_STATUS_INVALID_STATE;
  }
  return TUFFITE_STATUS_OK;
}

void Host::RunHostCall(uint64_t id, base::OnceClosure task) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  base::OnceClosure rejected;
  {
    base::AutoLock lock(host_calls_lock_);
    auto found = queued_host_calls_.find(id);
    if (found == queued_host_calls_.end()) {
      return;
    }
    rejected = std::move(found->second);
    queued_host_calls_.erase(found);
  }
  if (!application_context_) {
    if (rejected) {
      std::move(rejected).Run();
    }
    return;
  }
  std::move(task).Run();
}

void Host::BroadcastApplicationEvent(std::string name,
                                     std::vector<uint8_t> payload) {
  DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
  if (!application_context_) {
    return;
  }
  if (browser_context_) {
    if (auto* service = ShellApiServiceFactory::GetForBrowserContext(
            browser_context_.get())) {
      service->Broadcast(name, payload);
    }
    return;
  }
  ShellApiServiceFactory::BroadcastEvent(*application_context(),
                                         std::move(name), std::move(payload));
}

tuffite_status_t Host::CreateUtilityHostSession(
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
  auto* host = static_cast<Host*>(user_data);
  std::string service_name;
  std::vector<uint8_t> initialization;
  if (!host || !session_data || !event || !closed || !create_data || !created ||
      session_id == 0 || max_event_bytes > kMaxUtilitySessionMessageBytes ||
      !CopyFfiName(name, name_len, &service_name) ||
      !CopyFfiBytes(init, init_len, kMaxUtilitySessionMessageBytes,
                    &initialization)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return host->PostHostCall(
      base::BindOnce(
          [](Host* host, std::string service_name, uint64_t session_id,
             std::vector<uint8_t> initialization,
             uint32_t initial_event_credits, uint32_t max_event_bytes,
             void* session_data, tuffite_utility_host_event_fn event,
             tuffite_utility_host_closed_fn closed, void* create_data,
             tuffite_utility_host_create_reply_fn created) {
            host->utility_service()->CreateUtilitySession(
                std::move(service_name), session_id, std::move(initialization),
                initial_event_credits, max_event_bytes,
                base::BindRepeating(
                    [](void* session_data, tuffite_utility_host_event_fn event,
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
                       tuffite_utility_host_closed_fn closed, int32_t status,
                       uint64_t last_sequence, std::vector<uint8_t> summary) {
                      closed(session_data, status, last_sequence,
                             {.data = summary.data(), .len = summary.size()});
                    },
                    session_data, closed),
                base::BindOnce(
                    [](void* create_data,
                       tuffite_utility_host_create_reply_fn created,
                       int32_t status) { created(create_data, status); },
                    create_data, created));
          },
          host, std::move(service_name), session_id, std::move(initialization),
          initial_event_credits, static_cast<uint32_t>(max_event_bytes),
          session_data, event, closed, create_data, created),
      base::BindOnce(
          [](void* data, tuffite_utility_host_create_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE);
          },
          create_data, created));
}

tuffite_status_t Host::SendUtilityHostSession(void* user_data,
                                              const uint8_t* name,
                                              size_t name_len,
                                              uint64_t session_id,
                                              uint64_t command_id,
                                              const uint8_t* command,
                                              size_t command_len,
                                              void* reply_data,
                                              tuffite_utility_reply_fn reply) {
  auto* host = static_cast<Host*>(user_data);
  std::string service_name;
  std::vector<uint8_t> command_bytes;
  if (!host || !reply_data || !reply || session_id == 0 || command_id == 0 ||
      !CopyFfiName(name, name_len, &service_name) ||
      !CopyFfiBytes(command, command_len, kMaxUtilitySessionMessageBytes,
                    &command_bytes)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return host->PostHostCall(
      base::BindOnce(
          [](Host* host, std::string service_name, uint64_t session_id,
             uint64_t command_id, std::vector<uint8_t> command,
             void* reply_data, tuffite_utility_reply_fn reply) {
            host->utility_service()->SendUtilitySession(
                service_name, session_id, command_id, std::move(command),
                base::BindOnce(
                    [](void* reply_data, tuffite_utility_reply_fn reply,
                       int32_t status, std::vector<uint8_t> response) {
                      reply(reply_data, status,
                            {.data = response.data(), .len = response.size()});
                    },
                    reply_data, reply));
          },
          host, std::move(service_name), session_id, command_id,
          std::move(command_bytes), reply_data, reply),
      base::BindOnce(
          [](void* data, tuffite_utility_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE,
                  {.data = nullptr, .len = 0});
          },
          reply_data, reply));
}

tuffite_status_t Host::AcknowledgeUtilityHostSession(
    void* user_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    uint64_t committed_sequence,
    uint32_t credits) {
  auto* host = static_cast<Host*>(user_data);
  std::string service_name;
  if (!host || session_id == 0 || !CopyFfiName(name, name_len, &service_name)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return host->PostHostCall(
      base::BindOnce(
          [](Host* host, std::string name, uint64_t session_id,
             uint64_t committed_sequence, uint32_t credits) {
            host->utility_service()->AcknowledgeUtilitySession(
                name, session_id, committed_sequence, credits);
          },
          host, std::move(service_name), session_id, committed_sequence,
          credits),
      base::OnceClosure());
}

tuffite_status_t Host::CancelUtilityHostSession(void* user_data,
                                                const uint8_t* name,
                                                size_t name_len,
                                                uint64_t session_id,
                                                uint64_t command_id) {
  auto* host = static_cast<Host*>(user_data);
  std::string service_name;
  if (!host || session_id == 0 || command_id == 0 ||
      !CopyFfiName(name, name_len, &service_name)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return host->PostHostCall(
      base::BindOnce(
          [](Host* host, std::string name, uint64_t session_id,
             uint64_t command_id) {
            host->utility_service()->CancelUtilitySessionCommand(
                name, session_id, command_id);
          },
          host, std::move(service_name), session_id, command_id),
      base::OnceClosure());
}

tuffite_status_t Host::CloseUtilityHostSession(void* user_data,
                                               const uint8_t* name,
                                               size_t name_len,
                                               uint64_t session_id,
                                               uint32_t reason,
                                               void* reply_data,
                                               tuffite_utility_reply_fn reply) {
  auto* host = static_cast<Host*>(user_data);
  std::string service_name;
  if (!host || !reply_data || !reply || session_id == 0 ||
      !CopyFfiName(name, name_len, &service_name)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  return host->PostHostCall(
      base::BindOnce(
          [](Host* host, std::string service_name, uint64_t session_id,
             uint32_t reason, void* reply_data,
             tuffite_utility_reply_fn reply) {
            host->utility_service()->CloseUtilitySession(
                service_name, session_id, reason,
                base::BindOnce(
                    [](void* reply_data, tuffite_utility_reply_fn reply,
                       int32_t status, std::vector<uint8_t> response) {
                      reply(reply_data, status,
                            {.data = response.data(), .len = response.size()});
                    },
                    reply_data, reply));
          },
          host, std::move(service_name), session_id, reason, reply_data, reply),
      base::BindOnce(
          [](void* data, tuffite_utility_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE,
                  {.data = nullptr, .len = 0});
          },
          reply_data, reply));
}

tuffite_status_t Host::ApplyContexts(void* data,
                                     tuffite_byte_slice_t request,
                                     void* reply_data,
                                     tuffite_utility_reply_fn reply) {
  auto* host = static_cast<Host*>(data);
  if (!host || !reply || !reply_data || !request.data ||
      request.len > 1024 * 1024) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  std::string json(reinterpret_cast<const char*>(request.data), request.len);
  return host->PostHostCall(
      base::BindOnce(
          [](Host* host, std::string json, void* data,
             tuffite_utility_reply_fn reply) {
            auto* app = host->application_context();
            auto* manager = app ? app->GetService<ProfileManager>() : nullptr;
            auto request =
                base::JSONReader::ReadDict(json, base::JSON_PARSE_RFC);
            if (!manager || !request) {
              reply(data, TUFFITE_STATUS_INVALID_STATE, {});
              return;
            }
            manager->Apply(*request, host->browser_context().get(),
                           base::BindOnce(
                               [](void* data, tuffite_utility_reply_fn reply,
                                  int32_t status, std::vector<uint8_t> bytes) {
                                 return reply(data, status,
                                              {bytes.data(), bytes.size()}) ==
                                        TUFFITE_STATUS_OK;
                               },
                               data, reply));
          },
          host, std::move(json), reply_data, reply),
      base::BindOnce(
          [](void* data, tuffite_utility_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE, {});
          },
          reply_data, reply));
}

tuffite_status_t Host::ApplyIsolated(void* data,
                                     tuffite_byte_slice_t request,
                                     void* reply_data,
                                     tuffite_isolated_reply_fn reply) {
  auto* host = static_cast<Host*>(data);
  if (!host || !reply || !reply_data || !request.data || request.len == 0 ||
      request.len > 64 * 1024 * 1024) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  const std::string json(reinterpret_cast<const char*>(request.data),
                         request.len);
  return host->PostHostCall(
      base::BindOnce(
          [](Host* host, std::string json, void* data,
             tuffite_isolated_reply_fn reply) {
            const auto context = host->browser_context();
            if (!context) {
              reply(data, TUFFITE_STATUS_INVALID_STATE, 0);
              return;
            }
            auto* service =
                IsolatedServiceFactory::GetForBrowserContext(context.get());
            auto value = base::JSONReader::ReadDict(json, base::JSON_PARSE_RFC);
            auto result = value ? service->Apply(*value) : std::nullopt;
            uint64_t generation = 0;
            const bool ok =
                result && base::StringToUint64(*result, &generation);
            reply(data,
                  ok ? TUFFITE_STATUS_OK : TUFFITE_STATUS_INVALID_ARGUMENT,
                  generation);
          },
          host, json, reply_data, reply),
      base::BindOnce(
          [](void* data, tuffite_isolated_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE, 0);
          },
          reply_data, reply));
}

}  // namespace tuffite::abi
