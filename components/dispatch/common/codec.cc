// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/common/codec.h"

#include <algorithm>
#include <string_view>

#include "base/check.h"
#include "base/check_op.h"
#include "base/numerics/byte_conversions.h"
#include "base/numerics/safe_conversions.h"

namespace tuffite::rpc {
namespace {
constexpr size_t kRequestHeaderSize = 36;
constexpr size_t kResponseHeaderSize = 20;
constexpr size_t kCancelHeaderSize = 16;
constexpr size_t kRequestIdOffset = 8;
constexpr std::string_view kFrameMagic = "CERM";

enum class FrameKind : uint16_t {
  kRequest = 1,
  kResponse = 2,
  kCancel = 3,
};

// Every frame starts with CERM, kind:u16, flags/status:u16, request_id:u64.
// Request:
// command_id:u64, schema_fingerprint:u64, payload_length:u32, payload.
// Response: payload_length:u32, payload. Cancellation ends after request_id.
template <typename T, typename Decoder>
T Read(base::span<const uint8_t> bytes, size_t offset, Decoder decoder) {
  return decoder(bytes.subspan(offset).first<sizeof(T)>());
}
template <typename Bytes>
void Append(std::vector<uint8_t>* output, const Bytes& bytes) {
  output->insert(output->end(), bytes.begin(), bytes.end());
}
bool HasHeader(base::span<const uint8_t> bytes, FrameKind kind, size_t length) {
  return bytes.size() >= length &&
         std::equal(kFrameMagic.begin(), kFrameMagic.end(), bytes.begin()) &&
         Read<uint16_t>(bytes, 4, base::U16FromLittleEndian) ==
             static_cast<uint16_t>(kind);
}
std::vector<uint8_t> MakeHeader(FrameKind kind, uint16_t flags, uint64_t id) {
  std::vector<uint8_t> bytes(kFrameMagic.begin(), kFrameMagic.end());
  Append(&bytes, base::U16ToLittleEndian(static_cast<uint16_t>(kind)));
  Append(&bytes, base::U16ToLittleEndian(flags));
  Append(&bytes, base::U64ToLittleEndian(id));
  return bytes;
}
void WriteRequestId(std::vector<uint8_t>* bytes, uint64_t request_id) {
  const auto encoded = base::U64ToLittleEndian(request_id);
  std::copy(encoded.begin(), encoded.end(), bytes->begin() + kRequestIdOffset);
}
}  // namespace

std::optional<Request> DecodeRequest(base::span<const uint8_t> bytes) {
  if (!HasHeader(bytes, FrameKind::kRequest, kRequestHeaderSize) ||
      Read<uint16_t>(bytes, 6, base::U16FromLittleEndian) != 0 ||
      bytes.size() - kRequestHeaderSize > kMaxPayloadBytes ||
      Read<uint32_t>(bytes, 32, base::U32FromLittleEndian) !=
          bytes.size() - kRequestHeaderSize) {
    return std::nullopt;
  }
  return Request{
      Read<uint64_t>(bytes, 8, base::U64FromLittleEndian),
      Read<uint64_t>(bytes, 16, base::U64FromLittleEndian),
      Read<uint64_t>(bytes, 24, base::U64FromLittleEndian),
      std::vector<uint8_t>(bytes.begin() + kRequestHeaderSize, bytes.end())};
}
std::optional<Response> DecodeResponse(base::span<const uint8_t> bytes) {
  if (!HasHeader(bytes, FrameKind::kResponse, kResponseHeaderSize) ||
      bytes.size() - kResponseHeaderSize > kMaxPayloadBytes ||
      Read<uint32_t>(bytes, 16, base::U32FromLittleEndian) !=
          bytes.size() - kResponseHeaderSize) {
    return std::nullopt;
  }
  return Response{
      Read<uint64_t>(bytes, 8, base::U64FromLittleEndian),
      Read<uint16_t>(bytes, 6, base::U16FromLittleEndian),
      std::vector<uint8_t>(bytes.begin() + kResponseHeaderSize, bytes.end())};
}
std::optional<Cancel> DecodeCancel(base::span<const uint8_t> bytes) {
  if (!HasHeader(bytes, FrameKind::kCancel, kCancelHeaderSize) ||
      bytes.size() != kCancelHeaderSize ||
      Read<uint16_t>(bytes, 6, base::U16FromLittleEndian) != 0) {
    return std::nullopt;
  }
  return Cancel{Read<uint64_t>(bytes, 8, base::U64FromLittleEndian)};
}
std::vector<uint8_t> EncodeRequest(const Request& request) {
  CHECK_LE(request.payload.size(), kMaxPayloadBytes);
  auto bytes = MakeHeader(FrameKind::kRequest, 0, request.request_id);
  Append(&bytes, base::U64ToLittleEndian(request.command_id));
  Append(&bytes, base::U64ToLittleEndian(request.schema_fingerprint));
  Append(&bytes, base::U32ToLittleEndian(
                     base::checked_cast<uint32_t>(request.payload.size())));
  bytes.insert(bytes.end(), request.payload.begin(), request.payload.end());
  return bytes;
}
std::vector<uint8_t> EncodeResponse(const Response& response) {
  CHECK_LE(response.payload.size(), kMaxPayloadBytes);
  auto bytes =
      MakeHeader(FrameKind::kResponse, response.status, response.request_id);
  Append(&bytes, base::U32ToLittleEndian(
                     base::checked_cast<uint32_t>(response.payload.size())));
  bytes.insert(bytes.end(), response.payload.begin(), response.payload.end());
  return bytes;
}
std::vector<uint8_t> EncodeCancel(const Cancel& cancel) {
  return MakeHeader(FrameKind::kCancel, 0, cancel.request_id);
}
std::optional<uint64_t> ReplaceRequestId(std::vector<uint8_t>* bytes,
                                         uint64_t request_id) {
  const auto request = bytes ? DecodeRequest(*bytes) : std::nullopt;
  if (!request) {
    return std::nullopt;
  }
  WriteRequestId(bytes, request_id);
  return request->request_id;
}
bool ReplaceResponseRequestId(std::vector<uint8_t>* bytes,
                              uint64_t request_id) {
  if (!bytes || !DecodeResponse(*bytes)) {
    return false;
  }
  WriteRequestId(bytes, request_id);
  return true;
}
}  // namespace tuffite::rpc
