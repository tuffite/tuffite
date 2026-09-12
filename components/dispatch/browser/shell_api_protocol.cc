// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/shell_api_protocol.h"

#include "base/numerics/byte_conversions.h"
#include "base/numerics/safe_conversions.h"
#include "base/strings/string_util.h"

namespace tuffite::shell_api::generated {
namespace {

template <typename T, typename Decoder>
bool ReadNumber(base::span<const uint8_t>* input, T* value, Decoder decoder) {
  if (input->size() < sizeof(T)) {
    return false;
  }
  *value = decoder(input->first<sizeof(T)>());
  *input = input->subspan(sizeof(T));
  return true;
}

template <typename Bytes>
void AppendNumber(std::vector<uint8_t>* output, const Bytes& bytes) {
  output->insert(output->end(), bytes.begin(), bytes.end());
}

bool ReadSpan(base::span<const uint8_t>* input,
              base::span<const uint8_t>* value) {
  uint32_t length = 0;
  if (!ReadNumber(input, &length, base::U32FromLittleEndian) ||
      length > input->size()) {
    return false;
  }
  *value = input->first(length);
  *input = input->subspan(length);
  return true;
}

void AppendSpan(std::vector<uint8_t>* output, base::span<const uint8_t> value) {
  AppendNumber(output, base::U32ToLittleEndian(
                           base::checked_cast<uint32_t>(value.size())));
  output->insert(output->end(), value.begin(), value.end());
}

}  // namespace

bool DecodeValue(base::span<const uint8_t>* input, bool* value) {
  if (input->empty() || input->front() > 1) {
    return false;
  }
  *value = input->front() == 1;
  *input = input->subspan(1u);
  return true;
}
bool DecodeValue(base::span<const uint8_t>* input, double* value) {
  return ReadNumber(input, value, base::DoubleFromLittleEndian);
}
bool DecodeValue(base::span<const uint8_t>* input, int64_t* value) {
  return ReadNumber(input, value, base::I64FromLittleEndian);
}
bool DecodeValue(base::span<const uint8_t>* input, uint32_t* value) {
  return ReadNumber(input, value, base::U32FromLittleEndian);
}
bool DecodeValue(base::span<const uint8_t>* input, std::string* value) {
  base::span<const uint8_t> bytes;
  if (!ReadSpan(input, &bytes) || bytes.size() > kMaxCollectionLength) {
    return false;
  }
  value->assign(bytes.begin(), bytes.end());
  return base::IsStringUTF8(*value);
}
bool DecodeValue(base::span<const uint8_t>* input,
                 std::vector<uint8_t>* value) {
  base::span<const uint8_t> bytes;
  if (!ReadSpan(input, &bytes) || bytes.size() > kMaxCollectionLength) {
    return false;
  }
  value->assign(bytes.begin(), bytes.end());
  return true;
}
bool DecodeValue(base::span<const uint8_t>*, std::monostate*) {
  return true;
}

void EncodeValue(std::vector<uint8_t>* output, bool value) {
  output->push_back(value ? 1 : 0);
}
void EncodeValue(std::vector<uint8_t>* output, double value) {
  AppendNumber(output, base::DoubleToLittleEndian(value));
}
void EncodeValue(std::vector<uint8_t>* output, int64_t value) {
  AppendNumber(output, base::I64ToLittleEndian(value));
}
void EncodeValue(std::vector<uint8_t>* output, uint32_t value) {
  AppendNumber(output, base::U32ToLittleEndian(value));
}
void EncodeValue(std::vector<uint8_t>* output, const std::string& value) {
  AppendSpan(output, base::as_byte_span(value));
}
void EncodeValue(std::vector<uint8_t>* output,
                 const std::vector<uint8_t>& value) {
  AppendSpan(output, value);
}
void EncodeValue(std::vector<uint8_t>*, const std::monostate&) {}

}  // namespace tuffite::shell_api::generated
