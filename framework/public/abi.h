// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_PUBLIC_ABI_H_
#define TUFFITE_FRAMEWORK_PUBLIC_ABI_H_

#include <stddef.h>
#include <stdint.h>

#include "tuffite/framework/public/abi_version.h"

// Opaque C ABI context pointers cannot use Chromium's C++ raw_ptr wrapper.
#if defined(__clang__) && defined(__cplusplus)
#define TUFFITE_FFI_RAW_PTR_EXCLUSION \
  __attribute__((annotate("raw_ptr_exclusion")))
#else
#define TUFFITE_FFI_RAW_PTR_EXCLUSION
#endif

typedef struct tuffite_abi_version {
  uint16_t major;
  uint16_t minor;
  uint16_t patch;
} tuffite_abi_version;

typedef struct tuffite_abi_header {
  size_t struct_size;
  tuffite_abi_version version;
  uint16_t reserved;
} tuffite_abi_header;

static inline tuffite_abi_version tuffite_abi_current_version(void) {
  tuffite_abi_version version = {TUFFITE_FFI_ABI_MAJOR, TUFFITE_FFI_ABI_MINOR,
                                 TUFFITE_FFI_ABI_PATCH};
  return version;
}

// Baseline required by the original tables in this stable major. Optional
// appended fields may require a newer minimum; do not require CURRENT blindly.
static inline tuffite_abi_version tuffite_abi_baseline_version(void) {
  tuffite_abi_version version = {TUFFITE_FFI_ABI_MAJOR,
                                 TUFFITE_FFI_ABI_MAJOR == 0 ? 1 : 0, 0};
  return version;
}

static inline tuffite_abi_header tuffite_abi_header_for_size(
    size_t struct_size) {
  tuffite_abi_header header = {struct_size, tuffite_abi_current_version(), 0};
  return header;
}

// Provider capabilities satisfy a stable semver minimum requirement.
static inline int tuffite_abi_is_compatible(tuffite_abi_version provided,
                                            tuffite_abi_version required) {
  return provided.major == required.major &&
         (provided.minor > required.minor ||
          (provided.minor == required.minor &&
           provided.patch >= required.patch));
}

typedef uint64_t tuffite_handle_t;

typedef int32_t tuffite_status_t;
#define TUFFITE_STATUS_OK ((tuffite_status_t)0)
#define TUFFITE_STATUS_INVALID_ARGUMENT ((tuffite_status_t)1)
#define TUFFITE_STATUS_INCOMPATIBLE_ABI ((tuffite_status_t)2)
#define TUFFITE_STATUS_INVALID_HANDLE ((tuffite_status_t)3)
#define TUFFITE_STATUS_INVALID_STATE ((tuffite_status_t)4)
#define TUFFITE_STATUS_RESOURCE_EXHAUSTED ((tuffite_status_t)5)
#define TUFFITE_STATUS_INTERNAL ((tuffite_status_t)6)
// ABI 0.1.0 resolver miss: native may fall back to app.assets.
#define TUFFITE_STATUS_NOT_FOUND ((tuffite_status_t)7)

typedef struct tuffite_byte_slice_t {
  const uint8_t* data;
  size_t len;
} tuffite_byte_slice_t;

typedef uint32_t tuffite_runtime_event_kind_t;
#define TUFFITE_RUNTIME_EVENT_STARTED ((tuffite_runtime_event_kind_t)1)
#define TUFFITE_RUNTIME_EVENT_DISPATCH_ACCEPTED \
  ((tuffite_runtime_event_kind_t)2)
#define TUFFITE_RUNTIME_EVENT_STOPPING ((tuffite_runtime_event_kind_t)3)
#define TUFFITE_RUNTIME_EVENT_COMMAND_RESPONSE ((tuffite_runtime_event_kind_t)4)
#define TUFFITE_RUNTIME_EVENT_APPLICATION_EVENT \
  ((tuffite_runtime_event_kind_t)5)

// This callback may be invoked from Runtime threads. It must copy or enqueue
// the borrowed payload and return without throwing, blocking for Runtime work,
// or synchronously re-entering an application callback.
typedef tuffite_status_t (*tuffite_post_event_fn)(
    void* user_data,
    tuffite_handle_t runtime,
    tuffite_runtime_event_kind_t kind,
    tuffite_byte_slice_t payload);

typedef uint32_t tuffite_task_priority_t;
#define TUFFITE_TASK_PRIORITY_BEST_EFFORT ((tuffite_task_priority_t)0)
#define TUFFITE_TASK_PRIORITY_USER_VISIBLE ((tuffite_task_priority_t)1)
#define TUFFITE_TASK_PRIORITY_USER_BLOCKING ((tuffite_task_priority_t)2)

typedef void (*tuffite_task_run_fn)(void* task_data);
typedef void (*tuffite_task_destroy_fn)(void* task_data);
typedef tuffite_status_t (*tuffite_post_task_fn)(
    void* user_data,
    tuffite_task_priority_t priority,
    uint8_t may_block,
    void* task_data,
    tuffite_task_run_fn run,
    tuffite_task_destroy_fn destroy);
typedef tuffite_status_t (*tuffite_utility_reply_fn)(
    void* user_data,
    tuffite_status_t status,
    tuffite_byte_slice_t payload);
typedef void (*tuffite_utility_host_create_reply_fn)(void* user_data,
                                                     tuffite_status_t status);
typedef tuffite_status_t (*tuffite_utility_host_event_fn)(
    void* user_data,
    uint64_t sequence,
    uint32_t kind,
    tuffite_byte_slice_t payload);
typedef void (*tuffite_utility_host_closed_fn)(void* user_data,
                                               tuffite_status_t status,
                                               uint64_t last_sequence,
                                               tuffite_byte_slice_t summary);
typedef tuffite_status_t (*tuffite_create_utility_host_session_fn)(
    void* host_data,
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
typedef tuffite_status_t (*tuffite_send_utility_host_session_fn)(
    void* host_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    uint64_t command_id,
    const uint8_t* command,
    size_t command_len,
    void* reply_data,
    tuffite_utility_reply_fn reply);
typedef tuffite_status_t (*tuffite_acknowledge_utility_host_session_fn)(
    void* host_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    uint64_t committed_sequence,
    uint32_t credits);
typedef tuffite_status_t (*tuffite_cancel_utility_host_session_fn)(
    void* host_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    uint64_t command_id);
typedef tuffite_status_t (*tuffite_close_utility_host_session_fn)(
    void* host_data,
    const uint8_t* name,
    size_t name_len,
    uint64_t session_id,
    uint32_t reason,
    void* reply_data,
    tuffite_utility_reply_fn reply);
// Shell API replies use RPC status codes; immediate acceptance uses ABI Status.
typedef tuffite_status_t (*tuffite_shell_api_reply_fn)(
    void* user_data,
    int32_t status,
    tuffite_byte_slice_t payload);
typedef tuffite_status_t (*tuffite_shell_api_invoke_fn)(
    void* host_data,
    const uint8_t* channel,
    size_t channel_len,
    const uint8_t* request,
    size_t request_len,
    void* reply_data,
    tuffite_shell_api_reply_fn reply);

// Browser-process operations routed by the host.
typedef struct tuffite_utility_session_client {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION void* user_data;
  tuffite_create_utility_host_session_fn create;
  tuffite_send_utility_host_session_fn send;
  tuffite_acknowledge_utility_host_session_fn acknowledge;
  tuffite_cancel_utility_host_session_fn cancel;
  tuffite_close_utility_host_session_fn close;
} tuffite_utility_session_client;

// Capability query is local, thread-safe, and returns borrowed immutable
// tables. Unknown IDs return null; tables and contexts live until application
// stop.
#define TUFFITE_CAPABILITY_SHELL_API ((uint32_t)0)
#define TUFFITE_CAPABILITY_UTILITY ((uint32_t)1)
#define TUFFITE_CAPABILITY_BROWSER ((uint32_t)2)
typedef const tuffite_abi_header* (
    *tuffite_query_extension_fn)(void* user_data, uint32_t capability);
typedef struct tuffite_utility {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION const tuffite_utility_session_client* session;
} tuffite_utility;
typedef struct tuffite_shell_api {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION void* user_data;
  tuffite_shell_api_invoke_fn invoke;
} tuffite_shell_api;

// ABI 0.1.0: browser-owned host capabilities. Child pointers are independently
// sized/versioned tables, borrowed until browser stop completes.
typedef struct tuffite_browser_host {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION const tuffite_shell_api* shell_api;
  TUFFITE_FFI_RAW_PTR_EXCLUSION const tuffite_utility* utility;
} tuffite_browser_host;

typedef struct tuffite_host {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION void* user_data;
  tuffite_post_event_fn post_event;
  tuffite_post_task_fn post_task;
  tuffite_query_extension_fn query_extension;
} tuffite_host;

typedef tuffite_status_t (*tuffite_application_start_fn)(
    const tuffite_host* host,
    const uint8_t* config,
    size_t config_len,
    tuffite_handle_t* runtime_out);
typedef tuffite_status_t (*tuffite_application_dispatch_fn)(
    tuffite_handle_t runtime,
    const uint8_t* event,
    size_t event_len);
typedef tuffite_status_t (*tuffite_application_stop_fn)(
    tuffite_handle_t runtime);
typedef tuffite_status_t (
    *tuffite_utility_is_registered_fn)(const uint8_t* name, size_t name_len);
typedef tuffite_status_t (*tuffite_utility_start_fn)(
    const uint8_t* name,
    size_t name_len,
    tuffite_handle_t* service_out);
typedef tuffite_status_t (*tuffite_utility_invoke_fn)(
    tuffite_handle_t service,
    uint64_t request_id,
    const uint8_t* request,
    size_t request_len,
    void* user_data,
    tuffite_utility_reply_fn reply);
typedef tuffite_status_t (*tuffite_utility_cancel_fn)(tuffite_handle_t service,
                                                      uint64_t request_id);
typedef tuffite_status_t (*tuffite_utility_stop_fn)(tuffite_handle_t service);
typedef tuffite_status_t (*tuffite_utility_session_event_fn)(
    void* user_data,
    uint64_t session_id,
    uint64_t sequence,
    uint32_t kind,
    tuffite_byte_slice_t payload);
typedef tuffite_status_t (*tuffite_utility_session_create_fn)(
    tuffite_handle_t service,
    uint64_t session_id,
    const uint8_t* init,
    size_t init_len,
    uint32_t initial_event_credits,
    size_t max_event_bytes,
    void* user_data,
    tuffite_utility_session_event_fn event,
    tuffite_handle_t* session_out);
typedef tuffite_status_t (*tuffite_utility_session_send_fn)(
    tuffite_handle_t session,
    uint64_t command_id,
    const uint8_t* command,
    size_t command_len,
    void* user_data,
    tuffite_utility_reply_fn reply);
typedef tuffite_status_t (*tuffite_utility_session_acknowledge_fn)(
    tuffite_handle_t session,
    uint64_t committed_sequence,
    uint32_t credits);
typedef tuffite_status_t (*tuffite_utility_session_cancel_fn)(
    tuffite_handle_t session,
    uint64_t command_id);
typedef tuffite_status_t (*tuffite_utility_session_close_fn)(
    tuffite_handle_t session,
    uint32_t reason,
    void* user_data,
    tuffite_utility_reply_fn reply);

// Utility-process operations implemented by the application.
typedef struct tuffite_utility_session_service {
  tuffite_abi_header header;
  tuffite_utility_session_create_fn create;
  tuffite_utility_session_send_fn send;
  tuffite_utility_session_acknowledge_fn acknowledge;
  tuffite_utility_session_cancel_fn cancel;
  tuffite_utility_session_close_fn close;
} tuffite_utility_session_service;

typedef struct tuffite_utility_service {
  tuffite_utility_is_registered_fn is_registered;
  tuffite_utility_start_fn start;
  tuffite_utility_invoke_fn invoke;
  tuffite_utility_cancel_fn cancel;
  tuffite_utility_stop_fn stop;
  // Immutable borrowed table and function code remain valid until Framework run
  // returns. Static storage is convenient, but not required by this C ABI.
  TUFFITE_FFI_RAW_PTR_EXCLUSION const tuffite_utility_session_service* session;
} tuffite_utility_service;

// ABI 0.1.0: synchronous browser argument preparation. Slices are borrowed;
// the sink copies each argument before returning. Child processes skip this
// hook.
typedef tuffite_status_t (*tuffite_launch_argument_fn)(
    void* user_data,
    tuffite_byte_slice_t argument);
typedef tuffite_status_t (*tuffite_prepare_browser_fn)(
    const tuffite_byte_slice_t* args,
    size_t count,
    void* user_data,
    tuffite_launch_argument_fn emit);

// ABI 0.1.0: native process role and selected utility runtime are raw integers.
#define TUFFITE_PROCESS_BROWSER ((uint32_t)0)
#define TUFFITE_PROCESS_UTILITY ((uint32_t)1)
#define TUFFITE_PROCESS_RENDERER ((uint32_t)2)
#define TUFFITE_PROCESS_GPU ((uint32_t)3)
#define TUFFITE_PROCESS_OTHER ((uint32_t)4)
#define TUFFITE_UTILITY_RUNTIME_RUST ((uint32_t)0)
#define TUFFITE_UTILITY_RUNTIME_V8 ((uint32_t)1)
typedef tuffite_status_t (*tuffite_prepare_process_fn)(
    uint32_t process_kind,
    const tuffite_byte_slice_t* args,
    size_t count,
    void* user_data,
    tuffite_launch_argument_fn emit);
typedef tuffite_status_t (*tuffite_utility_runtime_fn)(const uint8_t* name,
                                                       size_t name_len,
                                                       uint32_t* runtime);

// ABI 0.1.0: accepted resolution owns one asynchronous source reply. Context is
// thread-safe and remains valid until reply; rejection invokes no callback.
// A NOT_FOUND reply requests app.assets fallback; other errors fail loading.
typedef tuffite_status_t (*tuffite_resolve_v8_fn)(
    tuffite_handle_t runtime,
    tuffite_byte_slice_t name,
    tuffite_byte_slice_t url,
    void* user_data,
    tuffite_utility_reply_fn reply);

// ABI 0.1.0: borrowed service-local startup configuration, limited to 64 KiB.
typedef tuffite_status_t (*tuffite_utility_start_with_context_fn)(
    const uint8_t* name,
    size_t name_len,
    tuffite_byte_slice_t configuration,
    tuffite_handle_t* service_out);

typedef struct tuffite_application {
  tuffite_abi_header header;
  tuffite_application_start_fn start;
  tuffite_application_dispatch_fn dispatch;
  tuffite_application_stop_fn stop;
  tuffite_utility_service utility;
  tuffite_prepare_browser_fn prepare_browser;
  tuffite_prepare_process_fn prepare_process;
  tuffite_utility_runtime_fn utility_runtime;
  tuffite_resolve_v8_fn resolve_v8;
  tuffite_utility_start_with_context_fn utility_start_with_context;
} tuffite_application;

#if defined(_WIN32)
#if defined(TUFFITE_FRAMEWORK_IMPLEMENTATION)
#define TUFFITE_FRAMEWORK_EXPORT __declspec(dllexport)
#else
#define TUFFITE_FRAMEWORK_EXPORT __declspec(dllimport)
#endif
#else
#define TUFFITE_FRAMEWORK_EXPORT __attribute__((visibility("default")))
#endif

// C-only boundary between the application process and the shared Framework
// runtime. On Windows, the executable initializes sandbox state and passes it
// through this interface.
#ifdef __cplusplus
extern "C" {
#endif
TUFFITE_FRAMEWORK_EXPORT int tuffite_run_application(
    const tuffite_application* application,
    int argc,
    const char* const* argv,
    void* sandbox_info);

// Process bootstrap entry implemented by the shared Framework. Keeping its
// Chromium dependencies out of the macOS executable prevents a second copy of
// PartitionAlloc from being initialized in the same process.
TUFFITE_FRAMEWORK_EXPORT int tuffite_bootstrap(int argc,
                                               const char* const* argv,
                                               void* sandbox_info);
#ifdef __cplusplus
}  // extern "C"
#endif

#undef TUFFITE_FFI_RAW_PTR_EXCLUSION

#endif  // TUFFITE_FRAMEWORK_PUBLIC_ABI_H_
