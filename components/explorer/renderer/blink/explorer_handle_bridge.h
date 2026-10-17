// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_EXPLORER_HANDLE_BRIDGE_H_
#define TUFFITE_EXPLORER_HANDLE_BRIDGE_H_
#include <optional>
#include <vector>

#include "third_party/blink/public/common/frame/frame_policy.h"
#include "third_party/blink/public/common/tokens/tokens.h"
#include "v8/include/v8-function-callback.h"

// These feature-owned adapters are implemented in Blink Core. Consumers use
// only V8 types, without a dependency on Blink-private implementation headers.
#if defined(COMPONENT_BUILD) && defined(_WIN32)
#if defined(BLINK_CORE_IMPLEMENTATION) && BLINK_CORE_IMPLEMENTATION
#define TUFFITE_EXPLORER_BRIDGE_EXPORT __declspec(dllexport)
#else
#define TUFFITE_EXPLORER_BRIDGE_EXPORT __declspec(dllimport)
#endif
#elif defined(COMPONENT_BUILD)
#define TUFFITE_EXPLORER_BRIDGE_EXPORT __attribute__((visibility("default")))
#else
#define TUFFITE_EXPLORER_BRIDGE_EXPORT
#endif
namespace blink {
class WebFrame;
TUFFITE_EXPLORER_BRIDGE_EXPORT bool SetExplorerFrameAncestors(
    WebFrame*,
    const std::vector<FrameToken>&,
    const std::optional<FramePolicy>& = std::nullopt);
TUFFITE_EXPLORER_BRIDGE_EXPORT void PostExplorerSurfaceMessage(
    const v8::FunctionCallbackInfo<v8::Value>&);
TUFFITE_EXPLORER_BRIDGE_EXPORT void PostExplorerWindowMessage(
    const v8::FunctionCallbackInfo<v8::Value>&);
TUFFITE_EXPLORER_BRIDGE_EXPORT void AttachExplorerWindow(
    const v8::FunctionCallbackInfo<v8::Value>&);
}  // namespace blink
#endif
