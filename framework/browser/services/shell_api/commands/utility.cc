// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/utility.h"

#include <utility>

#include "base/functional/bind.h"
#include "content/public/browser/render_frame_host.h"
#include "tuffite/framework/app/application_context.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/app/services/utility/utility_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"
#include "tuffite/framework/public/abi.h"

namespace tuffite::shell_api::utility {

using CancelCallback = generated::Utility::CancelCallback;
using CancelRequestPtr = generated::Utility::CancelRequestPtr;
using InvokeCallback = generated::Utility::InvokeCallback;
using InvokeRequestPtr = generated::Utility::InvokeRequestPtr;

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

void UtilityImpl::Invoke(const dispatch::Context& context,
                         InvokeRequestPtr request,
                         InvokeCallback callback) {
  if (!IsUnscopedAllowed(context, generated::Utility::kInvoke)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  if (!IsValidRequest(request->name, request->request_id)) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  auto& application =
      (context.is_application()
           ? *context.native_application
           : BrowserContext::GetFrom(context.frame()).application_context());
  if (!context.is_application() &&
      (application.GetService<ManifestService>()->IsApplicationInterface(
           request->name) ||
       !application.GetService<UtilityService>()
            ->ResolveUtilityInterface(request->name)
            .empty())) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  application.GetService<UtilityService>()->InvokeUtilityService(
      context.document, static_cast<uint64_t>(request->request_id),
      request->name,
      std::move(request->payload).value_or(std::vector<uint8_t>()),
      base::BindOnce(
          [](InvokeCallback callback, int32_t status,
             std::vector<uint8_t> response) {
            std::move(callback).Run(ToRpcStatus(status), std::move(response));
          },
          std::move(callback)));
}

void UtilityImpl::Cancel(const dispatch::Context& context,
                         CancelRequestPtr request,
                         CancelCallback callback) {
  if (!IsUnscopedAllowed(context, generated::Utility::kCancel)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  if (!IsValidRequest(request->name, request->request_id)) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  auto& application =
      (context.is_application()
           ? *context.native_application
           : BrowserContext::GetFrom(context.frame()).application_context());
  if (!context.is_application() &&
      (application.GetService<ManifestService>()->IsApplicationInterface(
           request->name) ||
       !application.GetService<UtilityService>()
            ->ResolveUtilityInterface(request->name)
            .empty())) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  application.GetService<UtilityService>()->CancelUtilityService(
      context.document, request->name,
      static_cast<uint64_t>(request->request_id));
  std::move(callback).Run(rpc::Status::kOk);
}

}  // namespace tuffite::shell_api::utility
