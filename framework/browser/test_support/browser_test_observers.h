// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_TEST_SUPPORT_BROWSER_TEST_OBSERVERS_H_
#define TUFFITE_FRAMEWORK_BROWSER_TEST_SUPPORT_BROWSER_TEST_OBSERVERS_H_

#include <memory>

#include "base/files/file_path.h"

namespace content {
class WebContents;
}
namespace tuffite {
class Window;

class BrowserTestObserver {
 public:
  virtual ~BrowserTestObserver() = default;
};

std::unique_ptr<BrowserTestObserver> CreateSmokeTestObserver(
    content::WebContents* contents,
    const base::FilePath& marker_path);
std::unique_ptr<BrowserTestObserver> CreateWindowSurfaceTestObserver(
    content::WebContents* contents,
    Window* window,
    const base::FilePath& marker_path);
}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_TEST_SUPPORT_BROWSER_TEST_OBSERVERS_H_
