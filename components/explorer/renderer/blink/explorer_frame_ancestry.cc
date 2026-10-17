// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/renderer/blink/explorer_frame_ancestry.h"

#include <optional>

#include "third_party/blink/public/web/web_frame.h"
#include "third_party/blink/renderer/core/execution_context/security_context_init.h"
#include "third_party/blink/renderer/core/frame/dom_window.h"
#include "third_party/blink/renderer/core/frame/frame.h"
#include "third_party/blink/renderer/core/frame/local_dom_window.h"
#include "third_party/blink/renderer/core/frame/local_frame.h"
#include "third_party/blink/renderer/core/frame/location.h"
#include "third_party/blink/renderer/core/frame/remote_frame.h"
#include "third_party/blink/renderer/core/frame/web_local_frame_impl.h"
#include "third_party/blink/renderer/core/frame/web_remote_frame_impl.h"
#include "third_party/blink/renderer/core/loader/document_loader.h"
#include "third_party/blink/renderer/core/page/frame_tree.h"
#include "third_party/blink/renderer/platform/heap/collection_support/heap_hash_map.h"
#include "third_party/blink/renderer/platform/heap/persistent.h"
#include "tuffite/components/explorer/renderer/blink/explorer_handle_bridge.h"

namespace blink {
namespace {
class Ancestors final : public GarbageCollected<Ancestors> {
 public:
  Ancestors();
  ~Ancestors();
  void Trace(Visitor*) const {}
  std::optional<FrameToken> parent;
  std::optional<FrameToken> top;
  std::optional<FramePolicy> policy;
};
Ancestors::Ancestors() = default;
Ancestors::~Ancestors() = default;
using AncestorMap = GCedHeapHashMap<WeakMember<Frame>, Member<Ancestors>>;
AncestorMap& Map() {
  DEFINE_STATIC_LOCAL(Persistent<AncestorMap>, map,
                      (MakeGarbageCollected<AncestorMap>()));
  return *map;
}
void RefreshPermissions(Frame& frame) {
  if (auto* local = DynamicTo<LocalFrame>(frame)) {
    auto* window = local->DomWindow();
    if (window && local->GetDocument() && local->GetDocument()->Loader() &&
        window->GetSecurityContext().GetPermissionsPolicy()) {
      SecurityContextInit init(window);
      init.ApplyPermissionsPolicy(
          *local, local->GetDocument()->Loader()->GetResponse(), FramePolicy(),
          std::nullopt, std::nullopt, local->GetDocument()->Url());
    }
  } else {
    To<RemoteFrame>(frame).RefreshTuffitePermissionsPolicy();
  }
}
void Set(Frame* frame,
         std::optional<FrameToken> parent,
         std::optional<FrameToken> top,
         std::optional<FramePolicy> policy = std::nullopt) {
  if (!frame) {
    return;
  }
  if (frame->IsLocalFrame()) {
    for (auto* descendant = frame; descendant;
         descendant = descendant->Tree().TraverseNext(frame)) {
      if (auto* local = DynamicTo<LocalFrame>(descendant)) {
        local->DomWindow()->location()->InvalidateTuffiteAncestorOrigins();
      }
    }
  }
  if (!parent && !top) {
    Map().erase(frame);
    RefreshPermissions(*frame);
    return;
  }
  auto* value = MakeGarbageCollected<Ancestors>();
  value->parent = parent;
  value->top = top;
  value->policy = std::move(policy);
  Map().Set(frame, value);
  RefreshPermissions(*frame);
}
Frame* Resolve(const FrameToken& token) {
  return token.Is<LocalFrameToken>()
             ? static_cast<Frame*>(
                   LocalFrame::FromFrameToken(token.GetAs<LocalFrameToken>()))
             : static_cast<Frame*>(RemoteFrame::FromFrameToken(
                   token.GetAs<RemoteFrameToken>()));
}
Frame* Lookup(Frame& frame, bool top) {
  auto found = Map().find(&frame);
  if (found == Map().end()) {
    return nullptr;
  }
  auto token = top ? found->value->top : found->value->parent;
  auto* remote = token ? Resolve(*token) : nullptr;
  return remote;
}
}  // namespace
const FramePolicy* ExplorerFrameOwnerPolicy(Frame& frame) {
  auto found = Map().find(&frame);
  return found != Map().end() && found->value->policy ? &*found->value->policy
                                                      : nullptr;
}
Frame* ExplorerParentFrame(Frame& frame) {
  return frame.Parent() ? nullptr : Lookup(frame, false);
}
Frame* ExplorerTopFrame(Frame& frame) {
  return Lookup(*frame.Top(), true);
}
bool SetExplorerFrameAncestors(WebFrame* web_frame,
                               const std::vector<FrameToken>& ancestors,
                               const std::optional<FramePolicy>& policy) {
  if (!web_frame) {
    return false;
  }
  auto* frame =
      web_frame->IsWebLocalFrame()
          ? static_cast<Frame*>(
                static_cast<WebLocalFrameImpl*>(web_frame)->GetFrame())
          : static_cast<Frame*>(
                static_cast<WebRemoteFrameImpl*>(web_frame)->GetFrame());
  if (!frame) {
    return false;
  }
  if (ancestors.empty()) {
    Set(frame, {}, {});
    return true;
  }
  Set(frame, ancestors.front(), ancestors.back(), policy);
  bool ready = true;
  for (size_t i = 0; i < ancestors.size(); ++i) {
    auto* remote = Resolve(ancestors[i]);
    if (!remote) {
      ready = false;
      continue;
    }
    // Local ancestor windows already have their own authoritative frame
    // metadata.
    if (ancestors[i].Is<LocalFrameToken>()) {
      continue;
    }
    auto found = Map().find(remote);
    auto ancestor_policy =
        found != Map().end() ? found->value->policy : std::nullopt;
    Set(remote,
        i + 1 < ancestors.size() ? std::make_optional(ancestors[i + 1])
                                 : std::nullopt,
        i + 1 < ancestors.size() ? std::make_optional(ancestors.back())
                                 : std::nullopt,
        std::move(ancestor_policy));
  }
  RefreshPermissions(*frame);
  return ready;
}
}  // namespace blink
