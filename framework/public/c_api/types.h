// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_PUBLIC_C_API_ABI_TYPES_H_
#define TUFFITE_FRAMEWORK_PUBLIC_C_API_ABI_TYPES_H_

#include <stddef.h>
#include <stdint.h>

#include "tuffite/framework/public/c_api/version.h"

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
#define TUFFITE_STATUS_OK 0
#define TUFFITE_STATUS_INVALID_ARGUMENT 1
#define TUFFITE_STATUS_INCOMPATIBLE_ABI 2
#define TUFFITE_STATUS_INVALID_HANDLE 3
#define TUFFITE_STATUS_INVALID_STATE 4
#define TUFFITE_STATUS_RESOURCE_EXHAUSTED 5
#define TUFFITE_STATUS_INTERNAL 6
// ABI 0.1.0 resolver miss: native may fall back to app.assets.
#define TUFFITE_STATUS_NOT_FOUND 7

typedef struct tuffite_byte_slice_t {
  const uint8_t* data;
  size_t len;
} tuffite_byte_slice_t;

#endif  // TUFFITE_FRAMEWORK_PUBLIC_C_API_ABI_TYPES_H_
