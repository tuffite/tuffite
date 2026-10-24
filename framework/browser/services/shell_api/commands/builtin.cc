// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/shell_api/commands/builtin.h"

#include "tuffite/framework/browser/services/shell_api/commands/dialog.h"
#include "tuffite/framework/browser/services/shell_api/commands/fs.h"
#include "tuffite/framework/browser/services/shell_api/commands/log.h"
#include "tuffite/framework/browser/services/shell_api/commands/network.h"
#include "tuffite/framework/browser/services/shell_api/commands/path.h"
#include "tuffite/framework/browser/services/shell_api/commands/remoting.h"
#include "tuffite/framework/browser/services/shell_api/commands/storage.h"
#include "tuffite/framework/browser/services/shell_api/commands/utility.h"
#include "tuffite/framework/browser/services/shell_api/commands/window.h"
namespace tuffite::shell_api {
std::vector<rpc::Service> BuiltinServices() {
  return {
      generated::Dialog::Bind(std::make_shared<dialog::DialogImpl>()),
      generated::Fs::Bind(std::make_shared<fs::FsImpl>()),
      generated::Log::Bind(std::make_shared<log::LogImpl>()),
      generated::Net::Bind(std::make_shared<net::NetImpl>()),
      generated::Path::Bind(std::make_shared<path::PathImpl>()),
      generated::Remoting::Bind(std::make_shared<remoting::RemotingImpl>()),
      generated::Storage::Bind(std::make_shared<storage::StorageImpl>()),
      generated::Utility::Bind(std::make_shared<utility::UtilityImpl>()),
      generated::Window::Bind(std::make_shared<window::WindowImpl>()),
  };
}
}  // namespace tuffite::shell_api
