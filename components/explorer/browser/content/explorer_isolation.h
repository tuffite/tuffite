// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_CONTENT_EXPLORER_ISOLATION_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_CONTENT_EXPLORER_ISOLATION_H_

#include <cstdint>
#include <memory>
#include <optional>

#include "base/check_op.h"
#include "base/memory/scoped_refptr.h"
#include "base/supports_user_data.h"
#include "content/common/content_export.h"

class GURL;

namespace content {
class BrowserContext;
class SiteInstance;
class StoragePartitionConfig;
class WebContents;
}  // namespace content

namespace tuffite {

// Detaches only Tuffite-owned, non-Guest inner contents, preserving the page.
std::unique_ptr<content::WebContents> DetachExplorerWebContents(
    content::WebContents* contents);

namespace internal {

inline constexpr char kExplorerIsolationDataKey = 0;

class ExplorerIsolationData final : public base::SupportsUserData::Data {
 public:
  explicit ExplorerIsolationData(int64_t instance_id)
      : instance_id(instance_id) {}

  const int64_t instance_id;
};

}  // namespace internal

inline void SetExplorerIsolationId(base::SupportsUserData* contents,
                                   int64_t instance_id) {
  CHECK_GE(instance_id, 0);
  contents->SetUserData(
      &internal::kExplorerIsolationDataKey,
      std::make_unique<internal::ExplorerIsolationData>(instance_id));
}

inline std::optional<int64_t> GetExplorerIsolationId(
    base::SupportsUserData* contents) {
  if (!contents) {
    return std::nullopt;
  }
  auto* data = static_cast<internal::ExplorerIsolationData*>(
      contents->GetUserData(&internal::kExplorerIsolationDataKey));
  return data ? std::make_optional(data->instance_id) : std::nullopt;
}

// Creates the initial SiteInstance for an Explorer with both of its immutable
// boundaries in place before the first navigation: a fixed StoragePartition
// and a stable per-Explorer process-isolation id. Installing either boundary
// after WebContents creation allows the first navigation to replace the
// BrowsingInstance and lose the other one.
CONTENT_EXPORT scoped_refptr<content::SiteInstance> CreateExplorerSiteInstance(
    content::BrowserContext* browser_context,
    const GURL& url,
    const content::StoragePartitionConfig& partition_config,
    int64_t instance_id);

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_CONTENT_EXPLORER_ISOLATION_H_
