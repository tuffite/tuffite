// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_APP_ABI_APPLICATION_H_
#define TUFFITE_FRAMEWORK_APP_ABI_APPLICATION_H_

#include <atomic>
#include <memory>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

#include "base/containers/span.h"
#include "base/functional/callback.h"
#include "base/memory/scoped_refptr.h"
#include "base/memory/weak_ptr.h"
#include "base/sequence_checker.h"
#include "base/synchronization/lock.h"
#include "base/synchronization/waitable_event.h"
#include "base/task/sequenced_task_runner.h"
#include "base/timer/timer.h"
#include "base/unguessable_token.h"
#include "tuffite/components/dispatch/browser/host_dispatch.h"
#include "tuffite/framework/public/c_api/application.h"
#include "tuffite/framework/public/c_api/host.h"

namespace tuffite {
class ApplicationContext;
class UtilityService;
}  // namespace tuffite

namespace tuffite::abi {

// Owns the native-to-Rust connection and request correlation. No service
// ownership.
class Application final : public HostDispatch {
 public:
  using DispatchCallback = base::OnceCallback<void(bool, std::vector<uint8_t>)>;
  using ApplicationEvent =
      base::RepeatingCallback<void(std::string, std::vector<uint8_t>)>;

  static std::unique_ptr<Application> Create(
      const tuffite_application* application);
  Application(const Application&) = delete;
  Application& operator=(const Application&) = delete;
  ~Application() override;

  const tuffite_application& table() const { return application_; }

  bool Start(const tuffite_host& host, ApplicationContext& context);
  bool StartProfile(Application& parent,
                    const tuffite_host&,
                    ApplicationContext&,
                    const std::string& profile_id,
                    UtilityService&);
  tuffite_handle_t runtime() const { return runtime_.load(); }
  void StopAsync(base::OnceClosure complete);
  void BindBrowserTaskRunner();
  void SetApplicationEvent(ApplicationEvent event) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(browser_sequence_checker_);
    application_event_ = std::move(event);
  }
  void InitializeBrowser(base::OnceCallback<void(bool)> callback);
  bool Dispatch(std::vector<uint8_t>,
                DispatchCallback,
                const base::UnguessableToken&) override;
  void CancelDocument(const base::UnguessableToken&) override;
  void Quiesce();
  void Stop();

  // May be called on Runtime workers; copies borrowed payloads before
  // returning.
  tuffite_status_t OnEvent(tuffite_handle_t runtime,
                           tuffite_runtime_event_kind_t kind,
                           tuffite_byte_slice_t payload);

  bool initialization_failed() const { return initialization_failed_; }
  bool is_running() const { return runtime_.load() != 0; }

 private:
  explicit Application(const tuffite_application& application);
  static bool ReadResponseRequestId(base::span<const uint8_t>, uint64_t*);
  void CompletePending(uint64_t, bool, std::vector<uint8_t>);
  void FailPending();
  void PublishApplicationEvent(std::string, std::vector<uint8_t>);
  const tuffite_application application_;
  raw_ptr<Application> parent_ = nullptr;
  std::atomic<tuffite_handle_t> runtime_{0};
  ApplicationEvent application_event_;
  bool browser_initialized_ = false;
  bool initialization_failed_ = false;
  bool accepting_requests_ = true;
  bool stopped_ = false;
  base::WaitableEvent stop_complete_{
      base::WaitableEvent::ResetPolicy::MANUAL,
      base::WaitableEvent::InitialState::SIGNALED};
  struct PendingRequest {
    DispatchCallback callback;
    uint64_t caller_id;
    base::UnguessableToken document;
    std::unique_ptr<base::OneShotTimer> timeout;
  };
  std::unordered_map<uint64_t, PendingRequest> pending_;
  uint64_t next_request_id_ = 1;
  // Publish the UI destination atomically to Runtime event producers.
  base::Lock event_lock_;
  bool accepting_events_ = false;  // Protected by event_lock_.
  scoped_refptr<base::SequencedTaskRunner> browser_task_runner_;
  SEQUENCE_CHECKER(browser_sequence_checker_);
  base::WeakPtr<Application> weak_this_;
  base::WeakPtrFactory<Application> weak_factory_{this};
};

}  // namespace tuffite::abi

#endif  // TUFFITE_FRAMEWORK_APP_ABI_APPLICATION_H_
