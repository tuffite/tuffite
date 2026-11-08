// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_HOST_DISPATCH_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_HOST_DISPATCH_H_
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

#include "base/functional/callback.h"
#include "base/unguessable_token.h"

namespace tuffite {
struct MojomMethod {
  std::string channel;
  uint64_t command_id = 0;
  uint64_t schema_fingerprint = 0;
};
class HostDispatch {
 public:
  using DispatchCallback = base::OnceCallback<void(bool, std::vector<uint8_t>)>;
  using MojomReply =
      base::OnceCallback<void(int32_t, const std::vector<uint8_t>&)>;
  virtual ~HostDispatch() = default;
  virtual std::optional<MojomMethod> ResolveMojomMethod(const std::string&,
                                                        uint32_t) const;
  virtual void DispatchMojom(const MojomMethod&,
                             const std::vector<uint8_t>&,
                             const base::UnguessableToken&,
                             MojomReply);
  virtual bool Dispatch(std::vector<uint8_t>,
                        DispatchCallback,
                        const base::UnguessableToken&) = 0;
  virtual void DispatchFramework(uint64_t,
                                 std::string,
                                 DispatchCallback callback);
  virtual void CancelDocument(const base::UnguessableToken&) = 0;
};
}  // namespace tuffite

#endif  // TUFFITE_COMPONENTS_DISPATCH_BROWSER_HOST_DISPATCH_H_
