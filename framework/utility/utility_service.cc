// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/utility/utility_service.h"

#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

#include "base/check.h"
#include "base/containers/span.h"
#include "base/functional/bind.h"
#include "base/memory/ref_counted.h"
#include "base/task/sequenced_task_runner.h"
#include "mojo/public/cpp/bindings/receiver.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "mojo/public/cpp/bindings/unique_receiver_set.h"
#include "tuffite/framework/common/utility/utility_service_name.h"
#include "tuffite/framework/utility/js_extension_session.h"

namespace tuffite {
namespace {

std::optional<tuffite_application> g_utility_application;

class SessionContext : public base::RefCountedThreadSafe<SessionContext> {
 public:
  SessionContext(uint64_t session_id,
                 mojo::PendingRemote<mojom::UtilitySessionClient> client,
                 scoped_refptr<base::SequencedTaskRunner> task_runner)
      : session_id_(session_id),
        client_(std::move(client)),
        task_runner_(std::move(task_runner)) {}

  void PostEvent(uint64_t sequence,
                 uint32_t kind,
                 std::vector<uint8_t> payload) {
    task_runner_->PostTask(
        FROM_HERE,
        base::BindOnce(&SessionContext::Emit, base::WrapRefCounted(this),
                       sequence, kind, std::move(payload)));
  }

  void MarkClosed() {
    if (task_runner_->RunsTasksInCurrentSequence()) {
      closed_ = true;
      client_.reset();
    } else {
      task_runner_->PostTask(FROM_HERE,
                             base::BindOnce(&SessionContext::MarkClosed,
                                            base::WrapRefCounted(this)));
    }
  }

 private:
  friend class base::RefCountedThreadSafe<SessionContext>;
  ~SessionContext() = default;

  void Emit(uint64_t sequence, uint32_t kind, std::vector<uint8_t> payload) {
    if (!closed_ && client_) {
      client_->OnEvent(session_id_, sequence, kind, std::move(payload));
    }
  }

  const uint64_t session_id_;
  mojo::Remote<mojom::UtilitySessionClient> client_;
  scoped_refptr<base::SequencedTaskRunner> task_runner_;
  bool closed_ = false;
};

template <typename Callback>
struct ReplyContext {
  ReplyContext(Callback callback,
               scoped_refptr<base::SequencedTaskRunner> task_runner)
      : callback(std::move(callback)), task_runner(std::move(task_runner)) {}
  Callback callback;
  scoped_refptr<base::SequencedTaskRunner> task_runner;
};

struct SessionCloseContext {
  SessionCloseContext(mojom::UtilitySession::CloseCallback callback,
                      scoped_refptr<base::SequencedTaskRunner> task_runner,
                      scoped_refptr<SessionContext> session)
      : callback(std::move(callback)),
        task_runner(std::move(task_runner)),
        session(std::move(session)) {}
  mojom::UtilitySession::CloseCallback callback;
  scoped_refptr<base::SequencedTaskRunner> task_runner;
  scoped_refptr<SessionContext> session;
};

struct AbandonContext {
  explicit AbandonContext(scoped_refptr<SessionContext> session)
      : session(std::move(session)) {}
  scoped_refptr<SessionContext> session;
};

template <typename Callback>
tuffite_status_t CaptureReply(void* user_data,
                              tuffite_status_t status,
                              tuffite_byte_slice_t payload) {
  std::unique_ptr<ReplyContext<Callback>> context(
      static_cast<ReplyContext<Callback>*>(user_data));
  if (!context || (!payload.data && payload.len != 0)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  std::vector<uint8_t> response;
  if (payload.len) {
    // SAFETY: The native ABI producer keeps payload.len readable bytes alive
    // until this callback returns. Copy them before posting to another
    // sequence.
    const base::span<const uint8_t> bytes =
        UNSAFE_BUFFERS(base::span<const uint8_t>(payload.data, payload.len));
    response.assign(bytes.begin(), bytes.end());
  }
  context->task_runner->PostTask(
      FROM_HERE,
      base::BindOnce(
          [](std::unique_ptr<ReplyContext<Callback>> context,
             tuffite_status_t status, std::vector<uint8_t> response) {
            std::move(context->callback).Run(status, std::move(response));
          },
          std::move(context), status, std::move(response)));
  return TUFFITE_STATUS_OK;
}

tuffite_status_t CaptureSessionEvent(void* user_data,
                                     uint64_t session_id,
                                     uint64_t sequence,
                                     uint32_t kind,
                                     tuffite_byte_slice_t payload) {
  auto* context = static_cast<SessionContext*>(user_data);
  if (!context || (!payload.data && payload.len != 0)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  // SAFETY: The native ABI producer keeps payload.len readable bytes alive
  // until this callback returns. Copy them before posting to another sequence.
  const base::span<const uint8_t> bytes =
      payload.len
          ? UNSAFE_BUFFERS(base::span<const uint8_t>(payload.data, payload.len))
          : base::span<const uint8_t>();
  context->PostEvent(sequence, kind,
                     std::vector<uint8_t>(bytes.begin(), bytes.end()));
  return TUFFITE_STATUS_OK;
}

tuffite_status_t CaptureSessionCloseReply(void* user_data,
                                          tuffite_status_t status,
                                          tuffite_byte_slice_t payload) {
  std::unique_ptr<SessionCloseContext> context(
      static_cast<SessionCloseContext*>(user_data));
  if (!context || (!payload.data && payload.len != 0)) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  std::vector<uint8_t> response;
  if (payload.len) {
    // SAFETY: The native ABI producer keeps payload.len readable bytes alive
    // until this callback returns. Copy them before posting to another
    // sequence.
    const base::span<const uint8_t> bytes =
        UNSAFE_BUFFERS(base::span<const uint8_t>(payload.data, payload.len));
    response.assign(bytes.begin(), bytes.end());
  }
  context->task_runner->PostTask(
      FROM_HERE,
      base::BindOnce(
          [](std::unique_ptr<SessionCloseContext> context,
             tuffite_status_t status, std::vector<uint8_t> response) {
            context->session->MarkClosed();
            std::move(context->callback).Run(status, std::move(response));
          },
          std::move(context), status, std::move(response)));
  return TUFFITE_STATUS_OK;
}

tuffite_status_t CaptureAbandonReply(void* user_data,
                                     tuffite_status_t,
                                     tuffite_byte_slice_t) {
  std::unique_ptr<AbandonContext> context(
      static_cast<AbandonContext*>(user_data));
  if (!context) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  context->session->MarkClosed();
  return TUFFITE_STATUS_OK;
}

class UtilitySessionImpl final : public mojom::UtilitySession {
 public:
  UtilitySessionImpl(const tuffite_application& application,
                     tuffite_handle_t session,
                     scoped_refptr<SessionContext> context,
                     scoped_refptr<base::RefCountedData<bool>> service_stopping)
      : application_(application),
        session_(session),
        context_(std::move(context)),
        service_stopping_(std::move(service_stopping)) {}

  ~UtilitySessionImpl() override {
    if (service_stopping_->data) {
      context_->MarkClosed();
      return;
    }
    if (closed_ || !session_ || !application_.utility.session->close) {
      context_->MarkClosed();
      return;
    }
    closed_ = true;
    auto* context = new AbandonContext(context_);
    const tuffite_status_t status = application_.utility.session->close(
        session_, 1, context, &CaptureAbandonReply);
    if (status != TUFFITE_STATUS_OK) {
      std::unique_ptr<AbandonContext> failed(context);
      failed->session->MarkClosed();
    }
  }

  void Send(uint64_t command_id,
            const std::vector<uint8_t>& command,
            SendCallback callback) override {
    if (closed_ || !application_.utility.session->send) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE, {});
      return;
    }
    using Context = ReplyContext<SendCallback>;
    auto* context = new Context(std::move(callback),
                                base::SequencedTaskRunner::GetCurrentDefault());
    const tuffite_status_t status = application_.utility.session->send(
        session_, command_id, command.data(), command.size(), context,
        &CaptureReply<SendCallback>);
    if (status != TUFFITE_STATUS_OK) {
      std::unique_ptr<Context> failed(context);
      std::move(failed->callback).Run(status, {});
    }
  }

  void Acknowledge(uint64_t committed_sequence, uint32_t credits) override {
    if (!closed_ && application_.utility.session->acknowledge) {
      application_.utility.session->acknowledge(session_, committed_sequence,
                                                credits);
    }
  }

  void Cancel(uint64_t command_id) override {
    if (!closed_ && application_.utility.session->cancel) {
      application_.utility.session->cancel(session_, command_id);
    }
  }

  void Close(uint32_t reason, CloseCallback callback) override {
    if (closed_ || !application_.utility.session->close) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE, {});
      return;
    }
    closed_ = true;
    auto* context = new SessionCloseContext(
        std::move(callback), base::SequencedTaskRunner::GetCurrentDefault(),
        context_);
    const tuffite_status_t status = application_.utility.session->close(
        session_, reason, context, &CaptureSessionCloseReply);
    if (status != TUFFITE_STATUS_OK) {
      std::unique_ptr<SessionCloseContext> failed(context);
      failed->session->MarkClosed();
      std::move(failed->callback).Run(status, {});
    }
  }

 private:
  const tuffite_application application_;
  const tuffite_handle_t session_;
  scoped_refptr<SessionContext> context_;
  scoped_refptr<base::RefCountedData<bool>> service_stopping_;
  bool closed_ = false;
};

class UtilityServiceImpl final : public mojom::UtilityService {
 public:
  explicit UtilityServiceImpl(
      const tuffite_application& application,
      mojo::PendingReceiver<mojom::UtilityService> receiver)
      : application_(application), receiver_(this, std::move(receiver)) {}

  ~UtilityServiceImpl() override {
    service_stopping_->data = true;
    sessions_.Clear();
    if (service_ && application_.utility.stop) {
      application_.utility.stop(service_);
    }
  }

  void Start(const std::string& name,
             const std::vector<uint8_t>& configuration,
             StartCallback callback) override {
    if (configuration.size() > 64 * 1024) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT);
      return;
    }
    uint32_t runtime = TUFFITE_UTILITY_RUNTIME_RUST;
    if (application_.utility_runtime) {
      const auto status = application_.utility_runtime(
          reinterpret_cast<const uint8_t*>(name.data()), name.size(), &runtime);
      if (status != TUFFITE_STATUS_OK) {
        std::move(callback).Run(status);
        return;
      }
    } else if (name == kFrameworkJavaScriptExtensionService) {
      runtime = TUFFITE_UTILITY_RUNTIME_V8;
    }
    if (runtime != TUFFITE_UTILITY_RUNTIME_RUST &&
        runtime != TUFFITE_UTILITY_RUNTIME_V8) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT);
      return;
    }
    if (runtime == TUFFITE_UTILITY_RUNTIME_V8) {
      if (service_ || framework_js_extension_) {
        std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE);
        return;
      }
      framework_js_extension_ = true;
      std::move(callback).Run(TUFFITE_STATUS_OK);
      return;
    }
    if (service_ || !application_.utility.start) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE);
      return;
    }
    if (application_.utility_start_with_context) {
      std::move(callback).Run(application_.utility_start_with_context(
          reinterpret_cast<const uint8_t*>(name.data()), name.size(),
          {configuration.data(), configuration.size()}, &service_));
      return;
    }
    // Older providers cannot consume nonempty service startup configuration.
    if (!configuration.empty()) {
      std::move(callback).Run(TUFFITE_STATUS_INCOMPATIBLE_ABI);
      return;
    }
    std::move(callback).Run(application_.utility.start(
        reinterpret_cast<const uint8_t*>(name.data()), name.size(), &service_));
  }

  void Invoke(uint64_t request_id,
              const std::vector<uint8_t>& request,
              InvokeCallback callback) override {
    if (framework_js_extension_) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE, {});
      return;
    }
    if (!service_ || !application_.utility.invoke) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE, {});
      return;
    }
    using Context = ReplyContext<InvokeCallback>;
    auto* context = new Context(std::move(callback),
                                base::SequencedTaskRunner::GetCurrentDefault());
    const tuffite_status_t status = application_.utility.invoke(
        service_, request_id, request.data(), request.size(), context,
        &CaptureReply<InvokeCallback>);
    if (status != TUFFITE_STATUS_OK) {
      std::unique_ptr<Context> failed(context);
      std::move(failed->callback).Run(status, {});
    }
  }

  void CreateSession(
      uint64_t session_id,
      const std::vector<uint8_t>& init,
      uint32_t initial_event_credits,
      uint32_t max_event_bytes,
      mojo::PendingRemote<mojom::UtilitySessionClient> client,
      mojo::PendingReceiver<mojom::UtilitySession> session_receiver,
      CreateSessionCallback callback) override {
    if (framework_js_extension_) {
      if (session_id == 0 || !client.is_valid() ||
          !session_receiver.is_valid() || initial_event_credits == 0 ||
          max_event_bytes == 0) {
        std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT);
        return;
      }
      JavaScriptExtensionSession::CreateResult result =
          JavaScriptExtensionSession::Create(init, std::move(client));
      if (!result.session) {
        std::move(callback).Run(result.status);
        return;
      }
      JavaScriptExtensionSession* session = result.session.get();
      sessions_.Add(std::move(result.session), std::move(session_receiver));
      session->Start(std::move(callback));
      return;
    }
    if (!service_ || !application_.utility.session->create || session_id == 0 ||
        !client.is_valid() || !session_receiver.is_valid()) {
      std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE);
      return;
    }
    auto context = base::MakeRefCounted<SessionContext>(
        session_id, std::move(client),
        base::SequencedTaskRunner::GetCurrentDefault());
    tuffite_handle_t session = 0;
    const tuffite_status_t status = application_.utility.session->create(
        service_, session_id, init.data(), init.size(), initial_event_credits,
        max_event_bytes, context.get(), &CaptureSessionEvent, &session);
    if (status == TUFFITE_STATUS_OK && session != 0) {
      sessions_.Add(std::make_unique<UtilitySessionImpl>(
                        application_, session, context, service_stopping_),
                    std::move(session_receiver));
    } else {
      context->MarkClosed();
    }
    std::move(callback).Run(status);
  }

  void Cancel(uint64_t request_id) override {
    if (framework_js_extension_) {
      return;
    }
    if (service_ && application_.utility.cancel) {
      application_.utility.cancel(service_, request_id);
    }
  }

 private:
  const tuffite_application application_;
  mojo::Receiver<mojom::UtilityService> receiver_;
  tuffite_handle_t service_ = 0;
  scoped_refptr<base::RefCountedData<bool>> service_stopping_ =
      base::MakeRefCounted<base::RefCountedData<bool>>(false);
  mojo::UniqueReceiverSet<mojom::UtilitySession> sessions_;
  bool framework_js_extension_ = false;
};

}  // namespace

UtilityApplicationContext::UtilityApplicationContext(
    const tuffite_application& application) {
  CHECK(!g_utility_application.has_value());
  g_utility_application = application;
}

UtilityApplicationContext::~UtilityApplicationContext() {
  CHECK(g_utility_application.has_value());
  g_utility_application.reset();
}

std::unique_ptr<mojom::UtilityService> RunUtilityService(
    mojo::PendingReceiver<mojom::UtilityService> receiver) {
  if (!g_utility_application.has_value()) {
    return nullptr;
  }
  return std::make_unique<UtilityServiceImpl>(*g_utility_application,
                                              std::move(receiver));
}

}  // namespace tuffite
