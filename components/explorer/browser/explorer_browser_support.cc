// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_browser_support.h"

#include <atomic>
#include <utility>

namespace tuffite {

std::string_view ExplorerOpenEventType(WindowOpenDisposition disposition,
                                       bool is_window_open) {
  switch (disposition) {
    case WindowOpenDisposition::CURRENT_TAB:
      return "navigation";
    default:
      return "newwindow";
  }
}

uint64_t NextExplorerRequestId() {
  static std::atomic<uint64_t> next_id{1};
  return next_id.fetch_add(1, std::memory_order_relaxed);
}

std::string_view ExplorerDispositionName(WindowOpenDisposition disposition) {
  switch (disposition) {
    case WindowOpenDisposition::CURRENT_TAB:
      return "current-tab";
    case WindowOpenDisposition::NEW_FOREGROUND_TAB:
      return "new-foreground-tab";
    case WindowOpenDisposition::NEW_BACKGROUND_TAB:
      return "new-background-tab";
    case WindowOpenDisposition::NEW_POPUP:
      return "new-popup";
    case WindowOpenDisposition::NEW_WINDOW:
      return "new-window";
    default:
      return "other";
  }
}

ExplorerDialogState::ExplorerDialogState() = default;

ExplorerDialogState::~ExplorerDialogState() {
  Cancel();
}

uint64_t ExplorerDialogState::Begin(DialogClosedCallback callback) {
  Cancel();
  pending_callback_ = std::move(callback);
  pending_request_id_ = NextExplorerRequestId();
  return pending_request_id_;
}

bool ExplorerDialogState::Respond(uint64_t request_id,
                                  bool accept,
                                  const std::u16string& prompt_text) {
  if (!pending_callback_ || request_id != pending_request_id_) {
    return false;
  }
  std::move(pending_callback_).Run(accept, prompt_text);
  pending_request_id_ = 0;
  return true;
}

void ExplorerDialogState::Cancel() {
  if (pending_callback_) {
    std::move(pending_callback_).Run(false, std::u16string());
  }
  pending_request_id_ = 0;
}

}  // namespace tuffite
