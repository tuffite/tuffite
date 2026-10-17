// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_POINTER_GEOMETRY_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_POINTER_GEOMETRY_H_

#include <algorithm>
#include <cmath>
#include <limits>
#include <optional>
#include <utility>

namespace tuffite::remoting {

// The platform adapter supplies bounds in the injector's coordinate space:
// macOS global logical points; Windows virtual-desktop-relative physical
// pixels. Fractional input is independent of capture/encoder/Client display
// scaling.
struct PointerGeometry {
  int frame_width = 0;
  int frame_height = 0;
  double left = 0;
  double top = 0;
  double width = 0;
  double height = 0;

  std::optional<std::pair<int, int>> Map(int x, int y) const {
    if (frame_width <= 0 || frame_height <= 0) {
      return std::nullopt;
    }
    return MapFractional(static_cast<double>(x) / frame_width,
                         static_cast<double>(y) / frame_height);
  }

  std::optional<std::pair<int, int>> MapFractional(double x, double y) const {
    if (!std::isfinite(x) || !std::isfinite(y) || !std::isfinite(left) ||
        !std::isfinite(top) || !std::isfinite(width) ||
        !std::isfinite(height) || width < 1 || height < 1 ||
        left < std::numeric_limits<int>::min() ||
        top < std::numeric_limits<int>::min() ||
        left + width - 1 > std::numeric_limits<int>::max() ||
        top + height - 1 > std::numeric_limits<int>::max()) {
      return std::nullopt;
    }
    const auto axis = [](double fraction, double origin, double size) {
      const double offset = std::clamp(fraction, 0.0, 1.0) * size;
      return static_cast<int>(std::round(origin + std::min(offset, size - 1)));
    };
    return std::pair(axis(x, left, width), axis(y, top, height));
  }
};

}  // namespace tuffite::remoting

#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_POINTER_GEOMETRY_H_
