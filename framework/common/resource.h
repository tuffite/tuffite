// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_RESOURCE_H_
#define TUFFITE_FRAMEWORK_COMMON_RESOURCE_H_

#include <string>

namespace tuffite {

// Optional in-memory resources supplied by a document-scoped owner.
struct MemoryAsset {
  std::string mime_type;
  std::string contents;
  std::string allow_origin;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_COMMON_RESOURCE_H_
