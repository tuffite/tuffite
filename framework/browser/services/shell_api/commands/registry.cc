// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/registry.h"

#include <utility>

#include "tuffite/framework/browser/services/shell_api/commands/dialog.h"
#include "tuffite/framework/browser/services/shell_api/commands/fs.h"
#include "tuffite/framework/browser/services/shell_api/commands/log.h"
#include "tuffite/framework/browser/services/shell_api/commands/network.h"
#include "tuffite/framework/browser/services/shell_api/commands/path.h"
#include "tuffite/framework/browser/services/shell_api/commands/remoting.h"
#include "tuffite/framework/browser/services/shell_api/commands/storage.h"
#include "tuffite/framework/browser/services/shell_api/commands/utility.h"
#include "tuffite/framework/browser/services/shell_api/commands/window.h"
#include "tuffite/framework/shell_api/common.h"
#include "tuffite/framework/shell_api/dialog.h"
#include "tuffite/framework/shell_api/fs.h"
#include "tuffite/framework/shell_api/log.h"
#include "tuffite/framework/shell_api/network.h"
#include "tuffite/framework/shell_api/path.h"
#include "tuffite/framework/shell_api/remoting.h"
#include "tuffite/framework/shell_api/storage.h"
#include "tuffite/framework/shell_api/utility.h"
#include "tuffite/framework/shell_api/window.h"

namespace tuffite::shell_api {

Registry::Registry() = default;
Registry::~Registry() = default;
Registry::Registry(Registry&&) noexcept = default;
Registry& Registry::operator=(Registry&&) noexcept = default;

std::vector<Route> BuiltinRoutes() {
  namespace refl = generated;
  Registry registry;
  Register(registry, refl::dialog::reflection::openFile, &dialog::OpenFile);
  Register(registry, refl::dialog::reflection::openDirectory,
           &dialog::OpenDirectory);
  Register(registry, refl::fs::reflection::readFile, &fs::ReadFile);
  Register(registry, refl::fs::reflection::writeFile, &fs::WriteFile);
  Register(registry, refl::fs::reflection::exists, &fs::Exists);
  Register(registry, refl::fs::reflection::createDir, &fs::CreateDir);
  Register(registry, refl::fs::reflection::removeFile, &fs::RemoveFile);
  Register(registry, refl::log::reflection::trace, &log::Trace);
  Register(registry, refl::log::reflection::debug, &log::Debug);
  Register(registry, refl::log::reflection::info, &log::Info);
  Register(registry, refl::log::reflection::warn, &log::Warn);
  Register(registry, refl::log::reflection::error, &log::Error);
  Register(registry, refl::net::reflection::request, &net::Request);
  Register(registry, refl::path::reflection::directories, &path::Directories);
  Register(registry, refl::path::reflection::resolve, &path::Resolve);
  Register(registry, refl::remoting::reflection::capabilities,
           &remoting::Capabilities);
  Register(registry, refl::remoting::reflection::start, &remoting::Start);
  Register(registry, refl::remoting::reflection::processSignal,
           &remoting::ProcessSignal);
  Register(registry, refl::remoting::reflection::takeSignals,
           &remoting::TakeSignals);
  Register(registry, refl::remoting::reflection::sendInput,
           &remoting::SendInput);
  Register(registry, refl::remoting::reflection::stop, &remoting::Stop);
  Register(registry, refl::remoting::reflection::stats, &remoting::Stats);
  Register(registry, refl::remoting::reflection::capture, &remoting::Capture);
  Register(registry, refl::remoting::reflection::injectInput,
           &remoting::InjectInput);
  Register(registry, refl::storage::reflection::get, &storage::Get);
  Register(registry, refl::storage::reflection::set, &storage::Set);
  Register(registry, refl::storage::reflection::remove, &storage::Remove);
  Register(registry, refl::storage::reflection::clear, &storage::Clear);
  Register(registry, refl::utility::reflection::invoke, &utility::Invoke);
  Register(registry, refl::utility::reflection::cancel, &utility::Cancel);
  Register(registry, refl::window::reflection::current, &window::Current);
  Register(registry, refl::window::reflection::list, &window::List);
  Register(registry, refl::window::reflection::create, &window::Create);
  Register(registry, refl::window::reflection::focus, &window::Focus);
  Register(registry, refl::window::reflection::minimize, &window::Minimize);
  Register(registry, refl::window::reflection::maximize, &window::Maximize);
  Register(registry, refl::window::reflection::restore, &window::Restore);
  Register(registry, refl::window::reflection::setFullscreen,
           &window::SetFullscreen);
  Register(registry, refl::window::reflection::close, &window::Close);
  Register(registry, refl::window::reflection::setTitle, &window::SetTitle);
  Register(registry, refl::window::reflection::getBounds, &window::GetBounds);
  Register(registry, refl::window::reflection::setBounds, &window::SetBounds);
  return std::move(registry).Build();
}

}  // namespace tuffite::shell_api
