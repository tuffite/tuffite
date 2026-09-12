// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REGISTRY_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REGISTRY_H_

#include <utility>
#include <vector>

#include "tuffite/components/dispatch/browser/rpc_router.h"
#include "tuffite/components/dispatch/browser/shell_api_protocol.h"
#include "tuffite/components/dispatch/common/status.h"

namespace tuffite::shell_api {

// Keep status strongly typed until the final Dispatch wire boundary.
inline constexpr rpc::Status kStatusOk = rpc::Status::kOk;
inline constexpr rpc::Status kStatusInvalidArgument =
    rpc::Status::kInvalidArgument;
inline constexpr rpc::Status kStatusAccessDenied = rpc::Status::kAccessDenied;
inline constexpr rpc::Status kStatusUnavailable = rpc::Status::kUnavailable;

using Reply = rpc::Reply;
using Handler = rpc::Handler;
using Route = rpc::Route;

// Collects typed ShellAPI routes from generated Method reflections and
// user-supplied handler implementations. Implementations may be a free
// function taking `(Request&&, Callback&&)`, or a context function taking
// `(const dispatch::Context&, Request&&, Callback&&)`.
class Registry {
 public:
  Registry();
  ~Registry();
  Registry(Registry&&) noexcept;
  Registry& operator=(Registry&&) noexcept;
  Registry(const Registry&) = delete;
  Registry& operator=(const Registry&) = delete;

  template <typename Request, typename Result, typename Implementation>
  Registry& Register(const generated::Method<Request, Result>& method,
                     Implementation implementation) {
    auto route = method.Bind(std::move(implementation));
    routes_.push_back(std::move(route));
    return *this;
  }

  std::vector<rpc::Route> Build() && { return std::move(routes_); }

 private:
  std::vector<rpc::Route> routes_;
};

// Convenience wrapper matching the free-function registration style.
template <typename Request, typename Result, typename Implementation>
inline void Register(Registry& registry,
                     const generated::Method<Request, Result>& method,
                     Implementation implementation) {
  registry.Register(method, std::move(implementation));
}

std::vector<rpc::Route> BuiltinRoutes();

}  // namespace tuffite::shell_api

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_SHELL_API_COMMANDS_REGISTRY_H_
