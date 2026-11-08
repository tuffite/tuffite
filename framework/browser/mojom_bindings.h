// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#ifndef TUFFITE_FRAMEWORK_BROWSER_MOJOM_BINDINGS_H_
#define TUFFITE_FRAMEWORK_BROWSER_MOJOM_BINDINGS_H_
namespace content {
class WebContents;
}
namespace tuffite {
void ObserveMojomDocuments(content::WebContents*);
}
#endif
