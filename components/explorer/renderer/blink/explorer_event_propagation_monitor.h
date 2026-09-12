// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_EVENT_PROPAGATION_MONITOR_H_
#define TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_EVENT_PROPAGATION_MONITOR_H_

#include "base/memory/raw_ptr_exclusion.h"

namespace blink {

class Event;

// Remembers whether a DOM event stopped propagation before Blink clears that
// transient state at the end of dispatch. Explorer uses this to bridge the
// propagation decision across nested WebContents.
class TuffiteEventPropagationMonitor final {
 public:
  explicit TuffiteEventPropagationMonitor(const Event& event);
  ~TuffiteEventPropagationMonitor();

  TuffiteEventPropagationMonitor(const TuffiteEventPropagationMonitor&) =
      delete;
  TuffiteEventPropagationMonitor& operator=(
      const TuffiteEventPropagationMonitor&) = delete;

  bool WasStopped() const { return was_stopped_; }

 private:
  friend void TuffiteRecordEventPropagationState(const Event& event);

  // Oilpan-managed Event address used only for identity, never dereferenced.
  RAW_PTR_EXCLUSION const void* event_identity_;
  // Nested synchronous dispatch links stack-owned RAII monitors, not PA
  // objects.
  RAW_PTR_EXCLUSION TuffiteEventPropagationMonitor* previous_ = nullptr;
  bool was_stopped_ = false;
};

void TuffiteRecordEventPropagationState(const Event& event);

}  // namespace blink

#endif  // TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_EXPLORER_EVENT_PROPAGATION_MONITOR_H_
