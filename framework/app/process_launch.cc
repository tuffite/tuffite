// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/process_launch.h"

#include <utility>

#include "base/command_line.h"
#include "base/strings/string_util.h"
#include "base/strings/utf_string_conversions.h"
#include "build/build_config.h"

namespace tuffite {
namespace {
constexpr size_t kMaxArguments = 4096;
constexpr size_t kMaxArgumentBytes = 1024 * 1024;

bool ValidArguments(const std::vector<std::string>& args) {
  if (args.empty() || args.front().empty() || args.size() > kMaxArguments) {
    return false;
  }
  size_t total = 0;
  for (const auto& arg : args) {
    if (arg.size() > kMaxArgumentBytes || arg.find('\0') != std::string::npos ||
        !base::IsStringUTF8(arg)) {
      return false;
    }
    total += arg.size();
    if (total > kMaxArgumentBytes) {
      return false;
    }
  }
  return true;
}

std::string ProcessType(const std::vector<std::string>& args) {
  base::CommandLine::StringVector native;
  for (const auto& arg : args) {
#if BUILDFLAG(IS_WIN)
    native.push_back(base::UTF8ToWide(arg));
#else
    native.push_back(arg);
#endif
  }
  return base::CommandLine(native).GetSwitchValueASCII("type");
}

uint32_t ProcessKind(const std::string& type) {
  if (type.empty()) {
    return TUFFITE_PROCESS_BROWSER;
  }
  if (type == "utility") {
    return TUFFITE_PROCESS_UTILITY;
  }
  if (type == "renderer") {
    return TUFFITE_PROCESS_RENDERER;
  }
  if (type == "gpu-process") {
    return TUFFITE_PROCESS_GPU;
  }
  return TUFFITE_PROCESS_OTHER;
}

struct ArgumentSink {
  std::vector<std::string> args;
  size_t bytes = 0;
  bool rejected = false;
};

tuffite_status_t AppendArgument(void* context, tuffite_byte_slice_t argument) {
  auto* sink = static_cast<ArgumentSink*>(context);
  if (sink->rejected || argument.len > kMaxArgumentBytes ||
      argument.len > kMaxArgumentBytes - sink->bytes ||
      sink->args.size() >= kMaxArguments ||
      (!argument.data && argument.len != 0)) {
    sink->rejected = true;
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  std::string value;
  if (argument.len != 0) {
    // SAFETY: The startup-hook ABI supplies argument.len readable bytes for
    // this synchronous callback. Length limits are checked before the copy.
    value = UNSAFE_BUFFERS(std::string(
        reinterpret_cast<const char*>(argument.data), argument.len));
  }
  sink->bytes += argument.len;
  sink->args.push_back(std::move(value));
  return TUFFITE_STATUS_OK;
}
}  // namespace

bool PrepareProcessLaunch(const tuffite_application& application,
                          std::vector<std::string>* args) {
  if (!args || args->empty()) {
    return false;
  }
  const std::string original_type = ProcessType(*args);
  if (!application.prepare_process &&
      (!application.prepare_browser || !original_type.empty())) {
    return true;
  }
  if (!ValidArguments(*args)) {
    return false;
  }
  std::vector<tuffite_byte_slice_t> input;
  for (const auto& arg : *args) {
    input.push_back({reinterpret_cast<const uint8_t*>(arg.data()), arg.size()});
  }
  ArgumentSink sink;
  const tuffite_status_t status =
      application.prepare_process
          ? application.prepare_process(ProcessKind(original_type),
                                        input.data(), input.size(), &sink,
                                        &AppendArgument)
          : application.prepare_browser(input.data(), input.size(), &sink,
                                        &AppendArgument);
  if (status != TUFFITE_STATUS_OK || sink.rejected ||
      !ValidArguments(sink.args) || sink.args.front() != args->front() ||
      ProcessType(sink.args) != original_type) {
    return false;
  }
  *args = std::move(sink.args);
  return true;
}
}  // namespace tuffite
