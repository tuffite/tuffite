// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_POLICY_H_
#define TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_POLICY_H_
#include <string>
#include <vector>
namespace tuffite {
enum class ExplorerNavigationPolicy {
  kEmit,
  kAllow,
};

enum class ExplorerDialogPolicy {
  kEmit,
  kSuppress,
};

struct ExplorerPolicy {
  bool operator==(const ExplorerPolicy&) const = default;
  std::string partition;
  ExplorerNavigationPolicy navigation = ExplorerNavigationPolicy::kEmit;
  ExplorerDialogPolicy dialogs = ExplorerDialogPolicy::kSuppress;
  std::vector<std::string> execute_javascript_origins;
  bool allow_execute_javascript = false;
  bool allow_execute_javascript_main_world = false;
  bool allow_dialogs = false;
  bool allow_parent_messages = false;
  bool allow_top_messages = false;
  bool allow_shell_api = false;
  bool inherit_partition_storage = false;
};

}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_EXPLORER_BROWSER_EXPLORER_POLICY_H_
