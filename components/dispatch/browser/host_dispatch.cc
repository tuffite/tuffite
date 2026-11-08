// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/browser/host_dispatch.h"

#include <utility>

namespace tuffite {

std::optional<MojomMethod> HostDispatch::ResolveMojomMethod(const std::string&,
                                                            uint32_t) const {
  return std::nullopt;
}
void HostDispatch::DispatchMojom(const MojomMethod&,
                                 const std::vector<uint8_t>&,
                                 const base::UnguessableToken&,
                                 MojomReply reply) {
  std::move(reply).Run(3, {});
}
void HostDispatch::DispatchFramework(uint64_t,
                                     std::string,
                                     DispatchCallback callback) {
  std::move(callback).Run(false, {});
}

}  // namespace tuffite
