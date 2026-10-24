// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "base/strings/string_number_conversions.h"
#include "mojo/core/embedder/embedder.h"
#include "testing/gtest/include/gtest/gtest.h"
#include "tuffite/framework/shell_api/fs.mojom-wire.mojom.h"
#include "tuffite/framework/shell_api/remoting.mojom-wire.mojom.h"
#include "tuffite/framework/shell_api/window.mojom-wire.mojom.h"
#include "tuffite/playground/mojom_data.mojom.h"

namespace {
void InitializeMojo() {
  static const bool initialized = [] {
    mojo::core::Init();
    return true;
  }();
  (void)initialized;
}

constexpr char kSample[] =
#include "tuffite/components/dispatch/test/mojom_sample.inc"
    ;

TEST(MojomCodecTest, OfficialCppCodecMatchesSourceDistributedRustAndJs) {
  InitializeMojo();
  std::vector<uint8_t> fixture;
  ASSERT_TRUE(base::HexStringToBytes(kSample, &fixture));
  auto sample = playground::MojomSample::New();
  sample->small = 7;
  sample->large = (uint64_t{1} << 63) + 17;
  sample->enabled = true;
  sample->visible = false;
  sample->medium = 513;
  sample->signedValue = -12345;
  sample->fraction = 1.25;
  sample->nested = playground::MojomNested::New();
  sample->nested->label = "\xe4\xbd\xa0\xe5\xa5\xbd";
  sample->nested->bytes = {0, 1, 255};
  EXPECT_EQ(playground::MojomSample::Serialize(&sample), fixture);
  playground::MojomSamplePtr decoded;
  ASSERT_TRUE(playground::MojomSample::Deserialize(fixture, &decoded));
  EXPECT_TRUE(sample.Equals(decoded));
}
}  // namespace

namespace {
TEST(FrameworkMojomCodecTest,
     OfficialCodecMatchesNullableArrayAndEnumFixtures) {
  InitializeMojo();
  const char fs_hex[] =
#include "tuffite/components/dispatch/test/framework_fs.inc"
      ;
  const char window_hex[] =
#include "tuffite/components/dispatch/test/framework_window.inc"
      ;
  const char remoting_hex[] =
#include "tuffite/components/dispatch/test/framework_remoting.inc"
      ;
  auto check = []<typename T>(T value, const char* hex) {
    std::vector<uint8_t> expected;
    ASSERT_TRUE(base::HexStringToBytes(hex, &expected));
    using Data = typename T::Struct;
    EXPECT_EQ(Data::Serialize(&value), expected);
    T decoded;
    ASSERT_TRUE(Data::Deserialize(expected, &decoded));
    EXPECT_TRUE(value.Equals(decoded));
  };
  auto fs = tuffite::wire::FsReadFileRequest::New();
  fs->base = static_cast<tuffite::wire::BaseDirectory>(1);
  fs->path = "\xe9\x85\x8d\xe7\xbd\xae.bin";
  check(std::move(fs), fs_hex);
  auto window = tuffite::wire::WindowCreateRequest::New();
  window->options = tuffite::wire::WindowOptions::New();
  window->options->label = "child";
  window->options->bounds =
      tuffite::wire::WindowBounds::New(10., 20., 640., 480.);
  window->options->focused = false;
  check(std::move(window), window_hex);
  auto remoting = tuffite::wire::RemotingStartRequest::New();
  remoting->options = tuffite::wire::RemotingStartOptions::New();
  remoting->options->auth_key = "secret";
  auto server = tuffite::wire::RemotingIceServer::New();
  server->urls = {"stun:example.test", "turn:example.test"};
  server->username = "user";
  remoting->options->ice_servers.push_back(std::move(server));
  remoting->options->target_framerate = 60.;
  remoting->options->screen_id = 0;
  check(std::move(remoting), remoting_hex);
}
}  // namespace
