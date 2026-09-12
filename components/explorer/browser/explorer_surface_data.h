// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_SURFACE_DATA_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_SURFACE_DATA_H_
#include <cstdint>
#include <optional>

#include "content/public/browser/web_contents_user_data.h"
#include "tuffite/components/explorer/browser/explorer_policy.h"
namespace tuffite {
class ExplorerSurfaceData final
    : public content::WebContentsUserData<ExplorerSurfaceData> {
 public:
  ~ExplorerSurfaceData() override;
  static void SetNavigationDownload(content::WebContents*,
                                    std::optional<uint32_t>);
  static std::optional<uint32_t> NavigationDownload(
      const content::WebContents*);
  static void SetExplorerPolicy(content::WebContents*, ExplorerPolicy);
  static const ExplorerPolicy* GetExplorerPolicy(const content::WebContents*);
  static void SetExplorerContextMenuEnabled(content::WebContents*, bool);
  static bool IsExplorerContextMenuEnabled(const content::WebContents*);

 private:
  friend class content::WebContentsUserData<ExplorerSurfaceData>;
  explicit ExplorerSurfaceData(content::WebContents* contents);
  std::optional<ExplorerPolicy> policy_;
  bool menu_enabled_ = false;
  std::optional<uint32_t> navigation_download_;
  WEB_CONTENTS_USER_DATA_KEY_DECL();
};
}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_SURFACE_DATA_H_
