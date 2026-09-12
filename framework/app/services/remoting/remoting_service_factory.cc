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
  SessionResult Start(StartOptions) override { return {}; }
  EmptyResult ProcessSignal(uint64_t, Signal) override { return {}; }
  SignalsResult TakeSignals(uint64_t, uint64_t) override { return {}; }
  EmptyResult SendInput(uint64_t, InputEvent) override { return {}; }
  EmptyResult InjectInput(InputEvent) override { return {}; }
  void Capture(std::optional<int64_t>, SnapshotCallback callback) override {
    std::move(callback).Run(Status::kUnavailable, {});
  }
  EmptyResult Stop(uint64_t) override { return {}; }
  StatsResult GetStats(uint64_t) const override { return {}; }
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
