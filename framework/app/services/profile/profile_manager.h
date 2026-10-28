// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_PROFILE_PROFILE_MANAGER_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_PROFILE_PROFILE_MANAGER_H_
#include <map>
#include <memory>
#include <string>
#include <vector>

#include "base/files/file.h"
#include "base/files/file_path.h"
#include "base/functional/callback.h"
#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "base/values.h"
#include "components/keyed_service/core/keyed_service.h"
#include "tuffite/framework/public/c_api/application.h"
namespace content {
class WebContents;
}
namespace tuffite {
class ApplicationContext;
class BrowserContext;
namespace abi {
class Application;
class Host;
}  // namespace abi

// One application owns the catalog and every loaded profile. No renderer can
// invoke this trusted API directly. All operations run on the browser sequence.
class ProfileManager : public KeyedService {
 public:
  using Reply = base::OnceCallback<bool(int32_t, std::vector<uint8_t>)>;
  using OpenReply = base::OnceCallback<void(BrowserContext*)>;
  explicit ProfileManager(ApplicationContext&);
  ~ProfileManager() override;
  bool Configure(abi::Application&, base::FilePath root);
  void Open(const std::string& id, OpenReply);
  void Apply(const base::DictValue&, BrowserContext* caller, Reply);
  void Shutdown() override;

 private:
  struct Entry;
  struct View;
  void Opened(const std::string&, bool);
  void Closed(const std::string&, Reply);
  void ViewReady(uint64_t, bool);
  bool SaveCatalog();
  Entry* FindRuntime(uint64_t);
  uint64_t RuntimeFor(content::WebContents*);
  static bool Complete(Reply, int32_t, base::Value);
  const raw_ref<ApplicationContext> application_;
  raw_ptr<abi::Application> parent_ = nullptr;
  base::FilePath root_;
  base::File catalog_lock_;
  std::map<std::string, std::string> catalog_;
  std::map<std::string, std::unique_ptr<Entry>> entries_;
  // Retain only retired Host tables until application teardown. Their request
  // and runtime-event admission is closed; stopped runtime/utility objects go.
  std::vector<std::unique_ptr<abi::Host>> retired_hosts_;
  std::map<uint64_t, std::unique_ptr<View>> views_;
  uint64_t next_view_ = 1;
  bool stopped_ = false;
  base::WeakPtrFactory<ProfileManager> weak_factory_{this};
};
}  // namespace tuffite
#endif
