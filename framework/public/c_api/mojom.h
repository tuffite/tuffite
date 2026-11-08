// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_PUBLIC_C_API_MOJOM_H_
#define TUFFITE_FRAMEWORK_PUBLIC_C_API_MOJOM_H_
#include "tuffite/framework/public/c_api/utility.h"
// Borrowed metadata from the application's frozen, generated interface
// registry. This is an in-process ABI descriptor, never a renderer-controlled
// wire header.
typedef struct tuffite_mojom_method {
  tuffite_byte_slice_t channel;
  uint64_t command_id;
  uint64_t schema_fingerprint;
} tuffite_mojom_method;
typedef tuffite_status_t (*tuffite_resolve_mojom_method_fn)(
    tuffite_byte_slice_t interface_name,
    uint32_t ordinal,
    tuffite_mojom_method* method_out);
// Payloads are the method's Mojom parameter/result structs, without envelopes.
// Failed replies may carry UTF-8 error details instead of a result struct.
// Acceptance owns exactly one asynchronous reply; rejection invokes none.
typedef tuffite_status_t (*tuffite_dispatch_mojom_fn)(
    tuffite_handle_t runtime,
    uint64_t command_id,
    uint64_t schema_fingerprint,
    tuffite_byte_slice_t parameters,
    void* reply_data,
    tuffite_utility_reply_fn reply);
typedef tuffite_status_t (*tuffite_invoke_utility_mojom_fn)(
    tuffite_handle_t service,
    uint64_t request_id,
    uint64_t command_id,
    uint64_t schema_fingerprint,
    tuffite_byte_slice_t parameters,
    void* reply_data,
    tuffite_utility_reply_fn reply);
#endif
