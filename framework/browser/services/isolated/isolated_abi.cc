// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/isolated/isolated_abi.h"

#include <string>

#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/strings/string_number_conversions.h"
#include "tuffite/framework/app/abi_host_bridge.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/isolated/isolated_service_factory.h"

namespace tuffite {
namespace {
tuffite_status_t Apply(void* data,
                       tuffite_byte_slice_t request,
                       void* reply_data,
                       tuffite_isolated_reply_fn reply) {
  auto* bridge = static_cast<AbiHostBridge*>(data);
  if (!bridge || !reply || !reply_data || !request.data || request.len == 0 ||
      request.len > 64 * 1024 * 1024) {
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }
  const std::string json(reinterpret_cast<const char*>(request.data),
                         request.len);
  return bridge->PostHostCall(
      base::BindOnce(
          [](AbiHostBridge* bridge, std::string json, void* data,
             tuffite_isolated_reply_fn reply) {
            const auto context = bridge->browser_context();
            if (!context) {
              reply(data, TUFFITE_STATUS_INVALID_STATE, 0);
              return;
            }
            auto* service =
                IsolatedServiceFactory::GetForBrowserContext(context.get());
            auto value = base::JSONReader::ReadDict(json, base::JSON_PARSE_RFC);
            auto result = value ? service->Apply(*value) : std::nullopt;
            uint64_t generation = 0;
            const bool ok =
                result && base::StringToUint64(*result, &generation);
            reply(data,
                  ok ? TUFFITE_STATUS_OK : TUFFITE_STATUS_INVALID_ARGUMENT,
                  generation);
          },
          bridge, json, reply_data, reply),
      base::BindOnce(
          [](void* data, tuffite_isolated_reply_fn reply) {
            reply(data, TUFFITE_STATUS_INVALID_STATE, 0);
          },
          reply_data, reply));
}
}  // namespace
tuffite_isolated MakeIsolatedAbi(AbiHostBridge& bridge) {
  return {.header = tuffite_abi_header_for_size(sizeof(tuffite_isolated)),
          .user_data = &bridge,
          .apply = &Apply};
}
}  // namespace tuffite
