// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_PUBLIC_C_API_CONTEXTS_H_
#define TUFFITE_FRAMEWORK_PUBLIC_C_API_CONTEXTS_H_
#include "tuffite/framework/public/c_api/host.h"
#include "tuffite/framework/public/c_api/utility.h"

// Trusted control-plane operations. UTF-8 JSON requests/results are copied at
// the boundary. Acceptance owns exactly one reply, including during shutdown.
// Handles are decimal strings, never floating-point JSON numbers. Tables live
// through the associated Rust runtime's stop, even after admission closes.
// A rejected completion means its receiver was dropped. Producers release any
// newly returned detached view rather than leaving an unreachable reference.
typedef tuffite_status_t (*tuffite_contexts_apply_fn)(
    void* host_data,
    tuffite_byte_slice_t request,
    void* reply_data,
    tuffite_utility_reply_fn reply);
typedef struct tuffite_contexts {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION void* user_data;
  tuffite_contexts_apply_fn apply;
} tuffite_contexts;
#endif
