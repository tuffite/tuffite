// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_PARTITION_SNAPSHOT_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_PARTITION_SNAPSHOT_H_

#include "base/functional/callback_forward.h"

namespace content {
class StoragePartition;
}

namespace tuffite {

// Takes a creation-time snapshot of the browser-visible storage that can be
// safely migrated through public Chromium interfaces. The destination remains
// an independent StoragePartition after the callback runs.
void SnapshotExplorerPartition(content::StoragePartition* source,
                               content::StoragePartition* destination,
                               base::OnceCallback<void(bool)> callback);

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_PARTITION_SNAPSHOT_H_
