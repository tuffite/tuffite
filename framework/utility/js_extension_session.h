// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_UTILITY_JS_EXTENSION_SESSION_H_
#define TUFFITE_FRAMEWORK_UTILITY_JS_EXTENSION_SESSION_H_

#include <deque>
#include <map>
#include <memory>
#include <set>
#include <string>
#include <utility>
#include <vector>

#include "base/containers/span.h"
#include "base/functional/callback.h"
#include "base/memory/weak_ptr.h"
#include "base/timer/timer.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/framework/common/utility/utility_service.mojom.h"

namespace tuffite {

// One capability-free ECMAScript-module realm hosted on its own sequence.
// Extension code receives JSON values and can only return JSON values. All
// actual process, network, filesystem, secret, and persistence operations are
// carried as declarative values and executed by the trusted host application.
class JavaScriptExtensionSession final : public mojom::UtilitySession {
 public:
  struct CreateResult {
    int32_t status;
    std::unique_ptr<JavaScriptExtensionSession> session;
  };

  static CreateResult Create(
      base::span<const uint8_t> init,
      mojo::PendingRemote<mojom::UtilitySessionClient> client = {});

  JavaScriptExtensionSession(const JavaScriptExtensionSession&) = delete;
  JavaScriptExtensionSession& operator=(const JavaScriptExtensionSession&) =
      delete;
  ~JavaScriptExtensionSession() override;

  // Completes after V8 has compiled, linked, and evaluated the supplied graph.
  void Start(base::OnceCallback<void(int32_t)> callback);

  void Send(uint64_t command_id,
            const std::vector<uint8_t>& command,
            SendCallback callback) override;
  void Acknowledge(uint64_t committed_sequence, uint32_t credits) override;
  void Cancel(uint64_t command_id) override;
  void Close(uint32_t reason, CloseCallback callback) override;

 private:
  class Impl;
  class Realm;
  struct ExecutionControl;

  JavaScriptExtensionSession(
      std::map<std::string, std::string> sources,
      std::string entry,
      mojo::PendingRemote<mojom::UtilitySessionClient> client,
      bool resolved);

  void OnInitialized(int32_t status);
  void RequestNextModule();
  void OnModuleSource(std::string url,
                      int32_t status,
                      const std::vector<uint8_t>& source);
  void OnModuleCompiled(std::pair<int32_t, std::vector<std::string>> result);
  void CompleteStart(int32_t status);

  void OnStarted(base::OnceCallback<void(int32_t)> callback, int32_t status);
  void OnExecuted(SendCallback callback,
                  std::pair<int32_t, std::vector<uint8_t>> result);
  void OnClosed(CloseCallback callback,
                std::pair<int32_t, std::vector<uint8_t>> result);

  std::unique_ptr<Impl> impl_;
  const std::string entry_;
  const bool resolved_;
  mojo::Remote<mojom::UtilitySessionClient> client_;
  std::deque<std::string> pending_modules_;
  std::set<std::string> requested_modules_;
  size_t source_bytes_ = 0;
  base::OnceCallback<void(int32_t)> start_callback_;
  base::OneShotTimer start_timeout_;
  bool started_ = false;
  bool closed_ = false;
  base::WeakPtrFactory<JavaScriptExtensionSession> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_UTILITY_JS_EXTENSION_SESSION_H_
