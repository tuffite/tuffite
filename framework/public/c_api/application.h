// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_PUBLIC_C_API_APPLICATION_H_
#define TUFFITE_FRAMEWORK_PUBLIC_C_API_APPLICATION_H_

#include "tuffite/framework/public/c_api/host.h"
#include "tuffite/framework/public/c_api/utility.h"

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
#define TUFFITE_PROCESS_BROWSER 0
#define TUFFITE_PROCESS_UTILITY 1
#define TUFFITE_PROCESS_RENDERER 2
#define TUFFITE_PROCESS_GPU 3
#define TUFFITE_PROCESS_OTHER 4
#define TUFFITE_UTILITY_RUNTIME_RUST 0
#define TUFFITE_UTILITY_RUNTIME_V8 1
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
    uint64_t session_id,
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

// Pure lookup for an interface or typed owner. The returned owner borrows
// application-lifetime storage. The caller copies it without transferring
// ownership.
typedef tuffite_status_t (*tuffite_resolve_utility_route_fn)(
    tuffite_byte_slice_t name,
    tuffite_byte_slice_t* owner_out);

// Browser-only trusted setup, before services/windows. Emits denied origin
// patterns through a synchronous copying sink. Absent in older tables.
typedef tuffite_status_t (*tuffite_setup_browser_fn)(
    void* user_data,
    tuffite_launch_argument_fn emit);

// Acceptance owns exactly one reply from a Rust worker; rejection owns none.
typedef tuffite_status_t (*tuffite_initialize_browser_fn)(
    tuffite_handle_t runtime,
    void* user_data,
    tuffite_utility_reply_fn reply);

typedef tuffite_status_t (*tuffite_cancel_v8_session_fn)(
    tuffite_handle_t runtime,
    tuffite_byte_slice_t name,
    uint64_t session_id);

// Optional multi-profile extension. Parent identifies the application runtime;
// each successful call creates an independent Browser runtime.
typedef tuffite_status_t (*tuffite_start_profile_fn)(
    tuffite_handle_t parent,
    const tuffite_host* host,
    tuffite_byte_slice_t configuration,
    tuffite_byte_slice_t profile_id,
    tuffite_handle_t* runtime_out);
// 0 = unknown, 1 = Application, 2 = Browser. Pure static lookup.
typedef uint32_t (*tuffite_command_scope_fn)(uint64_t command_id);

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
  tuffite_resolve_utility_route_fn resolve_utility_route;
  tuffite_setup_browser_fn setup_browser;
  tuffite_initialize_browser_fn initialize_browser;
  tuffite_cancel_v8_session_fn cancel_v8_session;
  tuffite_start_profile_fn start_profile;
  tuffite_command_scope_fn command_scope;
  // Application-lifetime owned declarations, frozen before Chromium startup.
  tuffite_byte_slice_t (*configuration)(void);
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

#endif  // TUFFITE_FRAMEWORK_PUBLIC_C_API_APPLICATION_H_
