// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_COMMON_CODEC_H_
#define TUFFITE_COMPONENTS_DISPATCH_COMMON_CODEC_H_

#include <optional>

#include "base/containers/span.h"
#include "tuffite/components/dispatch/common/message.h"
namespace tuffite::rpc {
std::optional<Request> DecodeRequest(base::span<const uint8_t>);
std::optional<Response> DecodeResponse(base::span<const uint8_t>);
std::optional<Cancel> DecodeCancel(base::span<const uint8_t>);
std::vector<uint8_t> EncodeRequest(const Request&);
std::vector<uint8_t> EncodeResponse(const Response&);
std::vector<uint8_t> EncodeCancel(const Cancel&);
// Validate the full frame before replacing its correlation id in place.
std::optional<uint64_t> ReplaceRequestId(std::vector<uint8_t>*, uint64_t);
bool ReplaceResponseRequestId(std::vector<uint8_t>*, uint64_t);
}  // namespace tuffite::rpc

#endif  // TUFFITE_COMPONENTS_DISPATCH_COMMON_CODEC_H_
