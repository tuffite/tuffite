// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <string>
#include <utility>
#include <vector>

#include "base/command_line.h"
#include "base/containers/span.h"
#include "base/strings/utf_string_conversions.h"
#include "build/build_config.h"
#include "tuffite/framework/app/abi_host_bridge.h"
#include "tuffite/framework/app/main_delegate.h"
#include "tuffite/framework/app/process_launch.h"
#include "tuffite/framework/public/abi.h"

// DLL-side C ABI entry. Owns the process composition root for this run.
extern "C" int tuffite_run_application(const tuffite_application* application,
                                       int argc,
                                       const char* const* argv,
                                       void* sandbox_info) {
  auto bridge = tuffite::AbiHostBridge::Create(application);
  if (!bridge) {
    return 64;
  }
  std::vector<std::string> args;
#if BUILDFLAG(IS_WIN)
  // Windows ContentMain consumes the process CommandLine rather than argc/argv.
  if (!base::CommandLine::InitializedForCurrentProcess()) {
    base::CommandLine::Init(0, nullptr);
  }
  for (const auto& arg : base::CommandLine::ForCurrentProcess()->argv()) {
    args.push_back(base::WideToUTF8(arg));
  }
#else
  if (argc <= 0 || !argv) {
    return 64;
  }
  // SAFETY: The C entry-point caller supplies argc live argv entries, each
  // pointing to a null-terminated string for this synchronous call.
  auto arguments = UNSAFE_BUFFERS(base::span(argv, static_cast<size_t>(argc)));
  for (const char* arg : arguments) {
    if (!arg) {
      return 64;
    }
    args.emplace_back(arg);
  }
#endif
  if (!tuffite::PrepareProcessLaunch(bridge->application(), &args)) {
    return 64;
  }
#if BUILDFLAG(IS_WIN)
  base::CommandLine::StringVector native;
  for (const auto& arg : args) {
    native.push_back(base::UTF8ToWide(arg));
  }
  base::CommandLine::ForCurrentProcess()->InitFromArgv(native);
#endif
  std::vector<const char*> pointers;
  for (const auto& arg : args) {
    pointers.push_back(arg.c_str());
  }
  pointers.push_back(nullptr);
  tuffite::MainDelegate app(std::move(bridge));
  return app.Run(static_cast<int>(args.size()), pointers.data(), sandbox_info);
}
