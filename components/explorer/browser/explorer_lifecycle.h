// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_LIFECYCLE_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_LIFECYCLE_H_

#include <memory>
#include <set>
#include <string_view>

#include "base/memory/weak_ptr.h"
#include "base/memory_coordinator/memory_consumer.h"
#include "base/time/time.h"
#include "base/timer/timer.h"
#include "content/public/browser/global_routing_id.h"
#include "content/public/browser/web_contents_observer.h"
#include "content/public/browser/web_contents_user_data.h"
#include "tuffite/components/explorer/common/explorer_policy.h"

namespace tuffite {

// Owned by the page, so detach/transfer does not abandon its background policy.
class ExplorerLifecycle final
    : public content::WebContentsUserData<ExplorerLifecycle>,
      public content::WebContentsObserver,
      public base::MemoryConsumer {
 public:
  ~ExplorerLifecycle() override;
  void SetPolicy(const ExplorerPolicySettings& policy);
  // Detached handles promise to preserve their live document until adoption.
  void SetAttached(bool attached) { attached_ = attached; }
  bool Wake();
  bool Discard();
  bool frozen() const { return frozen_; }
  bool discarded() const;
  void SetDialogPending(bool pending);

 private:
  friend class content::WebContentsUserData<ExplorerLifecycle>;
  explicit ExplorerLifecycle(content::WebContents* contents);
  void OnVisibilityChanged(content::Visibility visibility) override;
  void OnAudioStateChanged(bool audible) override;
  void OnFrameIsCapturingMediaStreamChanged(content::RenderFrameHost*,
                                            bool) override;
  void RenderFrameDeleted(content::RenderFrameHost*) override;
  void DidStartNavigation(content::NavigationHandle*) override;
  void DidFinishNavigation(content::NavigationHandle*) override;
  void WebContentsDestroyed() override;
  void PrimaryMainFrameRenderProcessGone(base::TerminationStatus) override;
  void OnUpdateMemoryLimit() override {}
  void OnReleaseMemory() override;
  bool IsProtected();
  void Evaluate();
  void Thaw();
  void OnDiscardComplete();
  void RestoreIfVisible();

  ExplorerDelay freeze_;
  ExplorerDelay discard_;
  base::TimeTicks hidden_since_;
  bool attached_ = false;
  bool frozen_ = false;
  bool discarding_ = false;
  bool discarded_ = false;
  bool dialog_pending_ = false;
  std::set<content::GlobalRenderFrameHostId> capturing_frames_;
  base::RepeatingTimer timer_;
  std::unique_ptr<base::MemoryConsumerRegistration> memory_registration_;
  base::WeakPtrFactory<ExplorerLifecycle> weak_factory_{this};
  WEB_CONTENTS_USER_DATA_KEY_DECL();
};

}  // namespace tuffite
#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_LIFECYCLE_H_
