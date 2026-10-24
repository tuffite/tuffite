// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <algorithm>
#include <deque>
#include <map>
#include <memory>
#include <optional>
#include <tuple>
#include <utility>

#include "base/functional/bind.h"
#include "base/functional/callback_helpers.h"
#include "base/memory/raw_ref.h"
#include "base/memory/weak_ptr.h"
#include "base/sequence_checker.h"
#include "base/task/sequenced_task_runner.h"
#include "base/time/time.h"
#include "base/timer/timer.h"
#include "mojo/public/cpp/bindings/receiver.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/framework/app/services/utility/utility_process_launcher.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/common/utility/utility_service.mojom.h"
#include "tuffite/framework/common/utility/utility_service_name.h"
#include "tuffite/framework/public/abi.h"

namespace tuffite {
namespace {

constexpr size_t kMaxQueuedInvocations = 64;
constexpr size_t kMaxConcurrentInvocations = 8;
constexpr size_t kMaxSessionsPerService = 16;
constexpr size_t kMaxCommandsPerSession = 64;
constexpr int kCrashLimit = 5;
constexpr base::TimeDelta kInitialRestartDelay = base::Milliseconds(100);
constexpr base::TimeDelta kMaximumRestartDelay = base::Seconds(5);
constexpr base::TimeDelta kCircuitOpenDuration = base::Seconds(30);
constexpr base::TimeDelta kIdleShutdownDelay = base::Seconds(30);
constexpr base::TimeDelta kInvocationTimeout = base::Seconds(30);

struct PendingInvocation {
  uint64_t request_id;
  std::vector<uint8_t> request;
  UtilityResponseCallback callback;
};

struct ActiveInvocation {
  UtilityResponseCallback callback;
  std::unique_ptr<base::OneShotTimer> timer;
};

struct PendingSessionOpen {
  uint64_t session_id;
  std::vector<uint8_t> init;
  uint32_t initial_event_credits;
  uint32_t max_event_bytes;
  UtilitySessionEventCallback event_callback;
  UtilitySessionClosedCallback closed_callback;
  base::OnceCallback<void(int32_t)> callback;
};

class UtilitySessionClient final : public mojom::UtilitySessionClient {
 public:
  UtilitySessionClient(
      uint64_t session_id,
      uint32_t initial_event_credits,
      uint32_t max_event_bytes,
      UtilitySessionEventCallback event_callback,
      UtilitySessionClosedCallback closed_callback,
      base::RepeatingClosure abort_callback,
      base::RepeatingCallback<void(std::string, UtilityResponseCallback)>
          resolver)
      : session_id_(session_id),
        available_credits_(initial_event_credits),
        max_credits_(initial_event_credits),
        max_event_bytes_(max_event_bytes),
        event_callback_(std::move(event_callback)),
        closed_callback_(std::move(closed_callback)),
        abort_callback_(std::move(abort_callback)),
        resolver_(std::move(resolver)),
        receiver_(this) {}

  mojo::PendingRemote<mojom::UtilitySessionClient> Bind() {
    return receiver_.BindNewPipeAndPassRemote();
  }

  void ResolveModule(const std::string& url,
                     ResolveModuleCallback callback) override {
    if (!resolver_ || ++module_requests_ > 64 || url.empty() ||
        url.size() > 512) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT, {});
      return;
    }
    resolver_.Run(url, base::BindOnce(
                           [](ResolveModuleCallback callback, int32_t status,
                              std::vector<uint8_t> source) {
                             std::move(callback).Run(status, source);
                           },
                           std::move(callback)));
  }

  bool Acknowledge(uint64_t committed_sequence, uint32_t credits) {
    if (committed_sequence < committed_sequence_ ||
        committed_sequence > last_sequence_) {
      return false;
    }
    const uint64_t outstanding = last_sequence_ - committed_sequence;
    if (available_credits_ + outstanding > max_credits_ ||
        credits > max_credits_ - available_credits_ - outstanding) {
      return false;
    }
    committed_sequence_ = committed_sequence;
    available_credits_ += credits;
    return true;
  }

  void Close(int32_t status, std::vector<uint8_t> summary) {
    receiver_.reset();
    if (closed_callback_) {
      std::move(closed_callback_)
          .Run(status, last_sequence_, std::move(summary));
    }
  }

  void OnEvent(uint64_t session_id,
               uint64_t sequence,
               uint32_t kind,
               const std::vector<uint8_t>& payload) override {
    if (session_id != session_id_ || kind == 0 ||
        sequence != last_sequence_ + 1 || available_credits_ == 0 ||
        payload.size() > max_event_bytes_ || !event_callback_) {
      Close(TUFFITE_STATUS_INVALID_ARGUMENT, {});
      base::SequencedTaskRunner::GetCurrentDefault()->PostTask(FROM_HERE,
                                                               abort_callback_);
      return;
    }
    --available_credits_;
    last_sequence_ = sequence;
    if (!event_callback_.Run(
            sequence, kind,
            std::vector<uint8_t>(payload.begin(), payload.end()))) {
      Close(TUFFITE_STATUS_INVALID_STATE, {});
      base::SequencedTaskRunner::GetCurrentDefault()->PostTask(FROM_HERE,
                                                               abort_callback_);
    }
  }

 private:
  const uint64_t session_id_;
  uint64_t last_sequence_ = 0;
  uint64_t committed_sequence_ = 0;
  uint32_t available_credits_;
  const uint32_t max_credits_;
  const uint32_t max_event_bytes_;
  UtilitySessionEventCallback event_callback_;
  UtilitySessionClosedCallback closed_callback_;
  base::RepeatingClosure abort_callback_;
  base::RepeatingCallback<void(std::string, UtilityResponseCallback)> resolver_;
  size_t module_requests_ = 0;
  mojo::Receiver<mojom::UtilitySessionClient> receiver_;
};

struct HostedUtilitySession {
  std::unique_ptr<UtilitySessionClient> client;
  mojo::Remote<mojom::UtilitySession> remote;
};

class UtilityServiceHost {
 public:
  UtilityServiceHost(std::string name,
                     UtilityProcessLauncher& launcher,
                     UtilityModuleResolver resolver,
                     std::vector<uint8_t> configuration)
      : name_(std::move(name)),
        launcher_(launcher),
        resolver_(std::move(resolver)),
        configuration_(std::move(configuration)) {}

  UtilityServiceHost(const UtilityServiceHost&) = delete;
  UtilityServiceHost& operator=(const UtilityServiceHost&) = delete;

  void Invoke(uint64_t request_id,
              std::vector<uint8_t> request,
              UtilityResponseCallback callback) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    const base::TimeTicks now = base::TimeTicks::Now();
    if (!circuit_until_.is_null()) {
      if (now < circuit_until_) {
        std::move(callback).Run(TUFFITE_STATUS_RESOURCE_EXHAUSTED, {});
        return;
      }
      circuit_until_ = base::TimeTicks();
      consecutive_crashes_ = 0;
    }
    if (active_.contains(request_id) ||
        std::ranges::any_of(pending_, [request_id](const auto& pending) {
          return pending.request_id == request_id;
        })) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT, {});
      return;
    }
    if (pending_.size() + active_.size() >= kMaxQueuedInvocations) {
      std::move(callback).Run(TUFFITE_STATUS_RESOURCE_EXHAUSTED, {});
      return;
    }
    idle_timer_.Stop();
    pending_.push_back(
        PendingInvocation{request_id, std::move(request), std::move(callback)});
    EnsureRunning();
  }

  void Cancel(uint64_t request_id) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    const auto queued =
        std::ranges::find(pending_, request_id, &PendingInvocation::request_id);
    if (queued != pending_.end()) {
      UtilityResponseCallback callback = std::move(queued->callback);
      pending_.erase(queued);
      std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE, {});
      return;
    }
    if (!active_.contains(request_id)) {
      return;
    }
    if (remote_) {
      remote_->Cancel(request_id);
    }
  }

  void CreateSession(uint64_t session_id,
                     std::vector<uint8_t> init,
                     uint32_t initial_event_credits,
                     uint32_t max_event_bytes,
                     UtilitySessionEventCallback event_callback,
                     UtilitySessionClosedCallback closed_callback,
                     base::OnceCallback<void(int32_t)> callback) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    const bool pending_id = std::ranges::any_of(
        pending_session_opens_, [session_id](const auto& pending) {
          return pending.session_id == session_id;
        });
    if (session_id == 0 || sessions_.contains(session_id) || pending_id ||
        sessions_.size() + pending_session_opens_.size() >=
            kMaxSessionsPerService ||
        initial_event_credits == 0 || initial_event_credits > 1024 ||
        max_event_bytes == 0 || max_event_bytes > 256 * 1024) {
      if (closed_callback) {
        std::move(closed_callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT, 0, {});
      }
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT);
      return;
    }
    idle_timer_.Stop();
    pending_session_opens_.push_back(
        PendingSessionOpen{session_id, std::move(init), initial_event_credits,
                           max_event_bytes, std::move(event_callback),
                           std::move(closed_callback), std::move(callback)});
    EnsureRunning();
    DispatchSessionOpens();
  }

  void DispatchSessionOpens() {
    if (state_ != State::kReady) {
      return;
    }
    std::deque<PendingSessionOpen> pending;
    pending.swap(pending_session_opens_);
    for (auto& request : pending) {
      StartSession(std::move(request));
    }
  }

  void StartSession(PendingSessionOpen request) {
    auto client = std::make_unique<UtilitySessionClient>(
        request.session_id, request.initial_event_credits,
        request.max_event_bytes, std::move(request.event_callback),
        std::move(request.closed_callback),
        base::BindRepeating(&UtilityServiceHost::AbortSession,
                            weak_factory_.GetWeakPtr(), request.session_id),
        base::BindRepeating(
            [](UtilityModuleResolver resolver, std::string name,
               std::string url, UtilityResponseCallback reply) {
              if (!resolver) {
                std::move(reply).Run(TUFFITE_STATUS_INVALID_STATE, {});
                return;
              }
              resolver.Run(std::move(name), std::move(url), std::move(reply));
            },
            resolver_, name_));
    auto client_remote = client->Bind();
    mojo::Remote<mojom::UtilitySession> session_remote;
    auto session_receiver = session_remote.BindNewPipeAndPassReceiver();
    session_remote.set_disconnect_handler(
        base::BindOnce(&UtilityServiceHost::OnSessionDisconnected,
                       weak_factory_.GetWeakPtr(), request.session_id));
    sessions_.emplace(
        request.session_id,
        HostedUtilitySession{std::move(client), std::move(session_remote)});
    session_open_callbacks_.emplace(request.session_id,
                                    std::move(request.callback));
    remote_->CreateSession(
        request.session_id, std::move(request.init),
        request.initial_event_credits, request.max_event_bytes,
        std::move(client_remote), std::move(session_receiver),
        base::BindOnce(&UtilityServiceHost::OnSessionOpened,
                       weak_factory_.GetWeakPtr(), request.session_id));
  }

  void SendSession(uint64_t session_id,
                   uint64_t command_id,
                   std::vector<uint8_t> command,
                   UtilityResponseCallback callback) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    if (state_ != State::kReady || !sessions_.contains(session_id) ||
        command_id == 0) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_HANDLE, {});
      return;
    }
    const auto key = std::pair(session_id, command_id);
    const size_t commands_for_session = std::ranges::count_if(
        session_commands_, [session_id](const auto& entry) {
          return entry.first.first == session_id;
        });
    if (session_commands_.contains(key)) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT, {});
      return;
    }
    if (commands_for_session >= kMaxCommandsPerSession) {
      std::move(callback).Run(TUFFITE_STATUS_RESOURCE_EXHAUSTED, {});
      return;
    }
    session_commands_.emplace(key, std::move(callback));
    sessions_.at(session_id)
        .remote->Send(command_id, std::move(command),
                      base::BindOnce(&UtilityServiceHost::OnSessionCommand,
                                     weak_factory_.GetWeakPtr(), key));
  }

  void AcknowledgeSession(uint64_t session_id,
                          uint64_t committed_sequence,
                          uint32_t credits) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    const auto found = sessions_.find(session_id);
    if (state_ == State::kReady && found != sessions_.end() &&
        found->second.client->Acknowledge(committed_sequence, credits)) {
      found->second.remote->Acknowledge(committed_sequence, credits);
    }
  }

  void CancelSessionCommand(uint64_t session_id, uint64_t command_id) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    if (state_ == State::kReady && sessions_.contains(session_id) &&
        session_commands_.contains(std::pair(session_id, command_id))) {
      sessions_.at(session_id).remote->Cancel(command_id);
    }
  }

  void CloseSession(uint64_t session_id,
                    uint32_t reason,
                    UtilityResponseCallback callback) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    if (state_ != State::kReady || !sessions_.contains(session_id)) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_HANDLE, {});
      return;
    }
    CancelSessionCommands(session_id, TUFFITE_STATUS_INVALID_STATE);
    sessions_.at(session_id)
        .remote->Close(reason,
                       base::BindOnce(&UtilityServiceHost::OnSessionClosed,
                                      weak_factory_.GetWeakPtr(), session_id,
                                      std::move(callback)));
  }

  void Shutdown() {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    weak_factory_.InvalidateWeakPtrs();
    idle_timer_.Stop();
    ResetRemote();
    state_ = State::kStopped;
    FailActiveInvocations(TUFFITE_STATUS_INVALID_STATE);
    FailPending(TUFFITE_STATUS_INVALID_STATE);
    FailPendingSessionOpens(TUFFITE_STATUS_INVALID_STATE);
    FailSessionOpenCallbacks(TUFFITE_STATUS_INVALID_STATE);
    FailSessions(TUFFITE_STATUS_INVALID_STATE);
  }

 private:
  // Stopped admits work and may launch immediately; WaitingToRestart holds
  // admitted work during crash backoff; Starting awaits the Mojo start reply;
  // Ready may dispatch work and is the only state eligible for idle shutdown.
  enum class State { kStopped, kWaitingToRestart, kStarting, kReady };

  void EnsureRunning() {
    if (state_ == State::kReady) {
      DispatchNext();
      return;
    }
    if (state_ != State::kStopped ||
        (pending_.empty() && pending_session_opens_.empty())) {
      return;
    }

    const base::TimeDelta delay =
        restart_not_before_.is_null()
            ? base::TimeDelta()
            : restart_not_before_ - base::TimeTicks::Now();
    if (delay.is_positive()) {
      state_ = State::kWaitingToRestart;
      base::SequencedTaskRunner::GetCurrentDefault()->PostDelayedTask(
          FROM_HERE,
          base::BindOnce(&UtilityServiceHost::RestartAfterDelay,
                         weak_factory_.GetWeakPtr()),
          delay);
      return;
    }
    StartProcess();
  }

  void RestartAfterDelay() {
    if (state_ != State::kWaitingToRestart) {
      return;
    }
    state_ = State::kStopped;
    EnsureRunning();
  }

  void StartProcess() {
    state_ = State::kStarting;
    remote_ = launcher_->Launch(name_);
    remote_.set_disconnect_handler(base::BindOnce(
        &UtilityServiceHost::OnDisconnected, weak_factory_.GetWeakPtr()));
    remote_->Start(name_, configuration_,
                   base::BindOnce(&UtilityServiceHost::OnStarted,
                                  weak_factory_.GetWeakPtr()));
  }

  void OnStarted(int32_t status) {
    if (status != TUFFITE_STATUS_OK) {
      ResetRemote();
      state_ = State::kStopped;
      FailPending(status);
      FailPendingSessionOpens(status);
      return;
    }
    state_ = State::kReady;
    DispatchNext();
    DispatchSessionOpens();
  }

  void OnSessionOpened(uint64_t session_id, int32_t status) {
    const auto callback_entry = session_open_callbacks_.find(session_id);
    if (callback_entry == session_open_callbacks_.end()) {
      return;
    }
    base::OnceCallback<void(int32_t)> callback =
        std::move(callback_entry->second);
    session_open_callbacks_.erase(callback_entry);
    if (status == TUFFITE_STATUS_OK && !sessions_.contains(session_id)) {
      status = TUFFITE_STATUS_INVALID_STATE;
    }
    if (status != TUFFITE_STATUS_OK) {
      auto found = sessions_.find(session_id);
      if (found != sessions_.end()) {
        found->second.client->Close(status, {});
        sessions_.erase(found);
      }
    }
    std::move(callback).Run(status);
    MaybeStopIdleProcess();
  }

  void OnSessionCommand(std::pair<uint64_t, uint64_t> key,
                        int32_t status,
                        const std::vector<uint8_t>& response) {
    const auto found = session_commands_.find(key);
    if (found == session_commands_.end()) {
      return;
    }
    UtilityResponseCallback callback = std::move(found->second);
    session_commands_.erase(found);
    std::move(callback).Run(
        status, std::vector<uint8_t>(response.begin(), response.end()));
  }

  void OnSessionClosed(uint64_t session_id,
                       UtilityResponseCallback callback,
                       int32_t status,
                       const std::vector<uint8_t>& summary) {
    const auto found = sessions_.find(session_id);
    if (found != sessions_.end()) {
      found->second.client->Close(
          status, std::vector<uint8_t>(summary.begin(), summary.end()));
      sessions_.erase(found);
    }
    std::move(callback).Run(
        status, std::vector<uint8_t>(summary.begin(), summary.end()));
    MaybeStopIdleProcess();
  }

  void OnSessionDisconnected(uint64_t session_id) {
    const auto found = sessions_.find(session_id);
    if (found == sessions_.end()) {
      return;
    }
    CancelSessionCommands(session_id, TUFFITE_STATUS_INTERNAL);
    found->second.client->Close(TUFFITE_STATUS_INTERNAL, {});
    sessions_.erase(found);
    const auto create_callback = session_open_callbacks_.find(session_id);
    if (create_callback != session_open_callbacks_.end()) {
      base::OnceCallback<void(int32_t)> callback =
          std::move(create_callback->second);
      session_open_callbacks_.erase(create_callback);
      std::move(callback).Run(TUFFITE_STATUS_INTERNAL);
    }
    MaybeStopIdleProcess();
  }

  void AbortSession(uint64_t session_id) {
    const auto found = sessions_.find(session_id);
    if (found == sessions_.end()) {
      return;
    }
    CancelSessionCommands(session_id, TUFFITE_STATUS_INVALID_STATE);
    sessions_.erase(found);
    MaybeStopIdleProcess();
  }

  void DispatchNext() {
    if (state_ != State::kReady) {
      return;
    }
    while (active_.size() < kMaxConcurrentInvocations && !pending_.empty()) {
      PendingInvocation pending = std::move(pending_.front());
      pending_.pop_front();
      const uint64_t request_id = pending.request_id;
      auto timer = std::make_unique<base::OneShotTimer>();
      timer->Start(FROM_HERE, kInvocationTimeout,
                   base::BindOnce(&UtilityServiceHost::OnInvocationTimeout,
                                  weak_factory_.GetWeakPtr(), request_id));
      active_.emplace(request_id, ActiveInvocation{std::move(pending.callback),
                                                   std::move(timer)});
      remote_->Invoke(request_id, std::move(pending.request),
                      base::BindOnce(&UtilityServiceHost::OnInvoked,
                                     weak_factory_.GetWeakPtr(), request_id));
    }
    if (pending_.empty() && active_.empty()) {
      MaybeStopIdleProcess();
    }
  }

  void OnInvoked(uint64_t request_id,
                 int32_t status,
                 const std::vector<uint8_t>& response) {
    const auto found = active_.find(request_id);
    if (found == active_.end()) {
      return;
    }
    UtilityResponseCallback callback = std::move(found->second.callback);
    active_.erase(found);
    consecutive_crashes_ = 0;
    restart_not_before_ = base::TimeTicks();
    std::move(callback).Run(status, response);
    DispatchNext();
  }

  void OnDisconnected() {
    ResetRemote();
    state_ = State::kStopped;

    // An invocation that reached the utility process has ambiguous completion
    // after a crash. It is failed exactly once and never replayed implicitly.
    std::map<uint64_t, ActiveInvocation> active;
    active.swap(active_);
    RecordCrashAndRestart();
    for (auto& [id, invocation] : active) {
      std::move(invocation.callback).Run(TUFFITE_STATUS_INTERNAL, {});
    }
    FailSessions(TUFFITE_STATUS_INTERNAL);
    FailPendingSessionOpens(TUFFITE_STATUS_INTERNAL);
    FailSessionOpenCallbacks(TUFFITE_STATUS_INTERNAL);
  }

  void OnInvocationTimeout(uint64_t request_id) {
    const auto found = active_.find(request_id);
    if (found == active_.end()) {
      return;
    }
    if (remote_) {
      remote_->Cancel(request_id);
    }
    UtilityResponseCallback callback = std::move(found->second.callback);
    active_.erase(found);
    ResetRemote();
    state_ = State::kStopped;
    FailActiveInvocations(TUFFITE_STATUS_INTERNAL);
    RecordCrashAndRestart();
    std::move(callback).Run(TUFFITE_STATUS_INTERNAL, {});
    FailSessions(TUFFITE_STATUS_INTERNAL);
    FailPendingSessionOpens(TUFFITE_STATUS_INTERNAL);
  }

  void RecordCrashAndRestart() {
    ++consecutive_crashes_;
    if (consecutive_crashes_ >= kCrashLimit) {
      circuit_until_ = base::TimeTicks::Now() + kCircuitOpenDuration;
      restart_not_before_ = base::TimeTicks();
      FailPending(TUFFITE_STATUS_RESOURCE_EXHAUSTED);
      return;
    }

    const int shift = std::min(consecutive_crashes_ - 1, 6);
    const base::TimeDelta delay =
        std::min(kInitialRestartDelay * (1 << shift), kMaximumRestartDelay);
    restart_not_before_ = base::TimeTicks::Now() + delay;
    EnsureRunning();
  }

  void StopIdleProcess() {
    if (state_ != State::kReady || !active_.empty() || !pending_.empty() ||
        !pending_session_opens_.empty() || !sessions_.empty()) {
      return;
    }
    ResetRemote();
    state_ = State::kStopped;
  }

  void MaybeStopIdleProcess() {
    if (state_ == State::kReady && active_.empty() && pending_.empty() &&
        pending_session_opens_.empty() && sessions_.empty()) {
      idle_timer_.Start(FROM_HERE, kIdleShutdownDelay,
                        base::BindOnce(&UtilityServiceHost::StopIdleProcess,
                                       weak_factory_.GetWeakPtr()));
    }
  }

  void ResetRemote() {
    if (remote_.is_bound()) {
      remote_.set_disconnect_handler(base::OnceClosure());
    }
    remote_.reset();
    idle_timer_.Stop();
  }

  void FailPending(int32_t status) {
    std::deque<PendingInvocation> pending;
    pending.swap(pending_);
    for (auto& invocation : pending) {
      std::move(invocation.callback).Run(status, {});
    }
  }

  void FailActiveInvocations(int32_t status) {
    std::map<uint64_t, ActiveInvocation> active;
    active.swap(active_);
    for (auto& [id, invocation] : active) {
      std::move(invocation.callback).Run(status, {});
    }
  }

  void FailPendingSessionOpens(int32_t status) {
    std::deque<PendingSessionOpen> pending;
    pending.swap(pending_session_opens_);
    for (auto& request : pending) {
      if (request.closed_callback) {
        std::move(request.closed_callback).Run(status, 0, {});
      }
      std::move(request.callback).Run(status);
    }
  }

  void FailSessionOpenCallbacks(int32_t status) {
    std::map<uint64_t, base::OnceCallback<void(int32_t)>> callbacks;
    callbacks.swap(session_open_callbacks_);
    for (auto& [id, callback] : callbacks) {
      std::move(callback).Run(status);
    }
  }

  void CancelSessionCommands(uint64_t session_id, int32_t status) {
    std::map<std::pair<uint64_t, uint64_t>, UtilityResponseCallback> pending;
    for (auto it = session_commands_.begin(); it != session_commands_.end();) {
      if (it->first.first == session_id) {
        pending.emplace(it->first, std::move(it->second));
        it = session_commands_.erase(it);
      } else {
        ++it;
      }
    }
    for (auto& [key, callback] : pending) {
      std::move(callback).Run(status, {});
    }
  }

  void FailSessions(int32_t status) {
    auto sessions = std::move(sessions_);
    sessions_.clear();
    for (auto& [id, client] : sessions) {
      CancelSessionCommands(id, status);
      client.client->Close(status, {});
    }
  }

  const std::string name_;
  const raw_ref<UtilityProcessLauncher> launcher_;
  UtilityModuleResolver resolver_;
  const std::vector<uint8_t> configuration_;
  State state_ = State::kStopped;
  int consecutive_crashes_ = 0;
  base::TimeTicks restart_not_before_;
  base::TimeTicks circuit_until_;
  std::deque<PendingInvocation> pending_;
  std::deque<PendingSessionOpen> pending_session_opens_;
  std::map<uint64_t, ActiveInvocation> active_;
  std::map<uint64_t, HostedUtilitySession> sessions_;
  std::map<uint64_t, base::OnceCallback<void(int32_t)>> session_open_callbacks_;
  std::map<std::pair<uint64_t, uint64_t>, UtilityResponseCallback>
      session_commands_;
  mojo::Remote<mojom::UtilityService> remote_;
  base::OneShotTimer idle_timer_;
  SEQUENCE_CHECKER(sequence_checker_);
  base::WeakPtrFactory<UtilityServiceHost> weak_factory_{this};
};

class UtilityServiceImpl final : public UtilityService {
 public:
  explicit UtilityServiceImpl(std::unique_ptr<UtilityProcessLauncher> launcher)
      : launcher_(std::move(launcher)) {}
  ~UtilityServiceImpl() override { Shutdown(); }
  void SetUtilityStartupConfigurations(
      std::map<std::string, std::vector<uint8_t>> configurations) override {
    if (!stopped_ && hosts_.empty()) {
      configurations_ = std::move(configurations);
    }
  }
  void SetUtilityModuleResolver(UtilityModuleResolver resolver) override {
    if (!stopped_) {
      resolver_ = std::move(resolver);
    }
  }
  void SetUtilityRegistrationChecker(
      UtilityRegistrationChecker checker) override {
    SetRegistrationChecker(std::move(checker));
  }
  bool IsRegisteredUtility(std::string_view name) const override {
    return !stopped_ && checker_ && checker_.Run(name);
  }
  void SetUtilityInterfaceResolver(UtilityInterfaceResolver resolver) override {
    if (!stopped_) {
      interface_resolver_ = std::move(resolver);
    }
  }
  std::string ResolveUtilityInterface(std::string_view name) const override {
    if (stopped_ || name.empty()) {
      return {};
    }
    if (interface_resolver_) {
      return interface_resolver_.Run(name);
    }
    return {};
  }
  void InvokeUtilityService(const base::UnguessableToken& owner,
                            uint64_t request_id,
                            std::string name,
                            std::vector<uint8_t> request,
                            UtilityResponseCallback callback) override {
    RequestKey key{owner, name, request_id};
    if (request_ids_.contains(key) || next_internal_request_id_ == 0) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT, {});
      return;
    }
    const uint64_t internal_id = next_internal_request_id_++;
    request_ids_.emplace(key, internal_id);
    Invoke(internal_id, std::move(name), std::move(request),
           base::BindOnce(&UtilityServiceImpl::CompleteInvocation,
                          weak_factory_.GetWeakPtr(), std::move(key),
                          std::move(callback)));
  }
  void CancelUtilityService(const base::UnguessableToken& owner,
                            std::string name,
                            uint64_t request_id) override {
    const auto found = request_ids_.find(RequestKey{owner, name, request_id});
    if (found != request_ids_.end()) {
      Cancel(name, found->second);
    }
  }
  void CancelUtilityOwner(const base::UnguessableToken& owner) override {
    std::vector<std::pair<std::string, uint64_t>> requests;
    for (const auto& [key, internal_id] : request_ids_) {
      if (std::get<0>(key) == owner) {
        requests.emplace_back(std::get<1>(key), internal_id);
      }
    }
    for (const auto& [name, internal_id] : requests) {
      Cancel(name, internal_id);
    }
  }
  void CreateUtilitySession(
      std::string name,
      uint64_t session_id,
      std::vector<uint8_t> init,
      uint32_t initial_event_credits,
      uint32_t max_event_bytes,
      UtilitySessionEventCallback event_callback,
      UtilitySessionClosedCallback closed_callback,
      base::OnceCallback<void(int32_t)> callback) override {
    CreateSession(std::move(name), session_id, std::move(init),
                  initial_event_credits, max_event_bytes,
                  std::move(event_callback), std::move(closed_callback),
                  std::move(callback));
  }
  void SendUtilitySession(std::string_view name,
                          uint64_t session_id,
                          uint64_t command_id,
                          std::vector<uint8_t> command,
                          UtilityResponseCallback callback) override {
    UtilityServiceHost* host = Find(name);
    if (!host) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_HANDLE, {});
      return;
    }
    host->SendSession(session_id, command_id, std::move(command),
                      std::move(callback));
  }
  void AcknowledgeUtilitySession(std::string_view name,
                                 uint64_t session_id,
                                 uint64_t committed_sequence,
                                 uint32_t credits) override {
    if (UtilityServiceHost* host = Find(name)) {
      host->AcknowledgeSession(session_id, committed_sequence, credits);
    }
  }
  void CancelUtilitySessionCommand(std::string_view name,
                                   uint64_t session_id,
                                   uint64_t command_id) override {
    if (UtilityServiceHost* host = Find(name)) {
      host->CancelSessionCommand(session_id, command_id);
    }
  }
  void CloseUtilitySession(std::string_view name,
                           uint64_t session_id,
                           uint32_t reason,
                           UtilityResponseCallback callback) override {
    UtilityServiceHost* host = Find(name);
    if (!host) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_HANDLE, {});
      return;
    }
    host->CloseSession(session_id, reason, std::move(callback));
  }

  using RequestKey = std::tuple<base::UnguessableToken, std::string, uint64_t>;

  void CompleteInvocation(RequestKey key,
                          UtilityResponseCallback callback,
                          int32_t status,
                          std::vector<uint8_t> response) {
    request_ids_.erase(key);
    std::move(callback).Run(status, std::move(response));
  }

  void SetRegistrationChecker(UtilityRegistrationChecker checker) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    if (!stopped_) {
      checker_ = std::move(checker);
    }
  }

  void Invoke(uint64_t request_id,
              std::string name,
              std::vector<uint8_t> request,
              UtilityResponseCallback callback) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    if (stopped_) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE, {});
      return;
    }
    if (!checker_ || !checker_.Run(name)) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT, {});
      return;
    }
    auto [entry, inserted] = hosts_.try_emplace(name, nullptr);
    if (inserted) {
      entry->second = std::make_unique<UtilityServiceHost>(
          name, *launcher_, resolver_, configurations_[name]);
    }
    entry->second->Invoke(request_id, std::move(request), std::move(callback));
  }

  void Cancel(std::string_view name, uint64_t request_id) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    const auto entry = hosts_.find(std::string(name));
    if (entry != hosts_.end()) {
      entry->second->Cancel(request_id);
    }
  }

  void CreateSession(std::string name,
                     uint64_t session_id,
                     std::vector<uint8_t> init,
                     uint32_t initial_event_credits,
                     uint32_t max_event_bytes,
                     UtilitySessionEventCallback event_callback,
                     UtilitySessionClosedCallback closed_callback,
                     base::OnceCallback<void(int32_t)> callback) {
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    if (!IsSessionAllowed(name)) {
      if (closed_callback) {
        std::move(closed_callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT, 0, {});
      }
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT);
      return;
    }
    auto [entry, inserted] = hosts_.try_emplace(name, nullptr);
    if (inserted) {
      entry->second = std::make_unique<UtilityServiceHost>(
          name, *launcher_, resolver_, configurations_[name]);
    }
    entry->second->CreateSession(
        session_id, std::move(init), initial_event_credits, max_event_bytes,
        std::move(event_callback), std::move(closed_callback),
        std::move(callback));
  }

  UtilityServiceHost* Find(std::string_view name) {
    const auto found = hosts_.find(std::string(name));
    return found == hosts_.end() ? nullptr : found->second.get();
  }

  bool IsSessionAllowed(std::string_view name) const {
    return !stopped_ && (name == kFrameworkJavaScriptExtensionService ||
                         (checker_ && checker_.Run(name)));
  }

  void Shutdown() override {
    if (stopped_) {
      return;
    }
    DCHECK_CALLED_ON_VALID_SEQUENCE(sequence_checker_);
    stopped_ = true;
    checker_.Reset();
    resolver_.Reset();
    auto hosts = std::exchange(hosts_, {});
    for (auto& entry : hosts) {
      entry.second->Shutdown();
    }
  }

 private:
  std::map<std::string, std::vector<uint8_t>> configurations_;
  std::unique_ptr<UtilityProcessLauncher> launcher_;
  bool stopped_ = false;
  uint64_t next_internal_request_id_ = 1;
  std::map<RequestKey, uint64_t> request_ids_;
  std::map<std::string, std::unique_ptr<UtilityServiceHost>> hosts_;
  UtilityRegistrationChecker checker_;
  UtilityInterfaceResolver interface_resolver_;
  UtilityModuleResolver resolver_;
  SEQUENCE_CHECKER(sequence_checker_);
  base::WeakPtrFactory<UtilityServiceImpl> weak_factory_{this};
};

}  // namespace

std::unique_ptr<UtilityService> CreateUtilityService(
    std::unique_ptr<UtilityProcessLauncher> launcher) {
  return std::make_unique<UtilityServiceImpl>(std::move(launcher));
}
}  // namespace tuffite
