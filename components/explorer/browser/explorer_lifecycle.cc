// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_lifecycle.h"

#include <algorithm>

#include "base/feature_list.h"
#include "base/functional/bind.h"
#include "base/memory_coordinator/memory_consumer_registry.h"
#include "base/task/single_thread_task_runner.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/devtools_agent_host.h"
#include "content/public/browser/download_item_utils.h"
#include "content/public/browser/download_manager.h"
#include "content/public/browser/navigation_controller.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/site_instance.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_capability_type.h"
#include "content/public/common/content_features.h"

namespace tuffite {
namespace {
constexpr auto kPollInterval = base::Seconds(5);
}  // namespace

WEB_CONTENTS_USER_DATA_KEY_IMPL(ExplorerLifecycle);

ExplorerLifecycle::ExplorerLifecycle(content::WebContents* contents)
    : WebContentsUserData(*contents), WebContentsObserver(contents) {
  if (base::MemoryConsumerRegistry::MaybeGet()) {
    using Traits = base::MemoryConsumerTraits;
    memory_registration_ = std::make_unique<base::MemoryConsumerRegistration>(
        "TuffiteExplorer",
        Traits(Traits::EstimatedMemoryUsage::kLarge,
               Traits::ReleaseMemoryCost::kRequiresTraversal,
               Traits::InformationRetention::kLossy,
               Traits::ExecutionType::kAsynchronous, Traits::InProcess::kNo,
               Traits::RecreateMemoryCost::kExpensive),
        this);
  }
  OnVisibilityChanged(contents->GetVisibility());
}

ExplorerLifecycle::~ExplorerLifecycle() = default;

void ExplorerLifecycle::SetPolicy(const ExplorerPolicySettings& policy) {
  if (freeze_ == policy.freeze && discard_ == policy.discard) {
    return;
  }
  Thaw();
  freeze_ = policy.freeze;
  discard_ = policy.discard;
  timer_.Stop();
  hidden_since_ = base::TimeTicks::Now();
  if (web_contents()) {
    OnVisibilityChanged(web_contents()->GetVisibility());
  }
  Evaluate();
}

bool ExplorerLifecycle::Wake() {
  if (discarding_ || discarded()) {
    return false;
  }
  Thaw();
  hidden_since_ = base::TimeTicks::Now();
  return true;
}

bool ExplorerLifecycle::discarded() const {
  return discarded_ && web_contents() &&
         web_contents()->GetController().NeedsReload();
}

bool ExplorerLifecycle::IsProtected() {
  auto* contents = web_contents();
  if (!contents || contents->GetVisibility() != content::Visibility::HIDDEN ||
      contents->IsLoading() ||
      contents->HasUncommittedNavigationInPrimaryMainFrame() ||
      dialog_pending_ || contents->IsCrashed() ||
      contents->IsCurrentlyAudible() || contents->IsBeingCaptured() ||
      !capturing_frames_.empty() || contents->HasPictureInPictureVideo() ||
      contents->HasPictureInPictureDocument() || contents->IsFullscreen() ||
      contents->HasFileSystemAccessHandles() ||
      !contents->GetInnerWebContents().empty() ||
      contents->GetSiteInstance()->GetRelatedActiveContentsCount() > 1) {
    return true;
  }
  using Capability = content::WebContentsCapabilityType;
  for (auto capability : {Capability::kUSB, Capability::kBluetoothConnected,
                          Capability::kBluetoothScanning, Capability::kHID,
                          Capability::kSerial, Capability::kGeolocation}) {
    if (contents->IsCapabilityActive(capability)) {
      return true;
    }
  }
  if (content::DevToolsAgentHost::HasFor(contents) &&
      content::DevToolsAgentHost::GetOrCreateFor(contents)->IsAttached()) {
    return true;
  }
  if (auto agent = content::DevToolsAgentHost::GetForTab(contents);
      agent && agent->IsAttached()) {
    return true;
  }
  content::DownloadManager::DownloadVector downloads;
  contents->GetBrowserContext()->GetDownloadManager()->GetAllDownloads(
      &downloads);
  for (const auto& item : downloads) {
    if (item->GetState() == download::DownloadItem::IN_PROGRESS &&
        content::DownloadItemUtils::GetWebContents(item.get()) == contents) {
      return true;
    }
  }
  return false;
}

void ExplorerLifecycle::SetDialogPending(bool pending) {
  dialog_pending_ = pending;
  Evaluate();
}

void ExplorerLifecycle::Thaw() {
  if (frozen_ && web_contents()) {
    // Showing a page automatically thaws it inside Content.
    if (web_contents()->GetVisibility() != content::Visibility::VISIBLE) {
      web_contents()->SetPageFrozen(false);
    }
    frozen_ = false;
  }
}

void ExplorerLifecycle::Evaluate() {
  if (!web_contents() || discarding_ || discarded()) {
    return;
  }
  if (IsProtected()) {
    Thaw();
    // Require a fresh idle interval after activity or a protection ends.
    hidden_since_ = base::TimeTicks::Now();
    return;
  }
  const auto idle = base::TimeTicks::Now() - hidden_since_;
  const auto discard = discard_.Resolve(kExplorerDefaultDiscardMs);
  if (discard && idle >= base::Milliseconds(*discard) && Discard()) {
    return;
  }
  const auto freeze = freeze_.Resolve(kExplorerDefaultFreezeMs);
  if (freeze && !frozen_ && idle >= base::Milliseconds(*freeze)) {
    web_contents()->SetPageFrozen(true);
    frozen_ = true;
  }
}

bool ExplorerLifecycle::Discard() {
  if (!attached_ || discarding_ || discarded() || IsProtected() ||
      !base::FeatureList::IsEnabled(features::kWebContentsDiscard)) {
    return false;
  }
  // A discard loses JS/form state. Event listeners alone must not prevent
  // freezing: pages use visibilitychange/freeze to cooperate with suspension.
  bool before_unload = false;
  web_contents()->ForEachRenderFrameHost([&](content::RenderFrameHost* frame) {
    before_unload |= frame->CouldDisplayBeforeUnloadDialog();
  });
  if (before_unload) {
    return false;
  }
  Thaw();
  discarding_ = true;
  web_contents()->Discard(base::BindOnce(&ExplorerLifecycle::OnDiscardComplete,
                                         weak_factory_.GetWeakPtr()));
  discarded_ = web_contents()->GetController().NeedsReload();
  if (!discarded_) {
    discarding_ = false;
    return false;
  }
  return true;
}

void ExplorerLifecycle::OnDiscardComplete() {
  // Content can invoke the callback synchronously before SetNeedsReload().
  base::SingleThreadTaskRunner::GetCurrentDefault()->PostTask(
      FROM_HERE, base::BindOnce(
                     [](base::WeakPtr<ExplorerLifecycle> self) {
                       if (self) {
                         self->discarding_ = false;
                         self->RestoreIfVisible();
                       }
                     },
                     weak_factory_.GetWeakPtr()));
}

void ExplorerLifecycle::RestoreIfVisible() {
  if (web_contents() && !discarding_ && discarded() &&
      web_contents()->GetVisibility() == content::Visibility::VISIBLE) {
    web_contents()->GetController().LoadIfNecessary();
  }
}

void ExplorerLifecycle::OnVisibilityChanged(content::Visibility visibility) {
  if (visibility != content::Visibility::HIDDEN) {
    timer_.Stop();
    hidden_since_ = base::TimeTicks();
    Thaw();
    RestoreIfVisible();
    return;
  }
  const auto freeze = freeze_.Resolve(kExplorerDefaultFreezeMs);
  const auto discard = discard_.Resolve(kExplorerDefaultDiscardMs);
  if (!freeze && !discard) {
    timer_.Stop();
    return;
  }
  if (!timer_.IsRunning()) {
    hidden_since_ = base::TimeTicks::Now();
    auto interval = kPollInterval;
    for (const auto delay : {freeze, discard}) {
      if (delay) {
        interval = std::min(interval, std::max(base::Milliseconds(100),
                                               base::Milliseconds(*delay)));
      }
    }
    timer_.Start(FROM_HERE, interval, this, &ExplorerLifecycle::Evaluate);
  }
}

void ExplorerLifecycle::OnAudioStateChanged(bool audible) {
  Evaluate();
}

void ExplorerLifecycle::OnFrameIsCapturingMediaStreamChanged(
    content::RenderFrameHost* frame,
    bool capturing) {
  if (capturing) {
    capturing_frames_.insert(frame->GetGlobalId());
  } else {
    capturing_frames_.erase(frame->GetGlobalId());
  }
  Evaluate();
}

void ExplorerLifecycle::RenderFrameDeleted(content::RenderFrameHost* frame) {
  capturing_frames_.erase(frame->GetGlobalId());
}

void ExplorerLifecycle::DidStartNavigation(content::NavigationHandle*) {
  Thaw();
  hidden_since_ = base::TimeTicks::Now();
}

void ExplorerLifecycle::DidFinishNavigation(content::NavigationHandle* handle) {
  if (handle->IsInPrimaryMainFrame() && handle->HasCommitted() &&
      !handle->IsSameDocument()) {
    frozen_ = false;
    discarding_ = false;
    discarded_ = false;
    hidden_since_ = base::TimeTicks::Now();
  }
}

void ExplorerLifecycle::PrimaryMainFrameRenderProcessGone(
    base::TerminationStatus) {
  discarding_ = false;
  frozen_ = false;
  capturing_frames_.clear();
  RestoreIfVisible();
}

void ExplorerLifecycle::WebContentsDestroyed() {
  timer_.Stop();
  memory_registration_.reset();
  weak_factory_.InvalidateWeakPtrs();
}

void ExplorerLifecycle::OnReleaseMemory() {
  const auto discard = discard_.Resolve(kExplorerDefaultDiscardMs);
  const auto minimum = discard_.mode == ExplorerDelay::Mode::kDefault
                           ? kExplorerDefaultFreezeMs
                           : discard.value_or(0);
  if (memory_limit() == 0 && discard && !hidden_since_.is_null() &&
      base::TimeTicks::Now() - hidden_since_ >= base::Milliseconds(minimum)) {
    Discard();
  }
}
}  // namespace tuffite
