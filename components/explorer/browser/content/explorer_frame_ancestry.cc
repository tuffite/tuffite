// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/browser/content/explorer_frame_ancestry.h"

#include <set>

#include "base/supports_user_data.h"
#include "content/browser/renderer_host/browsing_context_state.h"
#include "content/browser/renderer_host/frame_tree.h"
#include "content/browser/renderer_host/frame_tree_node.h"
#include "content/browser/renderer_host/render_frame_host_impl.h"
#include "content/browser/renderer_host/render_frame_host_manager.h"
#include "content/browser/renderer_host/render_frame_proxy_host.h"
#include "content/browser/renderer_host/render_view_host_impl.h"
#include "content/browser/site_instance_impl.h"
#include "content/public/browser/web_contents.h"
#include "ipc/constants.mojom.h"
#include "tuffite/components/explorer/browser/content/explorer_isolation.h"

namespace tuffite {
namespace {
const char kOwnerKey = 0;
struct OwnerData final : base::SupportsUserData::Data {
  OwnerData();
  ~OwnerData() override;
  content::WeakDocumentPtr document;
  bool allow_messages = false;
  bool isolated_browsing_context = false;
  blink::FramePolicy frame_policy;
};
OwnerData::OwnerData() = default;
OwnerData::~OwnerData() = default;
content::RenderFrameHostImpl* Owner(content::RenderFrameHostImpl* frame) {
  if (auto* parent = frame->GetParent()) {
    return parent;
  }
  auto* contents = content::WebContents::FromRenderFrameHost(frame);
  auto* data = static_cast<OwnerData*>(contents->GetUserData(&kOwnerKey));
  auto* owner = data ? data->document.AsRenderFrameHostIfValid() : nullptr;
  return owner && owner->IsActive()
             ? static_cast<content::RenderFrameHostImpl*>(owner)
             : nullptr;
}
content::RenderFrameHostImpl* Top(content::RenderFrameHostImpl* frame) {
  std::set<content::RenderFrameHostImpl*> seen;
  while (seen.insert(frame).second) {
    auto* parent = Owner(frame);
    if (!parent) {
      return frame;
    }
    frame = parent;
  }
  return nullptr;
}
bool Allows(content::RenderFrameHostImpl* frame) {
  auto* contents = content::WebContents::FromRenderFrameHost(frame);
  auto* data = static_cast<OwnerData*>(contents->GetUserData(&kOwnerKey));
  return !GetExplorerIsolationId(contents) ||
         (data && data->allow_messages &&
          data->document.AsRenderFrameHostIfValid());
}
content::RenderFrameProxyHost* Proxy(content::RenderFrameHostImpl& frame,
                                     content::SiteInstanceGroup* group) {
  using State = content::BrowsingContextState;
  auto* node = frame.frame_tree_node();
  auto state = frame.browsing_context_state();
  auto* proxy = state->GetRenderFrameProxyHost(
      group, State::ProxyAccessMode::kAllowOuterDelegate);
  if (!proxy) {
    auto view = node->frame_tree().GetRenderViewHost(group);
    if (!view && node->IsMainFrame()) {
      view = node->frame_tree().CreateRenderViewHost(
          group, IPC::mojom::kRoutingIdNone, false,
          features::GetBrowsingContextMode() ==
                  features::BrowsingContextStateImplementationType::
                      kSwapForCrossBrowsingInstanceNavigations
              ? state
              : nullptr,
          content::CreateRenderViewHostCase::kDefault, std::nullopt);
    }
    if (!view && !node->IsMainFrame()) {
      // An inner main-frame proxy lives in the embedder's Page and has no
      // RenderViewHost of its own. Its ordinary iframe descendants must use
      // that native embedding view when replying through event.source.
      for (auto* ancestor = Owner(&frame); ancestor && !view;
           ancestor = Owner(ancestor)) {
        view =
            ancestor->frame_tree_node()->frame_tree().GetRenderViewHost(group);
      }
    }
    if (!view) {
      return nullptr;
    }
    // Creating a view initializes its composed ancestry and can re-enter this
    // helper. Reuse any proxy created by that initialization.
    proxy = state->GetRenderFrameProxyHost(
        group, State::ProxyAccessMode::kAllowOuterDelegate);
    if (!proxy) {
      proxy = state->CreateRenderFrameProxyHost(
          group, std::move(view), node,
          State::ProxyAccessMode::kAllowOuterDelegate);
    }
  }
  if (!proxy->is_render_frame_proxy_live()) {
    if (node->IsMainFrame()) {
      node->render_manager()->InitRenderView(group, proxy->GetRenderViewHost(),
                                             proxy, std::nullopt);
    } else {
      proxy->InitRenderFrameProxy(std::nullopt);
    }
  }
  return proxy;
}
}  // namespace
void SetExplorerOwner(content::WebContents* contents,
                      content::RenderFrameHost* owner,
                      bool allow_messages,
                      const blink::FramePolicy& frame_policy,
                      bool isolated_browsing_context) {
  auto data = std::make_unique<OwnerData>();
  if (owner) {
    data->document = owner->GetWeakDocumentPtr();
  }
  data->allow_messages = allow_messages;
  data->isolated_browsing_context = isolated_browsing_context;
  data->frame_policy = frame_policy;
  contents->SetUserData(&kOwnerKey, std::move(data));
}
std::optional<blink::FramePolicy> ExplorerFrameOwnerPolicy(
    content::RenderFrameHost& frame) {
  if (frame.GetParent()) {
    return std::nullopt;
  }
  auto* contents = content::WebContents::FromRenderFrameHost(&frame);
  auto* data = static_cast<OwnerData*>(contents->GetUserData(&kOwnerKey));
  return data && data->document.AsRenderFrameHostIfValid()
             ? std::make_optional(data->frame_policy)
             : std::nullopt;
}
void SetExplorerFramePolicy(content::WebContents* contents,
                            const blink::FramePolicy& policy) {
  if (auto* data = static_cast<OwnerData*>(contents->GetUserData(&kOwnerKey))) {
    data->frame_policy = policy;
  }
}
std::vector<blink::FrameToken> ExplorerFrameAncestors(
    content::RenderFrameHost& frame,
    content::SiteInstanceGroup* group) {
  auto* current = static_cast<content::RenderFrameHostImpl*>(&frame);
  if (!group) {
    group = current->GetSiteInstance()->group();
  }
  std::vector<content::RenderFrameHostImpl*> chain;
  std::set<content::RenderFrameHostImpl*> seen;
  for (auto* owner = Owner(current); owner && seen.insert(owner).second;
       owner = Owner(owner)) {
    chain.push_back(owner);
  }
  std::vector<blink::FrameToken> result(chain.size());
  // Root views must exist before their descendant proxies are created. Keep
  // all references browser-authenticated and backed by Chromium proxy hosts.
  for (size_t i = chain.size(); i > 0; --i) {
    if (chain[i - 1]->GetSiteInstance()->group() == group) {
      result[i - 1] = chain[i - 1]->GetFrameToken();
      continue;
    }
    auto* proxy = Proxy(*chain[i - 1], group);
    if (!proxy) {
      return {};
    }
    result[i - 1] = proxy->GetFrameToken();
  }
  return result;
}
std::vector<bool> ExplorerFrameIsolationBoundaries(
    content::RenderFrameHost& frame) {
  std::vector<bool> result;
  std::set<content::RenderFrameHostImpl*> seen;
  for (auto* current = static_cast<content::RenderFrameHostImpl*>(&frame);
       current && seen.insert(current).second; current = Owner(current)) {
    auto* contents = content::WebContents::FromRenderFrameHost(current);
    auto* data = static_cast<OwnerData*>(contents->GetUserData(&kOwnerKey));
    result.push_back(!current->GetParent() && data &&
                     data->isolated_browsing_context);
  }
  return result;
}
std::vector<std::optional<blink::FramePolicy>> ExplorerFrameOwnerPolicies(
    content::RenderFrameHost& frame) {
  std::vector<std::optional<blink::FramePolicy>> result;
  std::set<content::RenderFrameHostImpl*> seen;
  for (auto* current = static_cast<content::RenderFrameHostImpl*>(&frame);
       current && seen.insert(current).second; current = Owner(current)) {
    result.push_back(ExplorerFrameOwnerPolicy(*current));
  }
  return result;
}
bool ExplorerWindowMessageAllowed(content::RenderFrameHost* source,
                                  content::RenderFrameHost* target) {
  if (!source || !target || !source->IsActive() || !target->IsActive() ||
      source->GetBrowserContext() != target->GetBrowserContext()) {
    return false;
  }
  auto* from = static_cast<content::RenderFrameHostImpl*>(source);
  auto* to = static_cast<content::RenderFrameHostImpl*>(target);
  if (!GetExplorerIsolationId(
          content::WebContents::FromRenderFrameHost(source)) &&
      !GetExplorerIsolationId(
          content::WebContents::FromRenderFrameHost(target))) {
    return false;
  }
  auto* root = Top(from);
  return root && Allows(from) && Allows(to) && root == Top(to);
}
bool EnsureExplorerMessageSourceProxy(content::RenderFrameHost& source,
                                      content::SiteInstanceGroup* group) {
  // Build only this sender and its composed ancestor chain. Do not expose
  // unrelated opener trees across the Explorer isolation boundary.
  auto& frame = static_cast<content::RenderFrameHostImpl&>(source);
  // Ordinary messages inside an Explorer's own frame tree already have native
  // source references. A live local frame must never also have a proxy in its
  // own SiteInstanceGroup: navigation creates that replacement during unload.
  auto view = frame.frame_tree_node()->frame_tree().GetRenderViewHost(group);
  if (frame.GetSiteInstance()->group() == group ||
      (view && view->GetMainRenderFrameHost() &&
       &view->GetMainRenderFrameHost()->frame_tree_node()->frame_tree() ==
           &frame.frame_tree_node()->frame_tree())) {
    return true;
  }
  ExplorerFrameAncestors(source, group);
  return Proxy(frame, group) != nullptr;
}
bool ExplorerAttachWouldCycle(content::WebContents* contents,
                              content::RenderFrameHost& owner) {
  std::set<content::RenderFrameHostImpl*> seen;
  auto* frame = static_cast<content::RenderFrameHostImpl*>(&owner);
  while (frame && seen.insert(frame).second) {
    if (content::WebContents::FromRenderFrameHost(frame) == contents) {
      return true;
    }
    frame = Owner(frame);
  }
  return frame != nullptr;
}
}  // namespace tuffite
