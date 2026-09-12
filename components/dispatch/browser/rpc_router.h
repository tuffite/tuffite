// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_ROUTER_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_ROUTER_H_

#include <string>
#include <string_view>
#include <vector>

#include "base/containers/span.h"
#include "tuffite/components/dispatch/browser/rpc_route.h"

namespace tuffite::rpc {
class Router final {
 public:
  explicit Router(std::vector<Route>);
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
    std::string path;
    Handler handler;
  };
  std::vector<Entry> routes_;
};

}  // namespace tuffite::rpc

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_RPC_ROUTER_H_
