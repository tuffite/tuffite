// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_ROUTE_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_ROUTE_H_

#include <stdint.h>

#include <string_view>

#include "base/containers/span.h"
#include "tuffite/components/dispatch/browser/context.h"
#include "tuffite/components/dispatch/browser/rpc_handler.h"

namespace tuffite::rpc {

using Reply = RpcHandler::Reply;
using Handler = base::RepeatingCallback<void(const dispatch::Context&,
                                             std::string_view,
                                             base::span<const uint8_t>,
                                             Reply)>;

// Input descriptors may use static views; Router copies every path.
struct Route {
  std::string_view path;
  Handler handler;
};

}  // namespace tuffite::rpc

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_ROUTE_H_
