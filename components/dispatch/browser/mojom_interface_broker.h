// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_COMPONENTS_DISPATCH_BROWSER_MOJOM_INTERFACE_BROKER_H_
#define TUFFITE_COMPONENTS_DISPATCH_BROWSER_MOJOM_INTERFACE_BROKER_H_
#include "mojo/public/cpp/bindings/pending_receiver.h"
#include "third_party/blink/public/mojom/browser_interface_broker.mojom.h"
namespace content {
class RenderFrameHost;
}
namespace tuffite {
class RpcHandler;
void BindMojomInterfaceBroker(
    RpcHandler*,
    content::RenderFrameHost*,
    mojo::PendingReceiver<blink::mojom::BrowserInterfaceBroker>);
}  // namespace tuffite
#endif
