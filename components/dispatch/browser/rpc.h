// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_H_

#include <memory>
#include <string>
#include <string_view>
#include <type_traits>
#include <utility>
#include <vector>

#include "base/check.h"
#include "base/containers/span.h"
#include "base/functional/bind.h"
#include "base/functional/callback.h"
#include "tuffite/components/dispatch/browser/context.h"
#include "tuffite/components/dispatch/browser/rpc_handler.h"
#include "tuffite/components/dispatch/common/status.h"

namespace tuffite::rpc {

using Reply = RpcHandler::Reply;
// A dispatcher consumes the reply only after recognizing the method.
using ServiceHandler = base::RepeatingCallback<bool(const dispatch::Context&,
                                                    std::string_view,
                                                    base::span<const uint8_t>,
                                                    Reply*)>;
struct Service {
  std::string_view name;
  ServiceHandler dispatch;
};

class Router final {
 public:
  explicit Router(std::vector<Service>);
  ~Router();
  bool Dispatch(content::RenderFrameHost&,
                std::string_view,
                base::span<const uint8_t>,
                const base::UnguessableToken&,
                Reply*) const;

  bool Dispatch(const dispatch::Context&,
                std::string_view,
                base::span<const uint8_t>,
                Reply*) const;

 private:
  struct Entry {
    std::string name;
    ServiceHandler dispatch;
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
                       const dispatch::Context& context,
                       base::span<const uint8_t> input,
                       Reply* reply,
                       Handler handler) {
    if (!context.is_valid()) {
      std::move(*reply).Run(static_cast<int32_t>(Status::kCancelled), {});
      return true;
    }
    RequestPtr request;
    if (!Request::Deserialize(input, &request)) {
      std::move(*reply).Run(static_cast<int32_t>(Status::kInvalidArgument), {});
      return true;
    }
    auto completion = base::BindOnce(
        [](std::shared_ptr<Interface>, Reply reply, Status status,
           Value<Fields>... values) {
          std::vector<uint8_t> bytes;
          if (status == Status::kOk) {
            auto response = Response::New();
            ((response.get()->*Fields = std::move(values)), ...);
            bytes = Response::Serialize(&response);
          }
          std::move(reply).Run(static_cast<int32_t>(status), bytes);
        },
        implementation, std::move(*reply));
    (implementation.get()->*handler)(context, std::move(request),
                                     std::move(completion));
    return true;
  }
};

template <typename Interface, typename Dispatcher>
Service BindService(std::string_view name,
                    std::shared_ptr<Interface> implementation,
                    Dispatcher dispatcher) {
  CHECK(implementation);
  return {name, base::BindRepeating(dispatcher, std::move(implementation))};
}

}  // namespace tuffite::rpc

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_H_
