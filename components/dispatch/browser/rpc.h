// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_H_

#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

#include "base/check.h"
#include "base/containers/span.h"
#include "base/functional/bind.h"
#include "base/functional/callback.h"
#include "tuffite/components/dispatch/browser/renderer_request_context.h"
#include "tuffite/components/dispatch/browser/rpc_handler.h"
#include "tuffite/components/dispatch/common/status.h"

namespace tuffite::rpc {

using Reply = RpcHandler::Reply;
// A dispatcher consumes the reply only after recognizing the method.
using ServiceHandler =
    base::RepeatingCallback<bool(const dispatch::RendererRequestContext&,
                                 std::string_view,
                                 base::span<const uint8_t>,
                                 Reply*)>;
struct ServiceMethod {
  std::string_view name;
  uint32_t ordinal;
};
struct Service {
  std::string_view name;
  ServiceHandler dispatch;
  std::vector<ServiceMethod> methods;
};

class Router final {
 public:
  explicit Router(std::vector<Service>);
  ~Router();
  std::optional<std::string> ResolveMethod(std::string_view, uint32_t) const;
  bool Dispatch(const dispatch::RendererRequestContext&,
                std::string_view,
                base::span<const uint8_t>,
                Reply*) const;

 private:
  struct Entry {
    std::string name;
    std::string wire_name;
    ServiceHandler dispatch;
    std::vector<ServiceMethod> methods;
  };
  std::vector<Entry> services_;
};

// Error replies carry no wire payload, regardless of the method's result types.
template <typename... Values>
void Reject(base::OnceCallback<void(Status, Values...)>& callback,
            Status status) {
  CHECK(status != Status::kOk);
  std::move(callback).Run(status, Values{}...);
}

// Generated bindings provide only the official payload types and result fields.
// All validation, completion ownership and serialization live here.
template <typename Request, typename Response, auto... Fields>
struct Method {
  // Chromium may choose StructPtr or InlinedStructPtr for each payload.
  using RequestPtr = decltype(Request::New());
  template <auto Field>
  using Value = std::remove_cvref_t<decltype(std::declval<Response>().*Field)>;
  using Callback = base::OnceCallback<void(Status, Value<Fields>...)>;

  template <typename Interface, typename Handler>
  static bool Dispatch(std::shared_ptr<Interface> implementation,
                       const dispatch::RendererRequestContext& context,
                       base::span<const uint8_t> input,
                       Reply* reply,
                       Handler handler) {
    if (!context.is_valid()) {
      std::move(*reply).Run(static_cast<int32_t>(Status::kCancelled), {});
      return true;
    }
    Call(input, std::move(*reply),
         base::BindOnce(
             [](std::shared_ptr<Interface> implementation,
                dispatch::RendererRequestContext context, Handler handler,
                RequestPtr request, Callback callback) {
               auto retained = base::BindOnce(
                   [](std::shared_ptr<Interface>, Callback callback,
                      Status status, Value<Fields>... values) {
                     std::move(callback).Run(status, std::move(values)...);
                   },
                   implementation, std::move(callback));
               (implementation.get()->*handler)(context, std::move(request),
                                                std::move(retained));
             },
             std::move(implementation), context, handler));
    return true;
  }

  static void Call(base::span<const uint8_t> input,
                   Reply reply,
                   base::OnceCallback<void(RequestPtr, Callback)> handler) {
    RequestPtr request;
    if (!Request::Deserialize(input, &request) || !request) {
      std::move(reply).Run(static_cast<int32_t>(Status::kInvalidArgument), {});
      return;
    }
    auto completion = base::BindOnce(
        [](Reply reply, Status status, Value<Fields>... values) {
          std::vector<uint8_t> bytes;
          if (status == Status::kOk) {
            auto response = Response::New();
            ((response.get()->*Fields = std::move(values)), ...);
            bytes = Response::Serialize(&response);
          }
          std::move(reply).Run(static_cast<int32_t>(status), bytes);
        },
        std::move(reply));
    std::move(handler).Run(std::move(request), std::move(completion));
  }
};

template <typename Interface, typename Dispatcher>
Service BindService(std::string_view name,
                    std::shared_ptr<Interface> implementation,
                    Dispatcher dispatcher,
                    std::vector<ServiceMethod> methods = {}) {
  CHECK(implementation);
  return {name, base::BindRepeating(dispatcher, std::move(implementation)),
          std::move(methods)};
}

}  // namespace tuffite::rpc

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_H_
