// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/components/dispatch/common/codec.h"

#include "testing/gtest/include/gtest/gtest.h"

namespace tuffite::rpc {
TEST(CodecTest, ExistingRequestAndResponseWireFormatRoundTrips) {
  auto request = DecodeRequest(EncodeRequest({42, 99, 77, {1, 2, 3}}));
  ASSERT_TRUE(request);
  EXPECT_EQ(request->request_id, 42u);
  EXPECT_EQ(request->command_id, 99u);
  EXPECT_EQ(request->schema_fingerprint, 77u);
  EXPECT_EQ(request->payload, (std::vector<uint8_t>{1, 2, 3}));
  auto response = DecodeResponse(EncodeResponse({42, 3, {7}}));
  ASSERT_TRUE(response);
  EXPECT_EQ(response->request_id, 42u);
  EXPECT_EQ(response->status, 3);
  EXPECT_EQ(response->payload, (std::vector<uint8_t>{7}));
  auto cancel = DecodeCancel(EncodeCancel({42}));
  ASSERT_TRUE(cancel);
  EXPECT_EQ(cancel->request_id, 42u);
}
TEST(CodecTest, ReplacesIdsOnlyAfterValidatingTheWholeFrame) {
  auto request = EncodeRequest({42, 99, 77, {1, 2, 3}});
  EXPECT_EQ(ReplaceRequestId(&request, 7), 42u);
  EXPECT_EQ(DecodeRequest(request)->request_id, 7u);
  auto response = EncodeResponse({7, 3, {4, 5}});
  EXPECT_TRUE(ReplaceResponseRequestId(&response, 42));
  EXPECT_EQ(DecodeResponse(response)->request_id, 42u);
  request.pop_back();
  EXPECT_FALSE(ReplaceRequestId(&request, 8));
  response.push_back(0);
  EXPECT_FALSE(ReplaceResponseRequestId(&response, 8));
}
TEST(CodecTest, RejectsTruncatedWrongKindAndWrongLength) {
  auto bytes = EncodeRequest({1, 2, 3, {3}});
  bytes.pop_back();
  EXPECT_FALSE(DecodeRequest(bytes));
  bytes = EncodeRequest({1, 2, 3, {3}});
  bytes[4] = 2;
  EXPECT_FALSE(DecodeRequest(bytes));
  bytes = EncodeResponse({1, 0, {}});
  bytes.push_back(0);
  EXPECT_FALSE(DecodeResponse(bytes));
  auto cancel = EncodeCancel({1});
  cancel.push_back(0);
  EXPECT_FALSE(DecodeCancel(cancel));
}
}  // namespace tuffite::rpc
