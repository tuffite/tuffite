// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_ABI_HOST_BRIDGE_H_
#define TUFFITE_FRAMEWORK_APP_ABI_HOST_BRIDGE_H_

#include <stddef.h>
#include <stdint.h>

#include <atomic>
#include <memory>
#include <string_view>
#include <unordered_map>
#include <vector>

#include "base/containers/span.h"
#include "base/functional/callback.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/raw_ptr_exclusion.h"
#include "base/memory/scoped_refptr.h"
#include "base/memory/weak_ptr.h"
#include "base/sequence_checker.h"
#include "base/synchronization/lock.h"
#include "base/task/sequenced_task_runner.h"
#include "base/timer/timer.h"
#include "base/unguessable_token.h"
#include "tuffite/components/dispatch/browser/host_dispatch.h"
#include "tuffite/framework/public/abi.h"

namespace tuffite {
class ApplicationContext;
namespace rpc {
class Router;
}

// Adapts the host application's callback table and owns one Runtime instance.
//
// Dispatch is asynchronous: the browser posts a request via
// `Dispatch()` with a completion callback and returns immediately. The host
// scheduler runs the handler off the UI thread and eventually invokes the
// stored callback on the browser task runner with the response bytes.
class AbiHostBridge : public HostDispatch {
 public:
  // Complete Shell Protocol response, with the caller request ID restored.
  // bool reports transport success; protocol errors stay in the frame.
  using DispatchCallback =
      base::OnceCallback<void(bool ok, std::vector<uint8_t> response)>;
  static std::unique_ptr<AbiHostBridge> Create(
      const tuffite_application* application);

  AbiHostBridge(const AbiHostBridge&) = delete;
  AbiHostBridge& operator=(const AbiHostBridge&) = delete;
  ~AbiHostBridge() override;

  const tuffite_application& application() const { return application_; }

  bool Start(std::string_view configuration);
  void BindBrowserTaskRunner();
  // Asynchronous services-ready barrier before startup windows and dispatch.
  void InitializeBrowser(base::OnceCallback<void(bool)> callback);
  void BindApplicationContext(ApplicationContext* context) {
    application_context_ = context;
  }
  ApplicationContext* application_context() const {
    return application_context_;
  }

  // Sends |event| to Runtime and invokes |callback| when the matching
  // response arrives. Returns false when the runtime is not ready or the
  // scheduler applies backpressure; in that case |callback| is invoked
  // synchronously with a failure.
  bool Dispatch(std::vector<uint8_t> event,
                DispatchCallback callback,
                const base::UnguessableToken& document) override;
  void CancelDocument(const base::UnguessableToken& document) override;
  void BeginShutdown();

  void Stop();

  bool initialization_failed() const { return initialization_failed_; }
  bool is_running() const { return runtime_.load() != 0; }
  tuffite_handle_t runtime_handle() const { return runtime_.load(); }

 private:
  explicit AbiHostBridge(const tuffite_application& application);
  static const tuffite_abi_header* QueryExtension(void* user_data,
                                                  uint32_t capability);
  tuffite_utility_session_client utility_session_{};
  tuffite_utility utility_{};
  tuffite_shell_api shell_api_{};
  tuffite_browser_host browser_host_{};
  std::unique_ptr<rpc::Router> shell_api_router_;
  const base::UnguessableToken native_owner_ = base::UnguessableToken::Create();
  static tuffite_status_t PostEvent(void* user_data,
                                    tuffite_handle_t runtime,
                                    tuffite_runtime_event_kind_t kind,
                                    tuffite_byte_slice_t payload);
  static tuffite_status_t PostTask(void* user_data,
                                   tuffite_task_priority_t priority,
                                   uint8_t may_block,
                                   void* task_data,
                                   tuffite_task_run_fn run,
                                   tuffite_task_destroy_fn destroy);
  static tuffite_status_t CreateUtilityHostSession(
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
      tuffite_utility_host_create_reply_fn created);
  static tuffite_status_t SendUtilityHostSession(
      void* user_data,
      const uint8_t* name,
      size_t name_len,
      uint64_t session_id,
      uint64_t command_id,
      const uint8_t* command,
      size_t command_len,
      void* reply_data,
      tuffite_utility_reply_fn reply);
  static tuffite_status_t AcknowledgeUtilityHostSession(
      void* user_data,
      const uint8_t* name,
      size_t name_len,
      uint64_t session_id,
      uint64_t committed_sequence,
      uint32_t credits);
  static tuffite_status_t CancelUtilityHostSession(void* user_data,
                                                   const uint8_t* name,
                                                   size_t name_len,
                                                   uint64_t session_id,
                                                   uint64_t command_id);
  static tuffite_status_t CloseUtilityHostSession(
      void* user_data,
      const uint8_t* name,
      size_t name_len,
      uint64_t session_id,
      uint32_t reason,
      void* reply_data,
      tuffite_utility_reply_fn reply);
  static tuffite_status_t InvokeShellApi(void* user_data,
                                         const uint8_t* channel,
                                         size_t channel_len,
                                         const uint8_t* request,
                                         size_t request_len,
                                         void* reply_data,
                                         tuffite_shell_api_reply_fn reply);
  // Extracts request_id from a shell-protocol response header. Header
  // layout: magic, version, status, request_id, payload length, payload.
  static bool ReadResponseRequestId(base::span<const uint8_t> data,
                                    uint64_t* request_id);

  // Called on the browser task runner. Looks up the pending request and
  // invokes its callback.
  void CompletePending(uint64_t request_id,
                       bool ok,
                       std::vector<uint8_t> response);
  void FailPending();
  void CompleteShellApi(uint64_t id,
                        int32_t status,
                        const std::vector<uint8_t>& response);
  struct PendingShellApi {
    // Opaque Rust reply allocation; freed by its callback, outside
    // PartitionAlloc.
    RAW_PTR_EXCLUSION void* data;
    tuffite_shell_api_reply_fn reply;
  };
  std::unordered_map<uint64_t, PendingShellApi> pending_shell_api_;
  uint64_t next_shell_api_id_ = 1;
  void BroadcastApplicationEvent(std::string name,
                                 std::vector<uint8_t> payload);
  tuffite_status_t PostHostCall(base::OnceClosure, base::OnceClosure rejected);
  void RunHostCall(uint64_t, base::OnceClosure);

  tuffite_application_start_fn runtime_start_ = nullptr;
  tuffite_application_dispatch_fn runtime_dispatch_ = nullptr;
  tuffite_application_stop_fn runtime_stop_ = nullptr;
  bool browser_initialized_ = false;
  bool initialization_failed_ = false;
  std::atomic<bool> accepting_tasks_{true};
  std::atomic<tuffite_handle_t> runtime_{0};
  const tuffite_application application_;
  scoped_refptr<base::SequencedTaskRunner> browser_task_runner_;
  raw_ptr<ApplicationContext> application_context_ = nullptr;

  // Pending Mojo callbacks are owned exclusively by the browser sequence.
  // Runtime worker callbacks only post copied data back to that sequence.
  struct PendingRequest {
    DispatchCallback callback;
    uint64_t caller_id;
    base::UnguessableToken document;
    std::unique_ptr<base::OneShotTimer> timeout;
  };
  std::unordered_map<uint64_t, PendingRequest> pending_;
  std::atomic<bool> accepting_requests_{true};
  base::Lock host_calls_lock_;
  std::unordered_map<uint64_t, base::OnceClosure> queued_host_calls_;
  uint64_t next_host_call_id_ = 1;
  uint64_t next_request_id_ = 1;
  SEQUENCE_CHECKER(browser_sequence_checker_);
  // Created on the browser sequence; workers only copy it into posted tasks.
  base::WeakPtr<AbiHostBridge> weak_this_;
  base::WeakPtrFactory<AbiHostBridge> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_APP_ABI_HOST_BRIDGE_H_
