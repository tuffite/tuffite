// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/remoting/pointer_geometry.h"

#include <cstdlib>
#include <limits>
#include <utility>

namespace {

void ExpectPoint(const tuffite::remoting::PointerGeometry& geometry,
                 int x, int y, int expected_x, int expected_y) {
  const auto point = geometry.Map(x, y);
  if (!point || *point != std::pair(expected_x, expected_y)) {
    std::abort();
  }
}

void ExpectFractional(const tuffite::remoting::PointerGeometry& geometry,
                      double x, double y, int expected_x, int expected_y) {
  const auto point = geometry.MapFractional(x, y);
  if (!point || *point != std::pair(expected_x, expected_y)) {
    std::abort();
  }
}

}  // namespace

// This geometry test intentionally has no Chromium runtime dependencies, so it
// can also be compiled and run on a machine without a Chromium checkout.
int main() {
  using tuffite::remoting::PointerGeometry;
  if (PointerGeometry{}.Map(100, 100)) {
    std::abort();
  }
  const PointerGeometry retina{3000, 2000, 0, 0, 1500, 1000};
  ExpectPoint(retina, 1500, 1000, 750, 500);
  ExpectPoint(retina, 750, 500, 375, 250);
  ExpectPoint(retina, 2999, 1999, 1499, 999);
  ExpectPoint(retina, -10, 2500, 0, 999);

  // Selected displays may sit left of or above the main display.
  const PointerGeometry secondary{3840, 2160, -1920, -1080, 1920, 1080};
  ExpectPoint(secondary, 1920, 1080, -960, -540);
  ExpectPoint(secondary, 0, 0, -1920, -1080);
  ExpectPoint(secondary, 3839, 2159, -1, -1);

  // Non-Retina and a live resolution change use the same Host bounds.
  const PointerGeometry standard{1920, 1080, 1440, 0, 1920, 1080};
  ExpectPoint(standard, 960, 540, 2400, 540);
  const PointerGeometry resized{1500, 1000, 0, 0, 1500, 1000};
  ExpectPoint(resized, 750, 500, 750, 500);
  const PointerGeometry non_uniform{3000, 1000, 0, 0, 1500, 1000};
  ExpectPoint(non_uniform, 1500, 500, 750, 500);

  // A 4K Windows capture can be encoded at 1080p or 720p: fractional input
  // always resolves to the same physical target. DPI does not enter this map.
  const PointerGeometry windows_4k{3840, 2160, 0, 0, 3840, 2160};
  ExpectFractional(windows_4k, 0.75, 0.25, 2880, 540);
  ExpectFractional(windows_4k, 1440.0 / 1920, 270.0 / 1080, 2880, 540);
  ExpectFractional(windows_4k, 960.0 / 1280, 180.0 / 720, 2880, 540);

  // Windows capture metadata already subtracts a negative virtual-desktop
  // origin. Main screen (global 0,0) in a desktop starting at (-1920,-1080)
  // therefore has capture top_left=(1920,1080), not (0,0).
  const PointerGeometry windows_main{2560, 1440, 1920, 1080, 2560, 1440};
  ExpectFractional(windows_main, 0.5, 0.5, 3200, 1800);
  ExpectFractional(windows_main, 0, 0, 1920, 1080);
  ExpectFractional(windows_main, 1, 1, 4479, 2519);
  const PointerGeometry windows_left{1920, 1080, 0, 1080, 1920, 1080};
  ExpectFractional(windows_left, 0.5, 0.5, 960, 1620);
  const PointerGeometry windows_above{2560, 1440, 1920, 0, 2560, 1440};
  ExpectFractional(windows_above, 0.5, 0.5, 3200, 720);
  const PointerGeometry windows_full{4480, 2520, 0, 0, 4480, 2520};
  ExpectFractional(windows_full, 0.5, 0.5, 2240, 1260);

  // macOS Retina + encoder reduction resolves in logical points, and changes
  // to captured pixels do not affect a stable fractional location.
  ExpectFractional(retina, 0.75, 0.25, 1125, 250);
  ExpectFractional(resized, 0.75, 0.25, 1125, 250);
  ExpectFractional(secondary, 0.5, 0.5, -960, -540);
  ExpectFractional(retina, 1, 1, 1499, 999);
  ExpectFractional(retina, -1, 2, 0, 999);
  if (retina.MapFractional(std::numeric_limits<double>::quiet_NaN(), 0.5) ||
      retina.MapFractional(0.5, std::numeric_limits<double>::infinity()) ||
      PointerGeometry{}.MapFractional(0.5, 0.5)) {
    std::abort();
  }
  const PointerGeometry overflow{1, 1, 0, 0, 1e100, 1};
  if (overflow.MapFractional(1, 0)) {
    std::abort();
  }
  return 0;
}
