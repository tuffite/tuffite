// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <array>
#include <optional>

#include "base/functional/bind.h"
#include "base/memory/raw_ref.h"
#include "base/test/launcher/unit_test_launcher.h"
#include "content/public/test/test_renderer_host.h"
#include "content/public/test/unittest_test_suite.h"
#include "content/test/content_test_suite.h"
#include "tuffite/components/dispatch/browser/rpc.h"
#include "tuffite/framework/public/shell_api/path.mojom.h"
#include "tuffite/framework/test/dispatch_fixture.mojom.h"
#include "url/gurl.h"

namespace tuffite {
namespace {
class DeferredPath final : public shell_api::generated::Path {
 public:
  DeferredPath(DirectoriesCallback& reply,
               std::optional<dispatch::RendererRequestContext>& caller)
      : reply_(reply), caller_(caller) {}
  void Directories(const dispatch::RendererRequestContext& context,
                   DirectoriesRequestPtr,
                   DirectoriesCallback reply) override {
    *caller_ = context;
    *reply_ = std::move(reply);
  }
  void Resolve(const dispatch::RendererRequestContext&,
               ResolveRequestPtr,
               ResolveCallback reply) override {
    rpc::Reject(reply, rpc::Status::kInvalidArgument);
  }

 private:
  raw_ref<DirectoriesCallback> reply_;
  raw_ref<std::optional<dispatch::RendererRequestContext>> caller_;
};

class DispatchFixtureImpl final : public shell_api::generated::DispatchFixture {
 public:
  void RoundTrip(const dispatch::RendererRequestContext& context,
                 RoundTripRequestPtr request,
                 RoundTripCallback reply) override {
    EXPECT_TRUE(context.is_valid());
    if (!request->value) {
      rpc::Reject(reply, rpc::Status::kAccessDenied);
      return;
    }
    // Directly transfer official StructPtrs and arrays into the reply.
    const auto count = static_cast<uint32_t>(request->values.size());
    std::move(reply).Run(rpc::Status::kOk, count, std::move(request->value),
                         std::move(request->values));
  }
  void Empty(const dispatch::RendererRequestContext& context,
             EmptyRequestPtr,
             EmptyCallback reply) override {
    EXPECT_TRUE(context.is_valid());
    std::move(reply).Run(rpc::Status::kOk);
  }
};

constexpr std::array<uint8_t, 8> kEmptyMojomRequest = {8, 0, 0, 0, 0, 0, 0, 0};
using RendererRpcTest = content::RenderViewHostTestHarness;

TEST_F(RendererRpcTest,
       GeneratedBindingsPreserveCodecsAndDeferredReplyOwnership) {
  NavigateAndCommit(GURL("https://example.test/first"));
  dispatch::RendererRequestContext first(*main_rfh(),
                                         base::UnguessableToken::Create(), 11);
  auto fixture = shell_api::generated::DispatchFixture::Bind(
      std::make_shared<DispatchFixtureImpl>());
  auto input = wire::DispatchFixtureRoundTripRequest::New();
  input->value =
      wire::DispatchValue::New("nullable", std::vector<uint8_t>{1, 2, 3});
  input->values.push_back(
      wire::DispatchValue::New("array", std::vector<uint8_t>{4, 5}));
  auto input_bytes = wire::DispatchFixtureRoundTripRequest::Serialize(&input);
  bool fixture_completed = false;
  rpc::Reply fixture_reply = base::BindOnce(
      [](bool* done, int32_t status, const std::vector<uint8_t>& bytes) {
        EXPECT_EQ(status, static_cast<int32_t>(rpc::Status::kOk));
        wire::DispatchFixtureRoundTripResponsePtr response;
        ASSERT_TRUE(wire::DispatchFixtureRoundTripResponse::Deserialize(
            bytes, &response));
        EXPECT_EQ(response->count, 1u);
        ASSERT_TRUE(response->value);
        EXPECT_EQ(response->value->label, "nullable");
        EXPECT_EQ(response->value->bytes, (std::vector<uint8_t>{1, 2, 3}));
        ASSERT_EQ(response->values.size(), 1u);
        EXPECT_EQ(response->values[0]->label, "array");
        EXPECT_EQ(response->values[0]->bytes, (std::vector<uint8_t>{4, 5}));
        *done = true;
      },
      &fixture_completed);
  EXPECT_TRUE(
      fixture.dispatch.Run(first, "RoundTrip", input_bytes, &fixture_reply));
  EXPECT_TRUE(fixture_completed);
  input->value.reset();
  input_bytes = wire::DispatchFixtureRoundTripRequest::Serialize(&input);
  fixture_reply =
      base::BindOnce([](int32_t status, const std::vector<uint8_t>& bytes) {
        EXPECT_EQ(status, static_cast<int32_t>(rpc::Status::kAccessDenied));
        EXPECT_TRUE(bytes.empty());
      });
  EXPECT_TRUE(
      fixture.dispatch.Run(first, "RoundTrip", input_bytes, &fixture_reply));
  fixture_reply =
      base::BindOnce([](int32_t status, const std::vector<uint8_t>& bytes) {
        EXPECT_EQ(status, static_cast<int32_t>(rpc::Status::kOk));
        wire::DispatchFixtureEmptyResponsePtr response;
        EXPECT_TRUE(
            wire::DispatchFixtureEmptyResponse::Deserialize(bytes, &response));
      });
  EXPECT_TRUE(
      fixture.dispatch.Run(first, "Empty", kEmptyMojomRequest, &fixture_reply));
  fixture_reply =
      base::BindOnce([](int32_t status, const std::vector<uint8_t>& bytes) {
        EXPECT_EQ(status, static_cast<int32_t>(rpc::Status::kInvalidArgument));
        EXPECT_TRUE(bytes.empty());
      });
  EXPECT_TRUE(fixture.dispatch.Run(first, "RoundTrip", kEmptyMojomRequest,
                                   &fixture_reply));

  // A null top-level Mojo request is not a valid RPC call, even though the
  // generic Mojo deserializer accepts it. Never pass it to a handler.
  for (const char* method : {"Empty", "RoundTrip"}) {
    bool rejected = false;
    fixture_reply = base::BindOnce(
        [](bool* rejected, int32_t status, const std::vector<uint8_t>& bytes) {
          EXPECT_EQ(status,
                    static_cast<int32_t>(rpc::Status::kInvalidArgument));
          EXPECT_TRUE(bytes.empty());
          *rejected = true;
        },
        &rejected);
    EXPECT_TRUE(fixture.dispatch.Run(first, method, {}, &fixture_reply));
    EXPECT_TRUE(rejected);
  }

  // Both route ownership and pending reply ownership must keep the service
  // alive, including after a router is torn down during an asynchronous call.
  shell_api::generated::Path::DirectoriesCallback pending;
  std::optional<dispatch::RendererRequestContext> pending_context;
  auto service = std::make_shared<DeferredPath>(pending, pending_context);
  std::weak_ptr<DeferredPath> weak_service = service;
  auto binding = shell_api::generated::Path::Bind(service);
  service.reset();
  EXPECT_FALSE(weak_service.expired());
  bool completed = false;
  rpc::Reply completion = base::BindOnce(
      [](bool* done, int32_t status, const std::vector<uint8_t>& bytes) {
        EXPECT_EQ(status, static_cast<int32_t>(rpc::Status::kOk));
        EXPECT_FALSE(bytes.empty());
        *done = true;
      },
      &completed);
  EXPECT_TRUE(binding.dispatch.Run(first, "Directories", kEmptyMojomRequest,
                                   &completion));
  ASSERT_TRUE(pending);
  EXPECT_EQ((*pending_context).connection(), first.connection());
  EXPECT_EQ(pending_context->request_id(), 11u);
  auto first_response = std::move(pending);
  const auto first_context = *pending_context;
  EXPECT_EQ((*pending_context).connection(), first.connection());
  dispatch::RendererRequestContext second(*main_rfh(),
                                          base::UnguessableToken::Create(), 12);
  bool second_completed = false;
  rpc::Reply second_reply = base::BindOnce(
      [](bool* done, int32_t status, const std::vector<uint8_t>&) {
        EXPECT_EQ(status, static_cast<int32_t>(rpc::Status::kOk));
        *done = true;
      },
      &second_completed);
  EXPECT_TRUE(binding.dispatch.Run(second, "Directories", kEmptyMojomRequest,
                                   &second_reply));
  EXPECT_EQ(first_context.connection(), first.connection());
  EXPECT_EQ((*pending_context).connection(), second.connection());
  EXPECT_EQ(pending_context->request_id(), 12u);
  rpc::Reply unknown_reply = base::BindOnce(
      [](int32_t, const std::vector<uint8_t>&) { ADD_FAILURE(); });
  EXPECT_FALSE(binding.dispatch.Run(first, "Unknown", kEmptyMojomRequest,
                                    &unknown_reply));
  EXPECT_TRUE(unknown_reply);
  binding.dispatch.Reset();
  EXPECT_FALSE(weak_service.expired());
  std::move(first_response)
      .Run(rpc::Status::kOk, shell_api::generated::PathDirectories::New());
  EXPECT_TRUE(completed);
  EXPECT_FALSE(weak_service.expired());
  std::move(pending).Run(rpc::Status::kOk,
                         shell_api::generated::PathDirectories::New());
  EXPECT_TRUE(second_completed);
  EXPECT_TRUE(weak_service.expired());

  // The frame may be reused; the old document identity must still be revoked.
  NavigateAndCommit(GURL("https://example.test/second"));
  EXPECT_FALSE(first.is_valid());
  EXPECT_EQ(first.sender(), nullptr);
  auto expired = shell_api::generated::DispatchFixture::Bind(
      std::make_shared<DispatchFixtureImpl>());
  bool cancelled = false;
  rpc::Reply expired_reply = base::BindOnce(
      [](bool* cancelled, int32_t status, const std::vector<uint8_t>& bytes) {
        EXPECT_EQ(status, static_cast<int32_t>(rpc::Status::kCancelled));
        EXPECT_TRUE(bytes.empty());
        *cancelled = true;
      },
      &cancelled);
  EXPECT_TRUE(
      expired.dispatch.Run(first, "Empty", kEmptyMojomRequest, &expired_reply));
  EXPECT_TRUE(cancelled);
}
}  // namespace
}  // namespace tuffite

int main(int argc, char** argv) {
  content::UnitTestTestSuite suite(
      new content::ContentTestSuite(argc, argv),
      base::BindRepeating(
          content::UnitTestTestSuite::CreateTestContentClients));
  return base::LaunchUnitTests(argc, argv,
                               base::BindOnce(&content::UnitTestTestSuite::Run,
                                              base::Unretained(&suite)));
}
