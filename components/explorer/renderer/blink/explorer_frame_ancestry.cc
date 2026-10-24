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
  bool isolated = false;
  bool has_boundaries = false;
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
         std::optional<FramePolicy> policy = std::nullopt,
         bool isolated = false,
         bool has_boundaries = false) {
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
  if (!parent && !top && !policy) {
    Map().erase(frame);
    RefreshPermissions(*frame);
    return;
  }
  auto* value = MakeGarbageCollected<Ancestors>();
  value->parent = parent;
  value->top = top;
  value->policy = std::move(policy);
  value->isolated = isolated;
  value->has_boundaries = has_boundaries;
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
  auto found = Map().find(&frame);
  return frame.Parent() || (found != Map().end() && found->value->isolated)
             ? nullptr
             : Lookup(frame, false);
}
Frame* ExplorerTopFrame(Frame& frame) {
  // Remote inner-main-frame proxies may live in their embedder's native Page
  // tree. Stop before traversing that physical parent as well.
  auto* root = &frame;
  while (!ExplorerIsolatedFrame(*root) && root->Parent()) {
    root = root->Parent();
  }
  return ExplorerIsolatedFrame(*root) ? root : Lookup(*root, true);
}
bool ExplorerIsolatedFrame(Frame& frame) {
  auto found = Map().find(&frame);
  return found != Map().end() && found->value->isolated;
}
Frame* ExplorerPermissionsPolicyParent(Frame& frame) {
  // DOM ancestry may hide a real native parent on an inner-main-frame proxy.
  // Permissions inheritance must retain that parent too.
  if (auto* owner = Lookup(frame, false)) {
    return owner;
  }
  return frame.Parent();
}
bool SetExplorerFrameAncestors(
    WebFrame* web_frame,
    const std::vector<FrameToken>& ancestors,
    const std::vector<std::optional<FramePolicy>>& policies,
    const std::vector<bool>& isolation_boundaries) {
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
  if (!isolation_boundaries.empty() &&
      isolation_boundaries.size() != ancestors.size() + 1) {
    return false;
  }
  if (!policies.empty() && policies.size() != ancestors.size() + 1) {
    return false;
  }
  const bool has_boundaries = !isolation_boundaries.empty();
  auto previous = Map().find(frame);
  if (!has_boundaries && previous != Map().end() &&
      previous->value->has_boundaries) {
    return true;
  }
  const auto policy = [&](size_t index) -> std::optional<FramePolicy> {
    return policies.empty() ? std::nullopt : policies[index];
  };
  const auto isolated = [&](size_t index) {
    return !isolation_boundaries.empty() && isolation_boundaries[index];
  };
  const auto top = [&](size_t start) -> std::optional<FrameToken> {
    if (start >= ancestors.size()) {
      return std::nullopt;
    }
    for (size_t i = start; i < ancestors.size(); ++i) {
      if (isolated(i + 1)) {
        return ancestors[i];
      }
    }
    return ancestors.back();
  };
  if (ancestors.empty()) {
    Set(frame, {}, {}, policy(0), isolated(0), has_boundaries);
    return true;
  }
  Set(frame, ancestors.front(), top(0), policy(0), isolated(0), has_boundaries);
  bool ready = true;
  // Initialize permissions from outermost owner to child.
  for (size_t i = ancestors.size(); i-- > 0;) {
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
    auto previous_ancestor = Map().find(remote);
    if (!has_boundaries && previous_ancestor != Map().end() &&
        previous_ancestor->value->has_boundaries) {
      continue;
    }
    auto ancestor_policy = policy(i + 1);
    Set(remote,
        i + 1 < ancestors.size() ? std::make_optional(ancestors[i + 1])
                                 : std::nullopt,
        top(i + 1), std::move(ancestor_policy), isolated(i + 1),
        has_boundaries);
  }
  RefreshPermissions(*frame);
  return ready;
}
bool InitializeExplorerFrameAncestors(
    WebFrame* web_frame,
    const std::vector<FrameToken>& ancestors,
    const std::optional<FramePolicy>& policy) {
  std::vector<std::optional<FramePolicy>> policies(ancestors.size() + 1);
  policies.front() = policy;
  for (size_t i = 0; i < ancestors.size(); ++i) {
    auto* remote = Resolve(ancestors[i]);
    auto found = remote ? Map().find(remote) : Map().end();
    if (found != Map().end()) {
      policies[i + 1] = found->value->policy;
    }
  }
  return SetExplorerFrameAncestors(web_frame, ancestors, policies);
}
}  // namespace blink
