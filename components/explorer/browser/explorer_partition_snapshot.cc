// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_partition_snapshot.h"

#include <algorithm>
#include <cstddef>
#include <optional>
#include <string_view>
#include <utility>
#include <vector>

#include "base/functional/bind.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/ref_counted.h"
#include "base/timer/timer.h"
#include "components/services/storage/public/mojom/local_storage_control.mojom.h"
#include "components/services/storage/public/mojom/storage_usage_info.mojom.h"
#include "content/public/browser/storage_partition.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "net/cookies/canonical_cookie.h"
#include "net/cookies/cookie_access_result.h"
#include "net/cookies/cookie_options.h"
#include "net/cookies/cookie_util.h"
#include "services/network/public/mojom/cookie_manager.mojom.h"
#include "third_party/blink/public/common/storage_key/storage_key.h"
#include "third_party/blink/public/mojom/dom_storage/storage_area.mojom.h"
#include "url/gurl.h"
#include "url/origin.h"

namespace tuffite {
namespace {

constexpr base::TimeDelta kSnapshotTimeout = base::Seconds(15);
constexpr char kSnapshotMarkerOrigin[] =
    "https://tuffite-explorer-partition-snapshot.invalid";

std::vector<uint8_t> EncodeLocalStorageString(std::string_view value) {
  // Blink's Latin-1 localStorage wire format is a one-byte format marker
  // followed by the bytes. The marker lives under an unreachable internal
  // origin and exists only to make an empty-source seed operation durable.
  std::vector<uint8_t> encoded(value.size() + 1, 0);
  encoded[0] = 1;
  std::ranges::copy(value, encoded.begin() + 1);
  return encoded;
}

class StorageAreaSnapshot final : public base::RefCounted<StorageAreaSnapshot> {
 public:
  StorageAreaSnapshot(storage::mojom::LocalStorageControl* source,
                      storage::mojom::LocalStorageControl* destination,
                      const blink::StorageKey& storage_key,
                      base::OnceCallback<void(bool)> callback)
      : callback_(std::move(callback)) {
    source->BindStorageArea(storage_key, source_.BindNewPipeAndPassReceiver());
    destination->BindStorageArea(storage_key,
                                 destination_.BindNewPipeAndPassReceiver());
  }

  void Start() {
    timeout_.Start(FROM_HERE, kSnapshotTimeout,
                   base::BindOnce(&StorageAreaSnapshot::Finish,
                                  base::RetainedRef(this), false));
    source_.set_disconnect_handler(base::BindOnce(
        &StorageAreaSnapshot::OnDisconnect, base::RetainedRef(this)));
    destination_.set_disconnect_handler(base::BindOnce(
        &StorageAreaSnapshot::OnDisconnect, base::RetainedRef(this)));
    source_->GetAll(
        mojo::NullRemote(),
        base::BindOnce(&StorageAreaSnapshot::OnRead, base::RetainedRef(this)));
  }

 private:
  friend class base::RefCounted<StorageAreaSnapshot>;
  ~StorageAreaSnapshot() = default;

  void OnRead(std::vector<blink::mojom::KeyValuePtr> entries) {
    if (finished_) {
      return;
    }
    if (entries.empty()) {
      Finish(true);
      return;
    }
    pending_writes_ = entries.size();
    for (const auto& entry : entries) {
      destination_->Put(entry->key, entry->value, std::nullopt, nullptr,
                        base::BindOnce(&StorageAreaSnapshot::OnWrite,
                                       base::RetainedRef(this)));
    }
  }

  void OnWrite(bool success) {
    if (finished_) {
      return;
    }
    succeeded_ &= success;
    if (--pending_writes_ == 0) {
      Finish(succeeded_);
    }
  }

  void Finish(bool success) {
    if (finished_) {
      return;
    }
    finished_ = true;
    timeout_.Stop();
    if (callback_) {
      std::move(callback_).Run(success);
    }
  }

  void OnDisconnect() { Finish(false); }

  mojo::Remote<blink::mojom::StorageArea> source_;
  mojo::Remote<blink::mojom::StorageArea> destination_;
  base::OnceCallback<void(bool)> callback_;
  base::OneShotTimer timeout_;
  size_t pending_writes_ = 0;
  bool succeeded_ = true;
  bool finished_ = false;
};

class PartitionSnapshot final : public base::RefCounted<PartitionSnapshot> {
 public:
  PartitionSnapshot(content::StoragePartition* source,
                    content::StoragePartition* destination,
                    base::OnceCallback<void(bool)> callback)
      : source_(source),
        destination_(destination),
        callback_(std::move(callback)) {}

  void Start() {
    timeout_.Start(FROM_HERE, kSnapshotTimeout,
                   base::BindOnce(&PartitionSnapshot::Finish,
                                  base::RetainedRef(this), false));
    destination_->GetCookieManagerForBrowserProcess()->GetAllCookies(
        base::BindOnce(&PartitionSnapshot::OnDestinationCookies,
                       base::RetainedRef(this)));
    destination_->GetLocalStorageControl()->GetUsage(
        base::BindOnce(&PartitionSnapshot::OnDestinationLocalStorageUsage,
                       base::RetainedRef(this)));
  }

 private:
  friend class base::RefCounted<PartitionSnapshot>;
  ~PartitionSnapshot() = default;

  void OnDestinationCookies(const net::CookieList& cookies) {
    if (finished_) {
      return;
    }
    destination_cookies_empty_ = cookies.empty();
    MaybeStartCopy();
  }

  void OnDestinationLocalStorageUsage(
      std::vector<storage::mojom::StorageUsageInfoPtr> usage) {
    if (finished_) {
      return;
    }
    destination_local_storage_empty_ = usage.empty();
    MaybeStartCopy();
  }

  void MaybeStartCopy() {
    if (finished_) {
      return;
    }
    if (!destination_cookies_empty_.has_value() ||
        !destination_local_storage_empty_.has_value()) {
      return;
    }

    // Snapshot inheritance is all-or-nothing across the storage types we can
    // migrate through public Chromium interfaces. If either destination
    // surface already contains state, preserve the entire destination rather
    // than producing a half-inherited partition.
    if (!*destination_cookies_empty_ || !*destination_local_storage_empty_) {
      Finish(true);
      return;
    }

    source_->GetCookieManagerForBrowserProcess()->GetAllCookies(
        base::BindOnce(&PartitionSnapshot::OnCookies, base::RetainedRef(this)));
    source_->GetLocalStorageControl()->GetUsage(base::BindOnce(
        &PartitionSnapshot::OnLocalStorageUsage, base::RetainedRef(this)));
  }

  void OnCookies(const net::CookieList& cookies) {
    if (finished_) {
      return;
    }
    if (cookies.empty()) {
      BranchDone(true);
      return;
    }
    pending_cookie_writes_ = cookies.size();
    network::mojom::CookieManager* manager =
        destination_->GetCookieManagerForBrowserProcess();
    for (const net::CanonicalCookie& cookie : cookies) {
      net::CookieOptions options;
      options.set_include_httponly();
      options.set_same_site_cookie_context(
          net::CookieOptions::SameSiteCookieContext::MakeInclusive());
      manager->SetCanonicalCookie(
          cookie, net::cookie_util::SimulatedCookieSource(cookie, "https"),
          options,
          base::BindOnce(&PartitionSnapshot::OnCookieWritten,
                         base::RetainedRef(this)));
    }
  }

  void OnCookieWritten(net::CookieAccessResult result) {
    if (finished_) {
      return;
    }
    cookies_succeeded_ &= result.status.IsInclude();
    if (--pending_cookie_writes_ == 0) {
      BranchDone(cookies_succeeded_);
    }
  }

  void OnLocalStorageUsage(
      std::vector<storage::mojom::StorageUsageInfoPtr> usage) {
    if (finished_) {
      return;
    }
    if (usage.empty()) {
      BranchDone(true);
      return;
    }
    pending_storage_areas_ = usage.size();
    for (const auto& info : usage) {
      auto transfer = base::MakeRefCounted<StorageAreaSnapshot>(
          source_->GetLocalStorageControl(),
          destination_->GetLocalStorageControl(), info->storage_key,
          base::BindOnce(&PartitionSnapshot::OnStorageAreaWritten,
                         base::RetainedRef(this)));
      transfer->Start();
    }
  }

  void OnStorageAreaWritten(bool success) {
    if (finished_) {
      return;
    }
    local_storage_succeeded_ &= success;
    if (--pending_storage_areas_ == 0) {
      BranchDone(local_storage_succeeded_);
    }
  }

  void BranchDone(bool success) {
    if (finished_) {
      return;
    }
    succeeded_ &= success;
    if (--pending_branches_ == 0) {
      if (!succeeded_) {
        Finish(false);
        return;
      }
      WriteMarker();
    }
  }

  void WriteMarker() {
    const blink::StorageKey marker_key = blink::StorageKey::CreateFirstParty(
        url::Origin::Create(GURL(kSnapshotMarkerOrigin)));
    destination_->GetLocalStorageControl()->BindStorageArea(
        marker_key, marker_area_.BindNewPipeAndPassReceiver());
    marker_area_.set_disconnect_handler(base::BindOnce(
        &PartitionSnapshot::Finish, base::RetainedRef(this), false));
    marker_area_->Put(EncodeLocalStorageString("snapshot"),
                      EncodeLocalStorageString("1"), std::nullopt, nullptr,
                      base::BindOnce(&PartitionSnapshot::OnMarkerWritten,
                                     base::RetainedRef(this)));
  }

  void OnMarkerWritten(bool success) { Finish(success); }

  void Finish(bool success) {
    if (finished_) {
      return;
    }
    finished_ = true;
    timeout_.Stop();
    if (callback_) {
      std::move(callback_).Run(success);
    }
  }

  const raw_ptr<content::StoragePartition> source_;
  const raw_ptr<content::StoragePartition> destination_;
  base::OnceCallback<void(bool)> callback_;
  std::optional<bool> destination_cookies_empty_;
  std::optional<bool> destination_local_storage_empty_;
  mojo::Remote<blink::mojom::StorageArea> marker_area_;
  base::OneShotTimer timeout_;
  size_t pending_cookie_writes_ = 0;
  size_t pending_storage_areas_ = 0;
  size_t pending_branches_ = 2;
  bool cookies_succeeded_ = true;
  bool local_storage_succeeded_ = true;
  bool succeeded_ = true;
  bool finished_ = false;
};

}  // namespace

void SnapshotExplorerPartition(content::StoragePartition* source,
                               content::StoragePartition* destination,
                               base::OnceCallback<void(bool)> callback) {
  if (!source || !destination || source == destination) {
    std::move(callback).Run(source && destination);
    return;
  }
  auto snapshot = base::MakeRefCounted<PartitionSnapshot>(source, destination,
                                                          std::move(callback));
  snapshot->Start();
}

}  // namespace tuffite
