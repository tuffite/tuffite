// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_PUBLIC_C_API_ISOLATED_H_
#define TUFFITE_FRAMEWORK_PUBLIC_C_API_ISOLATED_H_

#include "tuffite/framework/public/c_api/types.h"

// Optional DOM isolated-world registry capability. Request is a UTF-8 JSON
// registry operation, copied before returning. OK transfers reply ownership;
// any other return retains ownership and must not invoke reply. An accepted
// operation replies exactly once (including during shutdown). Status uses the
// TUFFITE_STATUS_* ABI values; generation is zero for unregister/failure.
typedef tuffite_status_t (*tuffite_isolated_reply_fn)(void* reply_data,
                                                      tuffite_status_t status,
                                                      uint64_t generation);
typedef tuffite_status_t (*tuffite_isolated_apply_fn)(
    void* host_data,
    tuffite_byte_slice_t request,
    void* reply_data,
    tuffite_isolated_reply_fn reply);
typedef struct tuffite_isolated {
  tuffite_abi_header header;
  TUFFITE_FFI_RAW_PTR_EXCLUSION void* user_data;
  tuffite_isolated_apply_fn apply;
} tuffite_isolated;

#endif  // TUFFITE_FRAMEWORK_PUBLIC_C_API_ISOLATED_H_
