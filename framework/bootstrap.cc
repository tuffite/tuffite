// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <optional>
#include <string>

#include "base/files/file_path.h"
#include "base/files/file_util.h"
#include "base/json/json_reader.h"
#include "base/logging.h"
#include "base/native_library.h"
#include "base/path_service.h"
#include "build/build_config.h"
#include "tuffite/framework/public/abi.h"

#if BUILDFLAG(IS_MAC)
#include "base/apple/foundation_util.h"
#include "sandbox/mac/seatbelt_exec.h"
#endif

namespace {

using GetApplicationFn = const tuffite_application* (*)();

}  // namespace

TUFFITE_FRAMEWORK_EXPORT int tuffite_bootstrap(int argc,
                                               const char* const* argv,
                                               void* sandbox_info) {
#if BUILDFLAG(IS_MAC)
  sandbox::SeatbeltExecServer::CreateFromArgumentsResult seatbelt =
      sandbox::SeatbeltExecServer::CreateFromArguments(argv[0], argc, argv);
  if (seatbelt.sandbox_required &&
      (!seatbelt.server || !seatbelt.server->InitializeSandbox())) {
    LOG(ERROR) << "Tuffite bootstrap: unable to initialize Seatbelt sandbox";
    return 1;
  }
#endif

  base::FilePath executable_directory;
  if (!base::PathService::Get(base::DIR_EXE, &executable_directory)) {
    LOG(ERROR) << "Tuffite bootstrap: unable to resolve executable directory";
    return 64;
  }
  std::string manifest_contents;
  base::FilePath package_directory = executable_directory;
#if BUILDFLAG(IS_MAC)
  if (executable_directory.BaseName().value() == FILE_PATH_LITERAL("MacOS") &&
      executable_directory.DirName().BaseName().value() ==
          FILE_PATH_LITERAL("Contents")) {
    package_directory = executable_directory.DirName().AppendASCII("Resources");
    // Tuffite runs all child roles through the application executable. Chrome's
    // bundled mode rewrites that explicit path to sibling helper app bundles.
    // Keep native bundle identity/resources while using its single-executable
    // process model. This does not change Seatbelt initialization above.
    base::apple::SetOverrideAmIBundled(false);
  }
#endif
  const base::FilePath manifest_path =
      package_directory.Append(FILE_PATH_LITERAL("manifest.json"));
  if (!base::ReadFileToString(manifest_path, &manifest_contents)) {
    LOG(ERROR) << "Tuffite bootstrap: unable to read package manifest";
    return 64;
  }
  std::optional<base::DictValue> manifest =
      base::JSONReader::ReadDict(manifest_contents, base::JSON_PARSE_RFC);
  const std::string* runtime_library =
      manifest ? manifest->FindString("runtime_library") : nullptr;
  if (!runtime_library || runtime_library->empty()) {
    LOG(ERROR) << "Tuffite bootstrap: package manifest has no runtime library";
    return 64;
  }
  const base::FilePath runtime_name =
      base::FilePath::FromUTF8Unsafe(*runtime_library);
  if (runtime_name.empty() || runtime_name.IsAbsolute() ||
      runtime_name.ReferencesParent() ||
      runtime_name.BaseName() != runtime_name) {
    LOG(ERROR) << "Tuffite bootstrap: invalid runtime library name";
    return 64;
  }
  const base::FilePath runtime_path = executable_directory.Append(runtime_name);
  base::NativeLibraryLoadError error;
  base::NativeLibrary library = base::LoadNativeLibrary(runtime_path, &error);
  if (!library) {
    LOG(ERROR) << "Tuffite bootstrap: unable to load application runtime: "
               << error.ToString();
    return 64;
  }
  auto* get_application = reinterpret_cast<GetApplicationFn>(
      base::GetFunctionPointerFromNativeLibrary(library,
                                                "tuffite_get_application"));
  const tuffite_application* application =
      get_application ? get_application() : nullptr;
  if (!get_application) {
    LOG(ERROR) << "Tuffite bootstrap: application getter export is missing";
  } else if (!application) {
    LOG(ERROR) << "Tuffite bootstrap: application initialization failed";
  }
  const int result = application ? tuffite_run_application(application, argc,
                                                           argv, sandbox_info)
                                 : 64;
  base::UnloadNativeLibrary(library);
  return result;
}
