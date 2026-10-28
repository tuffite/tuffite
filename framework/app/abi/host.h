// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_ABI_HOST_H_
#define TUFFITE_FRAMEWORK_APP_ABI_HOST_H_

#include <atomic>
#include <memory>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

#include "base/functional/callback.h"
#include "base/memory/raw_ptr_exclusion.h"
#include "base/memory/scoped_refptr.h"
#include "base/memory/weak_ptr.h"
#include "base/sequence_checker.h"
#include "base/synchronization/lock.h"
#include "base/task/sequenced_task_runner.h"
#include "base/unguessable_token.h"
#include "tuffite/framework/public/c_api/contexts.h"
#include "tuffite/framework/public/c_api/host.h"
#include "tuffite/framework/public/c_api/isolated.h"
#include "tuffite/framework/public/c_api/remoting.h"
#include "tuffite/framework/public/c_api/utility.h"

namespace tuffite {
class ApplicationContext;
class BrowserContext;
class UtilityService;

}  // namespace tuffite

namespace tuffite::abi {

// Implements the Rust-to-native ABI. Never owns services or a Rust runtime.
class Host final {
 public:
  using EventHandler =
      base::RepeatingCallback<tuffite_status_t(tuffite_handle_t,
                                               tuffite_runtime_event_kind_t,
                                               tuffite_byte_slice_t)>;
  using ApplicationEvent =
      base::RepeatingCallback<void(std::string, std::vector<uint8_t>)>;

  explicit Host(EventHandler events);
  ~Host();
  Host(const Host&) = delete;
  Host& operator=(const Host&) = delete;

  const tuffite_host& table() const { return host_; }

  void BindBrowserTaskRunner();
  void BindApplicationContext(ApplicationContext* context);
  ApplicationContext* application_context() const {
    return application_context_.get();
  }
  void BindBrowserContext(base::WeakPtr<BrowserContext> context) {
    browser_context_ = std::move(context);
  }
  bool has_browser_context() const { return !!browser_context_; }
  ApplicationEvent application_event();

  // Close business admission, retaining scheduling and callback tables for Rust
  // stop.
  void Close();

  // Called after Rust stop returns. Tables remain allocated until destruction.
  void Shutdown();

 private:
  base::WeakPtr<BrowserContext> browser_context() const {
    return browser_context_;
  }
  tuffite_status_t PostHostCall(base::OnceClosure, base::OnceClosure rejected);
  void InitializeAbi();
  template <typename Method, auto Handler>
  static tuffite_status_t CallRemoting(void*,
                                       tuffite_byte_slice_t,
                                       void*,
                                       tuffite_remoting_reply_fn);
  void CompleteRemoting(uint64_t, int32_t, const std::vector<uint8_t>&);
  struct PendingRemoting {
    TUFFITE_FFI_RAW_PTR_EXCLUSION void* data;
    tuffite_remoting_reply_fn reply;
  };
  std::unordered_map<uint64_t, PendingRemoting> pending_remoting_;
  uint64_t next_remoting_id_ = 1;
  tuffite_remoting remoting_{};
  UtilityService* utility_service() const;
  static tuffite_status_t ApplyContexts(void*,
                                        tuffite_byte_slice_t,
                                        void*,
                                        tuffite_utility_reply_fn);
  static tuffite_status_t ApplyIsolated(void* data,
                                        tuffite_byte_slice_t request,
                                        void* reply_data,
                                        tuffite_isolated_reply_fn reply);
  static const tuffite_abi_header* QueryExtension(void* user_data,
                                                  uint32_t capability);
  base::WeakPtr<BrowserContext> browser_context_;
  tuffite_host host_{};
  tuffite_isolated isolated_{};
  tuffite_contexts contexts_{};
  tuffite_utility_session_client utility_session_{};
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

  void BroadcastApplicationEvent(std::string name,
                                 std::vector<uint8_t> payload);
  void RunHostCall(uint64_t, base::OnceClosure);

  const EventHandler events_;
  base::WeakPtr<ApplicationContext> application_context_;
  scoped_refptr<base::SequencedTaskRunner> browser_task_runner_;
  std::atomic<bool> accepting_tasks_{true};
  std::atomic<bool> accepting_requests_{true};
  base::Lock host_calls_lock_;
  std::unordered_map<uint64_t, base::OnceClosure> queued_host_calls_;
  uint64_t next_host_call_id_ = 1;
  SEQUENCE_CHECKER(browser_sequence_checker_);
  base::WeakPtr<Host> weak_this_;
  base::WeakPtrFactory<Host> weak_factory_{this};
};

}  // namespace tuffite::abi

#endif  // TUFFITE_FRAMEWORK_APP_ABI_HOST_H_
