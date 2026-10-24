// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_EXPLORER_FRAME_ANCESTRY_H_
#define TUFFITE_EXPLORER_FRAME_ANCESTRY_H_
#include "third_party/blink/renderer/core/core_export.h"
namespace blink {
class Frame;
class DOMWindow;
struct FramePolicy;
CORE_EXPORT const FramePolicy* ExplorerFrameOwnerPolicy(Frame&);
CORE_EXPORT Frame* ExplorerParentFrame(Frame&);
CORE_EXPORT Frame* ExplorerTopFrame(Frame&);
CORE_EXPORT bool ExplorerIsolatedFrame(Frame&);
// Permissions inheritance follows actual ownership even when DOM ancestry
// stops.
CORE_EXPORT Frame* ExplorerPermissionsPolicyParent(Frame&);
}  // namespace blink
#endif
