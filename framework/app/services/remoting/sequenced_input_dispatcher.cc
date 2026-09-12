// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/app/services/remoting/sequenced_input_dispatcher.h"

#include <algorithm>
#include <array>

#include "remoting/base/compound_buffer.h"
#include "remoting/proto/internal.pb.h"
#include "remoting/protocol/message_serialization.h"

namespace tuffite::remoting {
SequencedInputDispatcher::SequencedInputDispatcher(
    bool pointer_only,
    InputSequence* sequence,
    ::remoting::protocol::InputStub* input)
    : ChannelDispatcherBase(pointer_only ? "pointer-latest"
                                         : "input-sequenced"),
      pointer_only_(pointer_only),
      sequence_(sequence),
      input_(input) {}
SequencedInputDispatcher::~SequencedInputDispatcher() = default;

void SequencedInputDispatcher::OnIncomingMessage(
    std::unique_ptr<::remoting::CompoundBuffer> buffer) {
  if (buffer->total_bytes() <= 8 || buffer->total_bytes() > 64 * 1024) {
    return;
  }
  ::remoting::CompoundBuffer header;
  header.CopyFrom(*buffer, 0, 8);
  std::array<uint8_t, 8> bytes;
  header.CopyTo(bytes);
  uint64_t sequence = 0;
  for (size_t i = 0; i < bytes.size(); ++i) {
    sequence |= uint64_t{bytes[i]} << (i * 8);
  }
  if (sequence == 0) {
    return;
  }
  ::remoting::CompoundBuffer payload;
  payload.CopyFrom(*buffer, 8, buffer->total_bytes());
  payload.Lock();
  auto message =
      ::remoting::protocol::ParseMessage<::remoting::protocol::EventMessage>(
          &payload);
  if (!message) {
    return;
  }
  if (pointer_only_) {
    if (!message->has_mouse_event() || sequence <= sequence_->newest) {
      return;
    }
    const auto& mouse = message->mouse_event();
    if (mouse.has_button() || mouse.has_button_down() ||
        mouse.has_wheel_delta_x() || mouse.has_wheel_delta_y() ||
        !mouse.has_x() || !mouse.has_y()) {
      return;
    }
  } else {
    if (sequence <= sequence_->reliable) {
      return;
    }
    sequence_->reliable = sequence;
  }
  sequence_->newest = std::max(sequence_->newest, sequence);
  if (message->has_key_event()) {
    const auto& key = message->key_event();
    if (key.has_usb_keycode() && key.has_pressed()) {
      input_->InjectKeyEvent(key);
    }
  } else if (message->has_text_event()) {
    if (message->text_event().has_text()) {
      input_->InjectTextEvent(message->text_event());
    }
  } else if (message->has_mouse_event()) {
    input_->InjectMouseEvent(message->mouse_event());
  }
}
}  // namespace tuffite::remoting
