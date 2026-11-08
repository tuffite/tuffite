// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/manifest/renderer/blink/document_configuration_reader.h"

#include "third_party/blink/renderer/bindings/core/v8/v8_binding_for_core.h"
#include "third_party/blink/renderer/core/dom/document.h"
#include "third_party/blink/renderer/core/frame/local_dom_window.h"
#include "third_party/blink/renderer/core/frame/policy_container.h"
#include "third_party/blink/renderer/platform/weborigin/security_origin.h"
#include "tuffite/components/manifest/common/document_configuration.mojom-blink.h"

namespace tuffite {
namespace {
const mojom::blink::DocumentConfiguration* Configuration(
    const blink::Document& document) {
  auto* window = document.domWindow();
  if (!window || !window->GetSecurityOrigin() ||
      window->GetSecurityOrigin()->IsOpaque() ||
      !window->GetPolicyContainer()) {
    return nullptr;
  }
  return window->GetPolicyContainer()
      ->GetPolicies()
      .tuffite_configuration.get();
}

}  // namespace

bool IsShellApiExposed(v8::Local<v8::Context> context) {
  auto* window = blink::ToLocalDOMWindow(context);
  auto* config = window && window->document()
                     ? Configuration(*window->document())
                     : nullptr;
  return config && config->expose_shell_api;
}

bool IsExplorerEnabled(const blink::Document& document) {
  auto* config = Configuration(document);
  return config && config->enable_explorer;
}

}  // namespace tuffite
