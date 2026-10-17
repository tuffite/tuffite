// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_LANGUAGES_H_
#define TUFFITE_FRAMEWORK_COMMON_LANGUAGES_H_
namespace tuffite {
// The current application locale is en-US; use the same ordered languages in
// network headers, document preferences and workers.
inline constexpr char kAcceptLanguages[] = "en-US,en";

}  // namespace tuffite
#endif  // TUFFITE_FRAMEWORK_COMMON_LANGUAGES_H_
