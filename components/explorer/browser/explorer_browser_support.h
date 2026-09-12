// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_BROWSER_SUPPORT_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_BROWSER_SUPPORT_H_

#include <cstdint>
#include <string>
#include <string_view>

#include "base/functional/callback.h"
#include "content/public/browser/javascript_dialog_manager.h"
#include "ui/base/window_open_disposition.h"

namespace tuffite {

// Maps the browser's final disposition to the stable Explorer event kind.
// This must run at CreateCustomWebContents/AddNewContents time; the earlier
// IsWebContentsCreationOverridden hook does not receive a disposition.
std::string_view ExplorerOpenEventType(WindowOpenDisposition disposition,
                                       bool is_window_open);
std::string_view ExplorerDispositionName(WindowOpenDisposition disposition);
uint64_t NextExplorerRequestId();

// Owns the one-dialog-at-a-time lifecycle independently from renderer JS.
// A new dialog cancels the previous one, and navigation/destruction cancel
// the active callback so no stale document can respond to it.
class ExplorerDialogState {
 public:
  using DialogClosedCallback =
      content::JavaScriptDialogManager::DialogClosedCallback;

  ExplorerDialogState();
  ~ExplorerDialogState();

  ExplorerDialogState(const ExplorerDialogState&) = delete;
  ExplorerDialogState& operator=(const ExplorerDialogState&) = delete;

  uint64_t Begin(DialogClosedCallback callback);
  bool Respond(uint64_t request_id,
               bool accept,
               const std::u16string& prompt_text);
  void Cancel();

  bool has_pending_dialog() const { return !!pending_callback_; }
  uint64_t pending_request_id() const { return pending_request_id_; }

 private:
  DialogClosedCallback pending_callback_;
  uint64_t pending_request_id_ = 0;
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_BROWSER_SUPPORT_H_
