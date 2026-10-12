// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/services/remoting/remoting_service_factory.h"

#include "tuffite/buildflags.h"
#include "tuffite/framework/app/application_context.h"

#if !BUILDFLAG(TUFFITE_DISABLE_REMOTING)
#include "tuffite/framework/app/services/remoting/remoting_service_impl.h"
#endif

namespace tuffite::remoting {

namespace {

class UnavailableRemotingService final : public RemotingService {
 public:
  CapabilitiesResult GetCapabilities() const override {
    return {.status = Status::kOk, .value = {}};
  }
  void Start(StartOptions, SessionCallback callback) override {
    std::move(callback).Run({});
  }
  void ProcessSignal(uint64_t, Signal, EmptyCallback callback) override {
    std::move(callback).Run({});
  }
  void TakeSignals(uint64_t, uint64_t, SignalsCallback callback) override {
    std::move(callback).Run({});
  }
  void SendInput(uint64_t, InputEvent, EmptyCallback callback) override {
    std::move(callback).Run({});
  }
  void InjectInput(InputEvent, EmptyCallback callback) override {
    std::move(callback).Run({});
  }
  void Capture(std::optional<int64_t>, SnapshotCallback callback) override {
    std::move(callback).Run(Status::kUnavailable, {});
  }
  void Stop(uint64_t, EmptyCallback callback) override {
    std::move(callback).Run({});
  }
  void GetStats(uint64_t, StatsCallback callback) const override {
    std::move(callback).Run({});
  }
};

}  // namespace

RemotingServiceFactory* RemotingServiceFactory::Get() {
  static base::NoDestructor<RemotingServiceFactory> instance;
  return instance.get();
}

RemotingServiceFactory::RemotingServiceFactory()
    : ApplicationKeyedServiceFactory<RemotingService>("TuffiteRemoting") {}

std::unique_ptr<RemotingService>
RemotingServiceFactory::BuildServiceInstanceForApplication(
    ApplicationContext&) const {
#if !BUILDFLAG(TUFFITE_DISABLE_REMOTING)
  return CreateRemotingService();
#else
  return std::make_unique<UnavailableRemotingService>();
#endif
}

}  // namespace tuffite::remoting
