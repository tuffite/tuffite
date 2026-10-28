// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_PUBLIC_C_API_UTILITY_H_
#define TUFFITE_FRAMEWORK_PUBLIC_C_API_UTILITY_H_

#include "tuffite/framework/public/c_api/types.h"

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

#endif  // TUFFITE_FRAMEWORK_PUBLIC_C_API_UTILITY_H_
