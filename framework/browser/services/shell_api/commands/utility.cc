// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/utility.h"

#include <utility>

#include "base/functional/bind.h"
#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
#include "tuffite/framework/public/abi.h"

namespace tuffite::shell_api::utility {

using generated::utility::CancelCallback;
using generated::utility::CancelRequest;
using generated::utility::InvokeCallback;
using generated::utility::InvokeRequest;

namespace {

constexpr size_t kMaxUtilityServiceNameBytes = 128;

bool IsValidRequest(std::string_view name, int64_t request_id) {
  return !name.empty() && name.size() <= kMaxUtilityServiceNameBytes &&
         request_id > 0;
}

rpc::Status ToRpcStatus(tuffite_status_t status) {
  switch (status) {
    case TUFFITE_STATUS_OK:
      return rpc::Status::kOk;
    case TUFFITE_STATUS_INVALID_ARGUMENT:
    case TUFFITE_STATUS_INVALID_HANDLE:
      return rpc::Status::kInvalidArgument;
    case TUFFITE_STATUS_RESOURCE_EXHAUSTED:
      return rpc::Status::kResourceExhausted;
    case TUFFITE_STATUS_INCOMPATIBLE_ABI:
    case TUFFITE_STATUS_INVALID_STATE:
    case TUFFITE_STATUS_INTERNAL:
      return rpc::Status::kUnavailable;
  }
  return rpc::Status::kUnavailable;
}

}  // namespace

void Invoke(const ::tuffite::dispatch::Context& context,
            InvokeRequest&& request,
            InvokeCallback&& callback) {
  if (!IsValidRequest(request.name, request.request_id)) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto& application =
      (context.is_application()
           ? *context.native_application
           : BrowserContext::GetFrom(context.frame()).application_context());
  application.GetService<UtilityService>()->InvokeUtilityService(
      context.document, static_cast<uint64_t>(request.request_id), request.name,
      std::move(request.payload).value_or(std::vector<uint8_t>()),
      base::BindOnce(
          [](InvokeCallback callback, int32_t status,
             std::vector<uint8_t> response) {
            std::move(callback).Run(ToRpcStatus(status), std::move(response));
          },
          std::move(callback)));
}

void Cancel(const ::tuffite::dispatch::Context& context,
            CancelRequest&& request,
            CancelCallback&& callback) {
  if (!IsValidRequest(request.name, request.request_id)) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto& application =
      (context.is_application()
           ? *context.native_application
           : BrowserContext::GetFrom(context.frame()).application_context());
  application.GetService<UtilityService>()->CancelUtilityService(
      context.document, request.name,
      static_cast<uint64_t>(request.request_id));
  std::move(callback).Run(kStatusOk, {});
}

}  // namespace tuffite::shell_api::utility
