// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_browser_support.h"

#include <atomic>
#include <utility>
#include <vector>

#include "content/public/browser/browser_context.h"
#include "tuffite/components/explorer/browser/content/explorer_isolation.h"
#include "tuffite/components/explorer/browser/explorer_policy.h"
#include "url/origin.h"

namespace tuffite {
namespace {
const char kProcessDomainsKey = 0;

int64_t NextDomainId() {
  static std::atomic<int64_t> next{kExplorerProcessDomainBase};
  return next.fetch_add(2, std::memory_order_relaxed);
}

class ProcessDomains final : public base::SupportsUserData::Data {
 public:
  struct Entry {
    url::Origin owner;
    ExplorerPolicy policy;
    int64_t id;
  };
  std::vector<Entry> entries;
};
}  // namespace

int64_t ResolveExplorerIsolationId(content::BrowserContext& context,
                                   const url::Origin& owner,
                                   const ExplorerPolicy& policy) {
  if (owner.opaque()) {
    // Opaque owners cannot establish a reusable security domain. Odd ids are
    // reserved for these internal non-reusable domains.
    return NextDomainId() + 1;
  }
  // Runtime handling and idle schedules do not grant authority. A reusable
  // process domain depends only on the owner and immutable capabilities.
  auto security_policy = policy;
  security_policy.settings = ExplorerPolicySettings();
  security_policy.navigation = ExplorerNavigationPolicy::kEmit;
  security_policy.dialogs = ExplorerDialogPolicy::kSuppress;
  auto* domains =
      static_cast<ProcessDomains*>(context.GetUserData(&kProcessDomainsKey));
  if (!domains) {
    auto data = std::make_unique<ProcessDomains>();
    domains = data.get();
    context.SetUserData(&kProcessDomainsKey, std::move(data));
  }
  for (const auto& entry : domains->entries) {
    if (entry.owner == owner && entry.policy == security_policy) {
      return entry.id;
    }
  }
  const int64_t id = NextDomainId();
  domains->entries.push_back({owner, std::move(security_policy), id});
  return id;
}

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
