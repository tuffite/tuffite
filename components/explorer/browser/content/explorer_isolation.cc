// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/content/explorer_isolation.h"

#include "content/browser/embedder_isolation_info.h"
#include "content/browser/site_instance_impl.h"
#include "content/browser/url_info.h"
#include "content/browser/web_contents/web_contents_impl.h"
#include "content/public/browser/storage_partition_config.h"
#include "url/gurl.h"

namespace tuffite {

std::unique_ptr<content::WebContents> DetachExplorerWebContents(
    content::WebContents* contents) {
  CHECK(GetExplorerIsolationId(contents));
  auto* inner = static_cast<content::WebContentsImpl*>(contents);
  auto* outer = inner->GetOuterWebContents();
  CHECK(outer);
  CHECK(!inner->IsGuest());
  auto owned = outer->node_.ReleaseExplorerOwnership(contents);
  outer->DetachUnownedInnerWebContents(contents, /*tuffite_transfer=*/true);
  owned->WasHidden();
  return owned;
}

scoped_refptr<content::SiteInstance> CreateExplorerSiteInstance(
    content::BrowserContext* browser_context,
    const GURL& url,
    const content::StoragePartitionConfig& partition_config,
    int64_t instance_id) {
  CHECK(browser_context);
  CHECK_GE(instance_id, 0);
  return content::SiteInstanceImpl::CreateForUrlInfo(
      browser_context,
      content::UrlInfo(
          content::UrlInfoInit(url)
              .WithStoragePartitionConfig(partition_config)
              .WithEmbedderIsolationInfo(
                  content::EmbedderIsolationInfo::CreateForUniqueInstance(
                      instance_id))),
      /*is_guest=*/false, /*is_fenced=*/false,
      // Chromium only needs the fixed-partition navigation contract for a
      // non-default config. Marking the default partition fixed is outside
      // the contract of SiteInstance::CreateForFixedStoragePartition and can
      // reject rapid Explorer recreation.
      /*is_fixed_storage_partition=*/!partition_config.is_default());
}

}  // namespace tuffite
