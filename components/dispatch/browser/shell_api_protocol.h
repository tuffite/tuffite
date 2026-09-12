// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_SHELL_API_PROTOCOL_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_SHELL_API_PROTOCOL_H_

#include <stddef.h>
#include <stdint.h>

#include <limits>
#include <optional>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

#include "base/check.h"
#include "base/containers/span.h"
#include "base/functional/bind.h"
#include "base/functional/callback.h"
#include "base/numerics/safe_conversions.h"
#include "tuffite/components/dispatch/browser/context.h"
#include "tuffite/components/dispatch/browser/rpc_route.h"
#include "tuffite/components/dispatch/common/status.h"

namespace tuffite::shell_api::generated {

inline constexpr size_t kMaxCollectionLength = 16 * 1024 * 1024;

template <typename RequestType, typename ResultType>
class Method {
 public:
  using Request = RequestType;
  using Result = ResultType;
  using Callback = base::OnceCallback<void(rpc::Status, Result)>;
  using Handler = base::RepeatingCallback<
      void(const ::tuffite::dispatch::Context&, Request&&, Callback&&)>;
  using ContextFunction = void (*)(const ::tuffite::dispatch::Context&,
                                   Request&&,
                                   Callback&&);
  using Function = void (*)(Request&&, Callback&&);
  using Binder = rpc::Handler (*)(Handler);

  constexpr Method(std::string_view path, Binder binder)
      : path_(path), binder_(binder) {}

  rpc::Route Bind(ContextFunction implementation) const {
    return Bind(base::BindRepeating(implementation));
  }
  rpc::Route Bind(Function implementation) const {
    return Bind(base::BindRepeating(
        [](Function implementation, const ::tuffite::dispatch::Context&,
           Request&& request, Callback&& callback) {
          implementation(std::move(request), std::move(callback));
        },
        implementation));
  }
  rpc::Route Bind(Handler implementation) const {
    return {path_, binder_(std::move(implementation))};
  }

  constexpr std::string_view path() const { return path_; }

 private:
  std::string_view path_;
  Binder binder_;
};

bool DecodeValue(base::span<const uint8_t>* input, bool* value);
bool DecodeValue(base::span<const uint8_t>* input, double* value);
bool DecodeValue(base::span<const uint8_t>* input, int64_t* value);
bool DecodeValue(base::span<const uint8_t>* input, uint32_t* value);
bool DecodeValue(base::span<const uint8_t>* input, std::string* value);
bool DecodeValue(base::span<const uint8_t>* input, std::vector<uint8_t>* value);
bool DecodeValue(base::span<const uint8_t>* input, std::monostate*);

void EncodeValue(std::vector<uint8_t>* output, bool value);
void EncodeValue(std::vector<uint8_t>* output, double value);
void EncodeValue(std::vector<uint8_t>* output, int64_t value);
void EncodeValue(std::vector<uint8_t>* output, uint32_t value);
void EncodeValue(std::vector<uint8_t>* output, const std::string& value);
void EncodeValue(std::vector<uint8_t>* output,
                 const std::vector<uint8_t>& value);
void EncodeValue(std::vector<uint8_t>* output, const std::monostate&);

template <typename T>
bool DecodeValue(base::span<const uint8_t>* input, std::optional<T>* value);
template <typename T>
bool DecodeValue(base::span<const uint8_t>* input, std::vector<T>* value);
template <typename T>
void EncodeValue(std::vector<uint8_t>* output, const std::optional<T>& value);
template <typename T>
void EncodeValue(std::vector<uint8_t>* output, const std::vector<T>& value);

template <typename T>
bool DecodeValue(base::span<const uint8_t>* input, std::optional<T>* value) {
  if (input->empty() || input->front() > 1) {
    return false;
  }
  const bool present = input->front() == 1;
  *input = input->subspan(1u);
  if (!present) {
    value->reset();
    return true;
  }
  T decoded{};
  if (!DecodeValue(input, &decoded)) {
    return false;
  }
  *value = std::move(decoded);
  return true;
}

template <typename T>
bool DecodeValue(base::span<const uint8_t>* input, std::vector<T>* value) {
  uint32_t length = 0;
  if (!DecodeValue(input, &length) || length > kMaxCollectionLength) {
    return false;
  }
  value->clear();
  value->reserve(length);
  for (uint32_t index = 0; index < length; ++index) {
    T item{};
    if (!DecodeValue(input, &item)) {
      return false;
    }
    value->push_back(std::move(item));
  }
  return true;
}

template <typename T>
void EncodeValue(std::vector<uint8_t>* output, const std::optional<T>& value) {
  output->push_back(value.has_value() ? 1 : 0);
  if (value) {
    EncodeValue(output, *value);
  }
}

template <typename T>
void EncodeValue(std::vector<uint8_t>* output, const std::vector<T>& value) {
  CHECK_LE(value.size(), std::numeric_limits<uint32_t>::max());
  EncodeValue(output, base::checked_cast<uint32_t>(value.size()));
  for (const auto& item : value) {
    EncodeValue(output, item);
  }
}

}  // namespace tuffite::shell_api::generated

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_SHELL_API_PROTOCOL_H_
