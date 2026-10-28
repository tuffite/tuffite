// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_PUBLIC_C_API_HOST_H_
#define TUFFITE_FRAMEWORK_PUBLIC_C_API_HOST_H_

#include "tuffite/framework/public/c_api/types.h"

typedef uint32_t tuffite_runtime_event_kind_t;
#define TUFFITE_RUNTIME_EVENT_STARTED 1
#define TUFFITE_RUNTIME_EVENT_DISPATCH_ACCEPTED 2
#define TUFFITE_RUNTIME_EVENT_STOPPING 3
#define TUFFITE_RUNTIME_EVENT_COMMAND_RESPONSE 4
#define TUFFITE_RUNTIME_EVENT_APPLICATION_EVENT 5

// This callback may be invoked from Runtime threads. It must copy or enqueue
// the borrowed payload and return without throwing, blocking for Runtime work,
// or synchronously re-entering an application callback.
typedef tuffite_status_t (*tuffite_post_event_fn)(
    void* user_data,
    tuffite_handle_t runtime,
    tuffite_runtime_event_kind_t kind,
    tuffite_byte_slice_t payload);

typedef uint32_t tuffite_task_priority_t;
#define TUFFITE_TASK_PRIORITY_BEST_EFFORT 0
#define TUFFITE_TASK_PRIORITY_USER_VISIBLE 1
#define TUFFITE_TASK_PRIORITY_USER_BLOCKING 2

typedef void (*tuffite_task_run_fn)(void* task_data);
typedef void (*tuffite_task_destroy_fn)(void* task_data);
typedef tuffite_status_t (*tuffite_post_task_fn)(
    void* user_data,
    tuffite_task_priority_t priority,
    uint8_t may_block,
    void* task_data,
    tuffite_task_run_fn run,
    tuffite_task_destroy_fn destroy);
// Capability query is local, thread-safe, and returns borrowed immutable
// tables. Unknown IDs return null; tables and contexts live until application
// stop.
// Native capabilities are discovered independently by domain.
#define TUFFITE_CAPABILITY_UTILITY_SESSION 0
#define TUFFITE_CAPABILITY_UTILITY_INTERFACES 1
#define TUFFITE_CAPABILITY_ISOLATED 2
#define TUFFITE_CAPABILITY_CONTEXTS 3
#define TUFFITE_CAPABILITY_REMOTING 4
typedef const tuffite_abi_header* (
    *tuffite_query_extension_fn)(void* user_data, uint32_t capability);
typedef struct tuffite_host {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION void* user_data;
  tuffite_post_event_fn post_event;
  tuffite_post_task_fn post_task;
  tuffite_query_extension_fn query_extension;
} tuffite_host;

#endif  // TUFFITE_FRAMEWORK_PUBLIC_C_API_HOST_H_
