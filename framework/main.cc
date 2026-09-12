// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "build/build_config.h"
#include "tuffite/framework/public/abi.h"

#if BUILDFLAG(IS_WIN)
#include <windows.h>

#include "content/public/app/sandbox_helper_win.h"
#include "sandbox/win/src/sandbox_types.h"

int APIENTRY wWinMain(HINSTANCE, HINSTANCE, wchar_t*, int) {
  // Chromium's Windows sandbox broker state must be initialized by the process
  // executable, then passed across the narrow Tuffite entry point.
  sandbox::SandboxInterfaceInfo sandbox_info = {};
  content::InitializeSandboxInfo(&sandbox_info);
  return tuffite_bootstrap(0, nullptr, &sandbox_info);
}
#else
int main(int argc, const char* argv[]) {
  return tuffite_bootstrap(argc, argv, nullptr);
}
#endif
