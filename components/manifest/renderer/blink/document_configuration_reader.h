// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_MANIFEST_RENDERER_BLINK_DOCUMENT_CONFIGURATION_READER_H_
#define TUFFITE_COMPONENTS_MANIFEST_RENDERER_BLINK_DOCUMENT_CONFIGURATION_READER_H_

#include "v8/include/v8-local-handle.h"
#if defined(COMPONENT_BUILD) && defined(_WIN32)
#if defined(BLINK_CORE_IMPLEMENTATION) && BLINK_CORE_IMPLEMENTATION
#define TUFFITE_MANIFEST_DOCUMENT_READER_EXPORT __declspec(dllexport)
#else
#define TUFFITE_MANIFEST_DOCUMENT_READER_EXPORT __declspec(dllimport)
#endif
#elif defined(COMPONENT_BUILD)
#define TUFFITE_MANIFEST_DOCUMENT_READER_EXPORT \
  __attribute__((visibility("default")))
#else
#define TUFFITE_MANIFEST_DOCUMENT_READER_EXPORT
#endif

namespace v8 {
class Context;
}

namespace blink {
class Document;
}

namespace tuffite {
TUFFITE_MANIFEST_DOCUMENT_READER_EXPORT bool IsShellApiExposed(
    v8::Local<v8::Context>);
bool IsExplorerEnabled(const blink::Document&);
}  // namespace tuffite

#endif
