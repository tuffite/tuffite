// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <array>
#include <optional>

#include "base/functional/bind.h"
#include "base/functional/callback_helpers.h"
#include "base/memory/raw_ref.h"
#include "base/run_loop.h"
#include "base/test/launcher/unit_test_launcher.h"
#include "base/test/test_future.h"
#include "content/public/test/back_forward_cache_util.h"
#include "content/public/test/test_renderer_host.h"
#include "content/public/test/unittest_test_suite.h"
#include "content/test/content_test_suite.h"
#include "mojo/public/cpp/bindings/generic_pending_receiver.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "tuffite/components/dispatch/browser/mojom_interface_broker.h"
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

class TypedInterfaceHandler final : public RpcHandler {
 public:
  TypedInterfaceHandler()
      : router_({shell_api::generated::DispatchFixture::Bind(
            std::make_shared<DispatchFixtureImpl>())}) {}
  std::optional<MojomMethod> ResolveMojomMethod(
      std::string_view name,
      uint32_t ordinal) const override {
    auto method = router_.ResolveMethod(name, ordinal);
    return method ? std::optional(MojomMethod{*method}) : std::nullopt;
  }
  void CancelMojomRequest(const base::UnguessableToken&, uint64_t id) override {
    cancelled_requests.push_back(id);
  }
  bool IsAuthorized(content::RenderFrameHost&) const override { return true; }
  bool IsEventAuthorized(content::RenderFrameHost&) const override {
    return false;
  }
  void DispatchMojom(content::RenderFrameHost& frame,
                     const MojomMethod& method,
                     const std::vector<uint8_t>& bytes,
                     const base::UnguessableToken& connection,
                     uint64_t id,
                     Reply reply) override {
    connections.push_back(connection);
    operations.push_back(id);
    if (deny) {
      std::move(reply).Run(2, {});
      return;
    }
    if (defer) {
      pending = std::move(reply);
      return;
    }
    EXPECT_TRUE(router_.Dispatch(
        dispatch::RendererRequestContext(frame, connection, id), method.channel,
        bytes, &reply));
  }
  void CancelDocument(const base::UnguessableToken&) override { ++cancelled; }
  void Subscribe(const base::UnguessableToken&, Event) override {}
  std::vector<base::UnguessableToken> connections;
  std::vector<uint64_t> operations;
  std::vector<uint64_t> cancelled_requests;
  bool deny = false;
  bool defer = false;
  int cancelled = 0;
  Reply pending;

 private:
  rpc::Router router_;
};

TEST_F(RendererRpcTest, ChromiumGeneratedRemoteUsesCanonicalTypedPipe) {
  NavigateAndCommit(GURL("https://example.test/typed"));
  TypedInterfaceHandler handler;
  mojo::Remote<blink::mojom::BrowserInterfaceBroker> broker;
  base::ScopedClosureRunner cleanup(base::BindOnce(
      [](mojo::Remote<blink::mojom::BrowserInterfaceBroker>* broker) {
        broker->reset();
        base::RunLoop().RunUntilIdle();
      },
      base::Unretained(&broker)));
  BindMojomInterfaceBroker(&handler, main_rfh(),
                           broker.BindNewPipeAndPassReceiver());
  mojo::Remote<wire::DispatchFixture> remote;
  broker->GetInterface(
      mojo::GenericPendingReceiver(remote.BindNewPipeAndPassReceiver()));
  base::test::TestFuture<int32_t, wire::DispatchFixtureRoundTripResponsePtr,
                         const std::optional<std::string>&>
      reply;
  remote->RoundTrip(
      wire::DispatchValue::New("native", std::vector<uint8_t>{1, 2}), {},
      reply.GetCallback());
  ASSERT_TRUE(reply.Wait());
  EXPECT_EQ(reply.Get<0>(), 0);
  ASSERT_TRUE(reply.Get<1>());
  EXPECT_EQ(reply.Get<1>()->value->label, "native");
  EXPECT_EQ(reply.Get<1>()->value->bytes, (std::vector<uint8_t>{1, 2}));
  handler.deny = true;
  base::test::TestFuture<int32_t, wire::DispatchFixtureEmptyResponsePtr,
                         const std::optional<std::string>&>
      denied;
  remote->Empty(denied.GetCallback());
  ASSERT_TRUE(denied.Wait());
  EXPECT_EQ(denied.Get<0>(), 2);
  EXPECT_FALSE(denied.Get<1>());
  broker.reset();
  base::RunLoop().RunUntilIdle();
  EXPECT_GT(handler.cancelled, 0);
}

TEST_F(RendererRpcTest,
       SiblingTypedPipesShareDocumentBudgetButCancelIndependently) {
  NavigateAndCommit(GURL("https://example.test/pipes"));
  TypedInterfaceHandler handler;
  handler.defer = true;
  mojo::Remote<blink::mojom::BrowserInterfaceBroker> broker;
  base::ScopedClosureRunner cleanup(base::BindOnce(
      [](mojo::Remote<blink::mojom::BrowserInterfaceBroker>* broker) {
        broker->reset();
        base::RunLoop().RunUntilIdle();
      },
      base::Unretained(&broker)));
  BindMojomInterfaceBroker(&handler, main_rfh(),
                           broker.BindNewPipeAndPassReceiver());
  mojo::Remote<wire::DispatchFixture> first, second;
  broker->GetInterface(
      mojo::GenericPendingReceiver(first.BindNewPipeAndPassReceiver()));
  broker->GetInterface(
      mojo::GenericPendingReceiver(second.BindNewPipeAndPassReceiver()));
  first->Empty(base::BindOnce([](int32_t, wire::DispatchFixtureEmptyResponsePtr,
                                 const std::optional<std::string>&) {}));
  base::RunLoop().RunUntilIdle();
  ASSERT_EQ(handler.operations.size(), 1u);
  base::test::TestFuture<int32_t, wire::DispatchFixtureEmptyResponsePtr,
                         const std::optional<std::string>&>
      reply;
  second->Empty(reply.GetCallback());
  base::RunLoop().RunUntilIdle();
  ASSERT_EQ(handler.operations.size(), 2u);
  EXPECT_EQ(handler.connections[0], handler.connections[1]);
  EXPECT_NE(handler.operations[0], handler.operations[1]);
  first.reset();
  base::RunLoop().RunUntilIdle();
  ASSERT_EQ(handler.cancelled_requests.size(), 1u);
  EXPECT_EQ(handler.cancelled_requests[0], handler.operations[0]);
  std::move(handler.pending).Run(2, {});
  ASSERT_TRUE(reply.Wait());
  EXPECT_EQ(reply.Get<0>(), 2);
  broker.reset();
  base::RunLoop().RunUntilIdle();
}

TEST_F(RendererRpcTest, NavigationClosesTypedPipesAndCancelsPendingReplies) {
  // Exercise document destruction; cached documents intentionally retain pipes.
  content::DisableBackForwardCacheForTesting(
      web_contents(), content::BackForwardCache::TEST_REQUIRES_NO_CACHING);
  NavigateAndCommit(GURL("https://example.test/first"));
  TypedInterfaceHandler handler;
  handler.defer = true;
  mojo::Remote<blink::mojom::BrowserInterfaceBroker> broker;
  base::ScopedClosureRunner cleanup(base::BindOnce(
      [](mojo::Remote<blink::mojom::BrowserInterfaceBroker>* broker) {
        broker->reset();
        base::RunLoop().RunUntilIdle();
      },
      base::Unretained(&broker)));
  BindMojomInterfaceBroker(&handler, main_rfh(),
                           broker.BindNewPipeAndPassReceiver());
  mojo::Remote<wire::DispatchFixture> remote;
  broker->GetInterface(
      mojo::GenericPendingReceiver(remote.BindNewPipeAndPassReceiver()));
  base::test::TestFuture<void> disconnected;
  remote.set_disconnect_handler(disconnected.GetCallback());
  remote->Empty(
      base::BindOnce([](int32_t, wire::DispatchFixtureEmptyResponsePtr,
                        const std::optional<std::string>&) {
        ADD_FAILURE() << "A destroyed document must receive no reply";
      }));
  base::RunLoop().RunUntilIdle();
  ASSERT_TRUE(handler.pending);
  NavigateAndCommit(GURL("https://example.test/second"));
  ASSERT_TRUE(disconnected.Wait());
  EXPECT_GT(handler.cancelled, 0);
  std::move(handler.pending).Run(4, {});
  base::RunLoop().RunUntilIdle();
}

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
