// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/network_context/safe_storage_key_provider_mac.h"

#import <CommonCrypto/CommonCryptor.h>
#import <CommonCrypto/CommonKeyDerivation.h>
#import <Foundation/Foundation.h>
#import <Security/Security.h>

#include <array>
#include <utility>

#include "base/apple/bridging.h"
#include "base/base64.h"
#include "base/command_line.h"
#include "base/files/file_enumerator.h"
#include "base/functional/bind.h"
#include "base/rand_util.h"
#include "base/strings/sys_string_conversions.h"
#include "base/task/thread_pool.h"
#include "components/os_crypt/async/common/algorithm.mojom.h"
#include "crypto/apple/fake_keychain_v2.h"
#include "crypto/apple/keychain_v2.h"
#include "sql/database.h"
#include "sql/statement.h"

namespace tuffite {
namespace {
using Provider = os_crypt_async::KeyProvider;
using Key = os_crypt_async::Encryptor::Key;

// A former Framework profile used Chromium's v10 key. Reuse that password
// only when upgrading a profile containing a Cookie database; never alter or
// delete Chromium's shared Keychain item. Existing profiles keep their data.
base::expected<bool, Provider::KeyError> HasExistingCookies(
    const base::FilePath& profile) {
  base::FileEnumerator files(profile, true, base::FileEnumerator::FILES,
                             FILE_PATH_LITERAL("Cookies"));
  for (auto path = files.Next(); !path.empty(); path = files.Next()) {
    sql::Database database(sql::DatabaseOptions().set_read_only(true),
                           "Cookie");
    if (!database.Open(path)) {
      return base::unexpected(Provider::KeyError::kTemporarilyUnavailable);
    }
    if (!database.DoesTableExist("cookies")) {
      continue;
    }
    sql::Statement statement(database.GetUniqueStatement(
        "SELECT 1 FROM cookies WHERE length(encrypted_value) > 0 LIMIT 1"));
    if (statement.Step()) {
      return true;
    }
    if (!statement.Succeeded()) {
      return base::unexpected(Provider::KeyError::kTemporarilyUnavailable);
    }
  }
  return false;
}

base::expected<Key, Provider::KeyError> LoadKey(
    const std::string& application_id,
    const std::string& product_name,
    const base::FilePath& profile,
    bool mock) {
  std::unique_ptr<crypto::apple::FakeKeychainV2> fake;
  auto* keychain = &crypto::apple::KeychainV2::GetInstance();
  if (mock) {
    fake = std::make_unique<crypto::apple::FakeKeychainV2>("test-access-group");
    keychain = fake.get();
  }
  // Identity is stable across product renames. The user-facing label is
  // independent from lookup attributes and is updated without changing the key.
  const std::string service = application_id + " Safe Storage";
  auto password = keychain->FindGenericPassword(service, application_id);
  NSDictionary* identity = @{
    base::apple::CFToNSPtrCast(kSecClass) :
        base::apple::CFToNSPtrCast(kSecClassGenericPassword),
    base::apple::CFToNSPtrCast(kSecAttrService) :
        base::SysUTF8ToNSString(service),
    base::apple::CFToNSPtrCast(kSecAttrAccount) :
        base::SysUTF8ToNSString(application_id),
  };
  NSDictionary* label = @{
    base::apple::CFToNSPtrCast(kSecAttrLabel) :
        base::SysUTF8ToNSString(product_name + " Safe Storage"),
  };
  if (!password.has_value()) {
    if (password.error() != errSecItemNotFound) {
      return base::unexpected(Provider::KeyError::kTemporarilyUnavailable);
    }
    auto existing_cookies = HasExistingCookies(profile);
    if (!existing_cookies.has_value()) {
      return base::unexpected(existing_cookies.error());
    }
    if (*existing_cookies) {
      password =
          keychain->FindGenericPassword("Chromium Safe Storage", "Chromium");
      if (!password.has_value()) {
        return base::unexpected(Provider::KeyError::kTemporarilyUnavailable);
      }
    } else {
      const auto generated = base::Base64Encode(base::RandBytesAsVector(16));
      password = std::vector<uint8_t>(generated.begin(), generated.end());
    }
    NSMutableDictionary* attributes = [identity mutableCopy];
    [attributes addEntriesFromDictionary:label];
    attributes[base::apple::CFToNSPtrCast(kSecValueData)] =
        [NSData dataWithBytes:password->data() length:password->size()];
    const auto status =
        keychain->ItemAdd(base::apple::NSToCFPtrCast(attributes), nullptr);
    if (status == errSecDuplicateItem) {
      // Another process may have created the application's key first.
      password = keychain->FindGenericPassword(service, application_id);
    } else if (status != noErr) {
      return base::unexpected(Provider::KeyError::kTemporarilyUnavailable);
    }
  } else if (!mock) {
    keychain->ItemUpdate(base::apple::NSToCFPtrCast(identity),
                         base::apple::NSToCFPtrCast(label));
  }
  if (!password.has_value() || password->empty()) {
    return base::unexpected(Provider::KeyError::kTemporarilyUnavailable);
  }
  // Preserve Chromium's macOS v10 format and PBKDF2 parameters so migrated
  // cookies remain decryptable. CommonCrypto supplies the system PBKDF2 API.
  constexpr std::array<uint8_t, 9> salt = {'s', 'a', 'l', 't', 'y',
                                           's', 'a', 'l', 't'};
  std::array<uint8_t, Key::kAES128CBCKeySize> bytes;
  if (CCKeyDerivationPBKDF(
          kCCPBKDF2, reinterpret_cast<const char*>(password->data()),
          password->size(), salt.data(), salt.size(), kCCPRFHmacAlgSHA1, 1003,
          bytes.data(), bytes.size()) != kCCSuccess) {
    return base::unexpected(Provider::KeyError::kTemporarilyUnavailable);
  }
  return Key(bytes, os_crypt_async::mojom::Algorithm::kAES128CBC);
}

class SafeStorageKeyProvider final : public Provider {
 public:
  SafeStorageKeyProvider(std::string application_id,
                         std::string product_name,
                         base::FilePath profile)
      : application_id_(std::move(application_id)),
        product_name_(std::move(product_name)),
        profile_(std::move(profile)) {}
  void GetKey(KeyCallback callback) override {
    base::ThreadPool::PostTaskAndReplyWithResult(
        FROM_HERE, {base::MayBlock(), base::TaskPriority::USER_BLOCKING},
        base::BindOnce(&LoadKey, application_id_, product_name_, profile_,
                       base::CommandLine::ForCurrentProcess()->HasSwitch(
                           "use-mock-keychain")),
        base::BindOnce(std::move(callback), "v10"));
  }
  bool UseForEncryption() override { return true; }

 private:
  const std::string application_id_;
  const std::string product_name_;
  const base::FilePath profile_;
};
}  // namespace
std::unique_ptr<Provider> CreateSafeStorageKeyProvider(
    std::string application_id,
    std::string product_name,
    base::FilePath profile) {
  return std::make_unique<SafeStorageKeyProvider>(
      std::move(application_id), std::move(product_name), std::move(profile));
}
}  // namespace tuffite
