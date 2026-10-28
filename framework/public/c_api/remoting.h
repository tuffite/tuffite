// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_PUBLIC_C_API_REMOTING_H_
#define TUFFITE_FRAMEWORK_PUBLIC_C_API_REMOTING_H_

#include "tuffite/framework/public/c_api/types.h"
// Requests/responses use the Remoting Mojom data codec. Result status uses the
// dispatch status values. OK admission transfers reply ownership; rejection
// retains ownership and never replies. Accepted calls reply exactly once,
// including during Host shutdown. Request bytes are copied before returning.
typedef tuffite_status_t (*tuffite_remoting_reply_fn)(
    void* reply_data,
    int32_t rpc_status,
    tuffite_byte_slice_t response);
typedef tuffite_status_t (*tuffite_remoting_call_fn)(
    void* host_data,
    tuffite_byte_slice_t request,
    void* reply_data,
    tuffite_remoting_reply_fn reply);

typedef struct tuffite_remoting {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION void* user_data;
  tuffite_remoting_call_fn capabilities;
  tuffite_remoting_call_fn start;
  tuffite_remoting_call_fn process_signal;
  tuffite_remoting_call_fn take_signals;
  tuffite_remoting_call_fn send_input;
  tuffite_remoting_call_fn inject_input;
  tuffite_remoting_call_fn stop;
  tuffite_remoting_call_fn stats;
  tuffite_remoting_call_fn capture;
} tuffite_remoting;
#endif  // TUFFITE_FRAMEWORK_PUBLIC_C_API_REMOTING_H_
