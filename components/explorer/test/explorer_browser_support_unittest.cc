// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/explorer/browser/explorer_browser_support.h"

#include <string>
#include <utility>

#include "base/functional/bind.h"
#include "testing/gtest/include/gtest/gtest.h"

namespace tuffite {
namespace {

TEST(ExplorerOpenKindTest, UnifiesNewWindowsAndKeepsNavigationDistinct) {
  EXPECT_EQ("navigation",
            ExplorerOpenEventType(WindowOpenDisposition::CURRENT_TAB, false));
  EXPECT_EQ("newwindow", ExplorerOpenEventType(
                             WindowOpenDisposition::NEW_FOREGROUND_TAB, true));
  EXPECT_EQ("newwindow", ExplorerOpenEventType(
                             WindowOpenDisposition::NEW_BACKGROUND_TAB, false));
  EXPECT_EQ("newwindow",
            ExplorerOpenEventType(WindowOpenDisposition::NEW_POPUP, true));
  EXPECT_EQ("newwindow",
            ExplorerOpenEventType(WindowOpenDisposition::NEW_WINDOW, true));
  EXPECT_EQ("newwindow",
            ExplorerOpenEventType(WindowOpenDisposition::NEW_WINDOW, false));
}

TEST(ExplorerDialogStateTest, RespondsOnlyToTheActiveRequest) {
  ExplorerDialogState state;
  bool accepted = false;
  std::u16string prompt;
  const uint64_t request_id = state.Begin(base::BindOnce(
      [](bool* accepted, std::u16string* prompt, bool result,
         const std::u16string& value) {
        *accepted = result;
        *prompt = value;
      },
      &accepted, &prompt));

  EXPECT_FALSE(state.Respond(request_id + 1, true, u"wrong"));
  EXPECT_TRUE(state.has_pending_dialog());
  EXPECT_TRUE(state.Respond(request_id, true, u"answer"));
  EXPECT_TRUE(accepted);
  EXPECT_EQ(u"answer", prompt);
  EXPECT_FALSE(state.has_pending_dialog());
  EXPECT_FALSE(state.Respond(request_id, true, u"again"));
}

TEST(ExplorerDialogStateTest, ReplacementAndCancellationRejectOldDialog) {
  ExplorerDialogState state;
  bool first_result = true;
  bool second_result = true;
  const uint64_t first_id = state.Begin(
      base::BindOnce([](bool* result, bool accepted,
                        const std::u16string&) { *result = accepted; },
                     &first_result));
  const uint64_t second_id = state.Begin(
      base::BindOnce([](bool* result, bool accepted,
                        const std::u16string&) { *result = accepted; },
                     &second_result));

  EXPECT_FALSE(first_result);
  EXPECT_NE(first_id, second_id);
  EXPECT_FALSE(state.Respond(first_id, true, std::u16string()));
  state.Cancel();
  EXPECT_FALSE(second_result);
  EXPECT_FALSE(state.has_pending_dialog());
}

}  // namespace
}  // namespace tuffite
