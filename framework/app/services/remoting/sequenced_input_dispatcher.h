// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_SEQUENCED_INPUT_DISPATCHER_H_
#define TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_SEQUENCED_INPUT_DISPATCHER_H_

#include <cstdint>
#include <memory>

#include "base/memory/raw_ptr.h"
#include "remoting/protocol/channel_dispatcher_base.h"
#include "remoting/protocol/input_stub.h"

namespace tuffite::remoting {

// Shared by both channels, on the remoting sequence. Reliable edges are never
// dropped because a newer lossy pointer packet arrived first.
struct InputSequence {
  uint64_t newest = 0;
  uint64_t reliable = 0;
};

class SequencedInputDispatcher final
    : public ::remoting::protocol::ChannelDispatcherBase {
 public:
  SequencedInputDispatcher(bool pointer_only,
                           InputSequence* sequence,
                           ::remoting::protocol::InputStub* input);
  ~SequencedInputDispatcher() override;

 private:
  void OnIncomingMessage(
      std::unique_ptr<::remoting::CompoundBuffer> buffer) override;
  const bool pointer_only_;
  const raw_ptr<InputSequence> sequence_;
  const raw_ptr<::remoting::protocol::InputStub> input_;
};
}  // namespace tuffite::remoting
#endif  // TUFFITE_FRAMEWORK_APP_SERVICES_REMOTING_SEQUENCED_INPUT_DISPATCHER_H_
