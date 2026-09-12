// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_WEB_CONTENTS_ROLE_H_
#define TUFFITE_FRAMEWORK_BROWSER_WEB_CONTENTS_ROLE_H_

#include <optional>
#include <string>
#include <vector>

#include "content/public/browser/web_contents_user_data.h"
#include "tuffite/components/explorer/browser/explorer_policy.h"

namespace content {
class WebContents;
}

namespace tuffite {

// Immutable capability tier that Tuffite Framework assigns to every
// WebContents it creates. The value is stored in browser-process
// WebContentsUserData and MUST be treated as authoritative when authorizing
// privileged bindings such as ShellAPI. Renderer-side role hints are
// convenience only and must NEVER be used as a security boundary.
//
// - `kApp` is the top-level trusted surface for the Tuffite application.
//   Its documents (from allowlisted origins in the manifest) are the only
//   ones eligible to receive ShellAPI. This is set exactly once, when the
//   Framework creates the primary Shell.
//
// - `kExplorer` is the sandbox for `<explorer>` inner WebContents. Even if
//   an Explorer document shares an origin with the App, it must be denied
//   ShellAPI, window, utility, and network capabilities. This is set exactly
//   once, when ExplorerHost creates the inner WebContents.
//
// Any WebContents WITHOUT an attached role must be treated as an untrusted
// third-party surface. This matches Chromium's default-deny model.
enum class WebContentsRole {
  kApp = 0,
  kExplorer = 1,
};

// The effective, browser-authorized Explorer policy. It is copied from the
// selected manifest capability when the WebContents is created and remains
// immutable for that WebContents' lifetime.
// Browser-owned role tag. Values must not be mutated after the initial
// SetRole() call for a given WebContents. Callers verify by DCHECK.
class WebContentsRoleData final
    : public content::WebContentsUserData<WebContentsRoleData> {
 public:
  ~WebContentsRoleData() override;

  // Assigns |role| to |web_contents|. Fails a CHECK if a different role is
  // already attached; re-assigning the same role is a no-op. Renderer-side
  // callers must never invoke this: it is browser-process only.
  static void SetRole(content::WebContents* web_contents, WebContentsRole role);

  // Returns the role attached in the browser process, or nullopt if none was
  // recorded. Callers MUST treat nullopt as "no privileged capabilities".
  static std::optional<WebContentsRole> GetRole(
      const content::WebContents* web_contents);

  static void SetExplorerPolicy(content::WebContents* web_contents,
                                ExplorerPolicy policy);
  static const ExplorerPolicy* GetExplorerPolicy(
      const content::WebContents* web_contents);

  // Controls browser-owned context menus for an Explorer surface. This is a
  // per-WebContents capability, not a renderer-side trust decision. App
  // surfaces use a separate explicit development-mode check in the view
  // delegate.
  static void SetExplorerContextMenuEnabled(content::WebContents* web_contents,
                                            bool enabled);
  static bool IsExplorerContextMenuEnabled(
      const content::WebContents* web_contents);

  WebContentsRole role() const { return role_; }

 private:
  friend class content::WebContentsUserData<WebContentsRoleData>;
  WebContentsRoleData(content::WebContents* web_contents, WebContentsRole role);

  WebContentsRole role_;
  WEB_CONTENTS_USER_DATA_KEY_DECL();
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_WEB_CONTENTS_ROLE_H_
