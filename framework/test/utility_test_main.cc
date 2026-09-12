// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "base/test/test_suite.h"
#include "mojo/core/embedder/embedder.h"
int main(int argc, char** argv) {
  base::TestSuite suite(argc, argv);
  mojo::core::Init();
  return suite.Run();
}
