// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/renderer/blink/explorer_event_propagation_monitor.h"

#include "base/check.h"
#include "third_party/blink/renderer/core/dom/events/event.h"

namespace blink {
namespace {

thread_local TuffiteEventPropagationMonitor* g_active_monitor = nullptr;

}  // namespace

TuffiteEventPropagationMonitor::TuffiteEventPropagationMonitor(
    const Event& event)
    : event_identity_(&event), previous_(g_active_monitor) {
  g_active_monitor = this;
}

TuffiteEventPropagationMonitor::~TuffiteEventPropagationMonitor() {
  CHECK_EQ(g_active_monitor, this);
  g_active_monitor = previous_;
}

void TuffiteRecordEventPropagationState(const Event& event) {
  if (!event.PropagationStopped()) {
    return;
  }
  for (auto* monitor = g_active_monitor; monitor;
       monitor = monitor->previous_) {
    if (monitor->event_identity_ == &event) {
      monitor->was_stopped_ = true;
    }
  }
}

}  // namespace blink
