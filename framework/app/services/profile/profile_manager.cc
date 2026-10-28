// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/services/profile/profile_manager.h"

#include <algorithm>

#include "base/files/file_util.h"
#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_util.h"
#include "base/task/single_thread_task_runner.h"
#include "base/timer/timer.h"
#include "content/public/browser/navigation_controller.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_observer.h"
#include "third_party/blink/public/common/renderer_preferences/renderer_preferences.h"
#include "tuffite/framework/app/abi/application.h"
#include "tuffite/framework/app/abi/host.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/window/window.h"
#include "tuffite/framework/browser/services/window/window_service_factory.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/common/languages.h"
#include "ui/base/page_transition_types.h"
#include "url/gurl.h"
namespace tuffite {
namespace {
bool ValidId(const std::string& id) {
  return !id.empty() && id.size() <= 128 && std::ranges::all_of(id, [](char c) {
    return base::IsAsciiLower(c) || base::IsAsciiDigit(c) || c == '-' ||
           c == '_';
  });
}
std::optional<uint64_t> Handle(const base::DictValue& request,
                               const char* key) {
  const auto* value = request.FindString(key);
  uint64_t result = 0;
  return value && base::StringToUint64(*value, &result) && result
             ? std::make_optional(result)
             : std::nullopt;
}
class ViewNavigation final : public content::WebContentsObserver {
 public:
  ViewNavigation(content::WebContents* contents,
                 base::OnceCallback<void(bool)> done)
      : content::WebContentsObserver(contents), done_(std::move(done)) {
    timer_.Start(
        FROM_HERE, base::Seconds(30),
        base::BindOnce(&ViewNavigation::Finish, base::Unretained(this), false));
  }
  void DidFinishNavigation(content::NavigationHandle* navigation) override {
    if (navigation->IsInPrimaryMainFrame() && navigation->HasCommitted()) {
      Finish(!navigation->IsErrorPage());
    }
  }
  void WebContentsDestroyed() override { Finish(false); }

 private:
  void Finish(bool ok) {
    if (!done_) {
      return;
    }
    timer_.Stop();
    Observe(nullptr);
    base::SingleThreadTaskRunner::GetCurrentDefault()->PostTask(
        FROM_HERE, base::BindOnce(std::move(done_), ok));
  }
  base::OnceCallback<void(bool)> done_;
  base::OneShotTimer timer_;
};
}  // namespace
struct ProfileManager::Entry {
  std::unique_ptr<abi::Application> runtime;
  std::unique_ptr<abi::Host> host;
  std::unique_ptr<UtilityService> utilities;
  std::unique_ptr<BrowserContext> browser;
  Reply closing_reply;
  std::vector<OpenReply> pending;
  bool ready = false;
  bool closing = false;
};
struct ProfileManager::View {
  uint64_t context = 0;
  std::unique_ptr<content::WebContents> contents;
  std::unique_ptr<ViewNavigation> navigation;
  Reply pending;
  bool ready = false;
  ~View() {
    navigation.reset();
    if (pending) {
      Complete(std::move(pending), TUFFITE_STATUS_INVALID_STATE, base::Value());
    }
  }
};
ProfileManager::ProfileManager(ApplicationContext& app) : application_(app) {}
ProfileManager::~ProfileManager() {
  Shutdown();
}
bool ProfileManager::Complete(Reply reply, int32_t status, base::Value value) {
  const auto json = base::WriteJson(value).value_or("null");
  return std::move(reply).Run(status,
                              std::vector<uint8_t>(json.begin(), json.end()));
}
bool ProfileManager::Configure(abi::Application& parent, base::FilePath root) {
  if (parent_ || stopped_ || !parent.table().start_profile) {
    return false;
  }
  root_ = root.AppendASCII("profiles");
  if (!base::CreateDirectory(root_)) {
    return false;
  }
  catalog_lock_ =
      base::File(root_.AppendASCII("catalog.lock"),
                 base::File::FLAG_OPEN_ALWAYS | base::File::FLAG_READ |
                     base::File::FLAG_WRITE);
  if (!catalog_lock_.IsValid() ||
      catalog_lock_.Lock(base::File::LockMode::kExclusive) !=
          base::File::FILE_OK) {
    return false;
  }
  const auto index = root_.AppendASCII("catalog.json");
  if (base::PathExists(index)) {
    std::string json;
    if (!base::ReadFileToStringWithMaxSize(index, &json, 1024 * 1024)) {
      return false;
    }
    auto value = base::JSONReader::ReadDict(json, base::JSON_PARSE_RFC);
    if (!value) {
      return false;
    }
    for (const auto [id, name] : *value) {
      if (!ValidId(id) || !name.is_string() || name.GetString().size() > 256) {
        return false;
      }
      catalog_.emplace(id, name.GetString());
    }
  }
  const bool inserted = catalog_.try_emplace("default", "Default").second;
  if (inserted && !SaveCatalog()) {
    return false;
  }
  parent_ = &parent;
  return true;
}
bool ProfileManager::SaveCatalog() {
  base::DictValue value;
  for (const auto& [id, name] : catalog_) {
    value.Set(id, name);
  }
  auto json = base::WriteJson(value);
  if (!json) {
    return false;
  }
  auto temporary = root_.AppendASCII("catalog.json.tmp");
  return base::WriteFile(temporary, *json) &&
         base::ReplaceFile(temporary, root_.AppendASCII("catalog.json"),
                           nullptr);
}
void ProfileManager::Open(const std::string& id, OpenReply reply) {
  if (stopped_ || !parent_ || !catalog_.contains(id)) {
    std::move(reply).Run(nullptr);
    return;
  }
  if (auto found = entries_.find(id); found != entries_.end()) {
    auto& entry = *found->second;
    if (entry.closing) {
      std::move(reply).Run(nullptr);
    } else if (entry.ready) {
      std::move(reply).Run(entry.browser.get());
    } else {
      entry.pending.push_back(std::move(reply));
    }
    return;
  }
  if (entries_.size() >= 32) {
    std::move(reply).Run(nullptr);
    return;
  }
  auto entry = std::make_unique<Entry>();
  entry->pending.push_back(std::move(reply));
  entry->runtime = abi::Application::Create(&parent_->table());
  entry->host = std::make_unique<abi::Host>(base::BindRepeating(
      &abi::Application::OnEvent, base::Unretained(entry->runtime.get())));
  entry->host->BindApplicationContext(&*application_);
  entry->host->BindBrowserTaskRunner();
  entry->runtime->BindBrowserTaskRunner();
  // Preserve the existing default profile without copying databases on startup.
  const auto path =
      id == "default" ? root_.DirName() : root_.AppendASCII("profile-" + id);
  if (!base::CreateDirectory(path)) {
    auto pending = std::move(entry->pending);
    for (auto& cb : pending) {
      std::move(cb).Run(nullptr);
    }
    return;
  }
  entry->utilities = CreateUtilityService();
  entry->browser =
      std::make_unique<BrowserContext>(path, *application_, *entry->runtime,
                                       root_.DirName(), entry->utilities.get());
  entry->host->BindBrowserContext(entry->browser->GetWeakPtr());
  entry->runtime->SetApplicationEvent(entry->host->application_event());
  auto* runtime = entry->runtime.get();
  auto* host = entry->host.get();
  entries_.emplace(id, std::move(entry));
  if (!runtime->StartProfile(*parent_, host->table(), *application_, id,
                             *entries_.at(id)->utilities)) {
    Opened(id, false);
    return;
  }
  runtime->InitializeBrowser(
      base::BindOnce(&ProfileManager::Opened, weak_factory_.GetWeakPtr(), id));
}
void ProfileManager::Opened(const std::string& id, bool ok) {
  auto found = entries_.find(id);
  if (found == entries_.end()) {
    return;
  }
  auto& entry = *found->second;
  entry.ready = ok;
  auto pending = std::move(entry.pending);
  if (!ok) {
    entry.host->Close();
    entry.utilities->Shutdown();
    entry.runtime->Stop();
    entry.browser.reset();
    entry.host->Shutdown();
    retired_hosts_.push_back(std::move(entry.host));
    entries_.erase(found);
  }
  for (auto& cb : pending) {
    std::move(cb).Run(ok ? entry.browser.get() : nullptr);
  }
}
ProfileManager::Entry* ProfileManager::FindRuntime(uint64_t handle) {
  for (auto& [id, entry] : entries_) {
    if (entry->runtime->runtime() == handle && entry->ready &&
        !entry->closing) {
      return entry.get();
    }
  }
  return nullptr;
}
uint64_t ProfileManager::RuntimeFor(content::WebContents* contents) {
  for (const auto& [id, entry] : entries_) {
    if (entry->browser.get() == contents->GetBrowserContext()) {
      return entry->runtime->runtime();
    }
  }
  return 0;
}
void ProfileManager::Closed(const std::string& id, Reply reply) {
  auto found = entries_.find(id);
  if (found != entries_.end()) {
    found->second->browser.reset();
    found->second->host->Shutdown();
    retired_hosts_.push_back(std::move(found->second->host));
    entries_.erase(found);
  }
  Complete(std::move(reply), TUFFITE_STATUS_OK, base::Value());
}
void ProfileManager::ViewReady(uint64_t id, bool ok) {
  auto found = views_.find(id);
  if (found == views_.end()) {
    return;
  }
  auto& view = *found->second;
  view.navigation.reset();
  auto reply = std::move(view.pending);
  view.ready = ok;
  if (!ok) {
    views_.erase(found);
  }
  const bool accepted = Complete(
      std::move(reply), ok ? TUFFITE_STATUS_OK : TUFFITE_STATUS_INVALID_STATE,
      base::Value(base::NumberToString(id)));
  if (ok && !accepted) {
    views_.erase(id);
  }
}
void ProfileManager::Apply(const base::DictValue& request,
                           BrowserContext* caller,
                           Reply reply) {
  const auto* op = request.FindString("op");
  if (stopped_ || !parent_ || !op) {
    Complete(std::move(reply), TUFFITE_STATUS_INVALID_STATE, base::Value());
    return;
  }
  const auto context = Handle(request, "context");
  auto* entry = context ? FindRuntime(*context) : nullptr;
  if (caller && (!op->starts_with("views.") || !entry ||
                 entry->browser.get() != caller)) {
    Complete(std::move(reply), TUFFITE_STATUS_INVALID_ARGUMENT, base::Value());
    return;
  }
  auto* windows = application_->GetService<window::WindowService>();
  if (*op == "windows.create") {
    const int width = request.FindInt("width").value_or(800);
    const int height = request.FindInt("height").value_or(600);
    if (width < 1 || height < 1 || width > 16384 || height > 16384) {
      Complete(std::move(reply), TUFFITE_STATUS_INVALID_ARGUMENT,
               base::Value());
      return;
    }
    auto* window =
        windows->CreateEmpty(gfx::Size(width, height),
                             request.FindBool("decorations").value_or(true));
    if (!window) {
      Complete(std::move(reply), TUFFITE_STATUS_INVALID_STATE, base::Value());
      return;
    }
    const auto id = windows->GetInfo(window)->id;
    if (!Complete(std::move(reply), TUFFITE_STATUS_OK,
                  base::Value(base::NumberToString(id)))) {
      windows->Close(window);
    }
    return;
  }
  if (*op == "windows.close") {
    const auto id = Handle(request, "window");
    auto* window = id ? windows->FindById(*id) : nullptr;
    const bool closed = window && windows->Close(window);
    Complete(std::move(reply),
             closed ? TUFFITE_STATUS_OK : TUFFITE_STATUS_INVALID_ARGUMENT,
             base::Value());
    return;
  }
  if (*op == "profiles.list") {
    base::ListValue result;
    for (const auto& [id, name] : catalog_) {
      result.Append(
          base::DictValue()
              .Set("id", id)
              .Set("name", name)
              .Set("loaded", entries_.contains(id) && entries_.at(id)->ready));
    }
    Complete(std::move(reply), TUFFITE_STATUS_OK,
             base::Value(std::move(result)));
    return;
  }
  if (*op == "profiles.create") {
    const auto* id = request.FindString("id");
    const auto* name = request.FindString("name");
    if (id && name && ValidId(*id) && !name->empty() && name->size() <= 256 &&
        catalog_.size() < 1024 && !catalog_.contains(*id)) {
      catalog_.emplace(*id, *name);
      if (SaveCatalog()) {
        Complete(std::move(reply), TUFFITE_STATUS_OK, base::Value());
        return;
      }
      catalog_.erase(*id);
    }
  } else if (*op == "profiles.open") {
    if (const auto* id = request.FindString("id")) {
      Open(*id,
           base::BindOnce(
               [](base::WeakPtr<ProfileManager> self, Reply reply,
                  BrowserContext* browser) {
                 uint64_t runtime = 0;
                 if (self && browser) {
                   for (const auto& [profile_key, loaded] : self->entries_) {
                     if (loaded->browser.get() == browser) {
                       runtime = loaded->runtime->runtime();
                     }
                   }
                 }
                 Complete(
                     std::move(reply),
                     runtime ? TUFFITE_STATUS_OK : TUFFITE_STATUS_INVALID_STATE,
                     base::Value(base::NumberToString(runtime)));
               },
               weak_factory_.GetWeakPtr(), std::move(reply)));
      return;
    }
  } else if (*op == "profiles.close" && entry) {
    bool busy = windows->HasContext(*entry->browser);
    for (const auto& [id, view] : views_) {
      busy |= view->context == *context;
    }
    if (!busy) {
      std::string id;
      for (const auto& [key, value] : entries_) {
        if (value.get() == entry) {
          id = key;
        }
      }
      entry->closing = true;
      entry->host->Close();
      entry->utilities->Shutdown();
      entry->closing_reply = std::move(reply);
      entry->runtime->StopAsync(base::BindOnce(
          [](base::WeakPtr<ProfileManager> self, std::string id) {
            if (self) {
              self->Closed(id, std::move(self->entries_.at(id)->closing_reply));
            }
          },
          weak_factory_.GetWeakPtr(), id));
      return;
    }
  } else if (*op == "views.create" && entry && views_.size() < 128) {
    const auto* url = request.FindString("url");
    if (url && GURL(*url).is_valid()) {
      const uint64_t id = next_view_++;
      auto view = std::make_unique<View>();
      view->context = *context;
      view->contents = content::WebContents::Create(
          content::WebContents::CreateParams(entry->browser.get()));
      WebContentsRoleData::SetRole(view->contents.get(), WebContentsRole::kApp);
      view->contents->GetMutableRendererPrefs()->accept_languages =
          kAcceptLanguages;
      view->pending = std::move(reply);
      view->navigation = std::make_unique<ViewNavigation>(
          view->contents.get(), base::BindOnce(&ProfileManager::ViewReady,
                                               weak_factory_.GetWeakPtr(), id));
      auto* contents = view->contents.get();
      views_.emplace(id, std::move(view));
      content::NavigationController::LoadURLParams params{GURL(*url)};
      params.transition_type = ui::PAGE_TRANSITION_TYPED;
      contents->GetController().LoadURLWithParams(params);
      return;
    }
  } else if (*op == "views.close") {
    const auto id = Handle(request, "view");
    auto found = id ? views_.find(*id) : views_.end();
    if (found != views_.end() &&
        (!context || found->second->context == *context)) {
      views_.erase(found);
      Complete(std::move(reply), TUFFITE_STATUS_OK, base::Value());
      return;
    }
  } else if (*op == "windows.setContent") {
    const auto id = Handle(request, "view");
    const auto window = Handle(request, "window");
    auto found = id ? views_.find(*id) : views_.end();
    auto* target = window ? windows->FindById(*window) : nullptr;
    if (found != views_.end() && found->second->ready && target) {
      auto old = std::make_unique<View>();
      old->context =
          target->web_contents() ? RuntimeFor(target->web_contents()) : 0;
      old->ready = true;
      old->contents = target->SetContent(std::move(found->second->contents));
      views_.erase(found);
      if (!old->contents) {
        Complete(std::move(reply), TUFFITE_STATUS_OK, base::Value());
        return;
      }
      const auto previous = next_view_++;
      views_.emplace(previous, std::move(old));
      if (!Complete(std::move(reply), TUFFITE_STATUS_OK,
                    base::Value(base::NumberToString(previous)))) {
        views_.erase(previous);
      }
      return;
    }
  }
  Complete(std::move(reply), TUFFITE_STATUS_INVALID_ARGUMENT, base::Value());
}
void ProfileManager::Shutdown() {
  if (stopped_) {
    return;
  }
  stopped_ = true;
  weak_factory_.InvalidateWeakPtrs();
  views_.clear();
  auto* windows = application_->GetService<window::WindowService>(false);
  if (windows) {
    windows->Shutdown();
  }
  for (auto& [id, entry] : entries_) {
    entry->host->Close();
    entry->utilities->Shutdown();
    entry->runtime->Stop();
    if (entry->closing_reply) {
      Complete(std::move(entry->closing_reply), TUFFITE_STATUS_INVALID_STATE,
               base::Value());
    }
    for (auto& pending : entry->pending) {
      std::move(pending).Run(nullptr);
    }
    entry->pending.clear();
    entry->browser.reset();
    entry->host->Shutdown();
  }
  entries_.clear();
  retired_hosts_.clear();
  parent_ = nullptr;
  catalog_lock_.Close();
}
}  // namespace tuffite
