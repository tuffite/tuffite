// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_ISOLATED_RENDERER_BLINK_ISOLATED_BRIDGE_H_
#define TUFFITE_COMPONENTS_ISOLATED_RENDERER_BLINK_ISOLATED_BRIDGE_H_

#include "v8/include/v8-function.h"
#include "v8/include/v8-local-handle.h"

#if defined(COMPONENT_BUILD) && defined(_WIN32)
#if defined(BLINK_CORE_IMPLEMENTATION) && BLINK_CORE_IMPLEMENTATION
#define TUFFITE_ISOLATED_EXPORT __declspec(dllexport)
#else
#define TUFFITE_ISOLATED_EXPORT __declspec(dllimport)
#endif
#elif defined(COMPONENT_BUILD)
#define TUFFITE_ISOLATED_EXPORT __attribute__((visibility("default")))
#else
#define TUFFITE_ISOLATED_EXPORT
#endif

namespace blink {
  class ScriptState;
  TUFFITE_ISOLATED_EXPORT void InstallIsolatedHost(v8::Local<v8::Context>,
    v8::Local<v8::Function> load);
  TUFFITE_ISOLATED_EXPORT bool IsTuffiteIsolatedWorld(ScriptState*);
}  // namespace blink
#endif
