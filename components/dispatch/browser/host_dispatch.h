// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_HOST_DISPATCH_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_HOST_DISPATCH_H_
#include <cstdint>
#include <string>
#include <utility>
#include <vector>

#include "base/functional/callback.h"
#include "base/unguessable_token.h"
namespace tuffite {
class HostDispatch {
 public:
  using DispatchCallback = base::OnceCallback<void(bool, std::vector<uint8_t>)>;
  virtual ~HostDispatch() = default;
  virtual bool Dispatch(std::vector<uint8_t>,
                        DispatchCallback,
                        const base::UnguessableToken&) = 0;
  virtual void DispatchFramework(uint64_t,
                                 std::string,
                                 DispatchCallback callback) {
    std::move(callback).Run(false, {});
  }
  virtual void CancelDocument(const base::UnguessableToken&) = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_HOST_DISPATCH_H_
