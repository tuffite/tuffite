// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#import <Foundation/Foundation.h>
#import <Security/Security.h>

#include "base/apple/scoped_cftyperef.h"
#include "base/process/process_info.h"
#include "base/strings/sys_string_conversions.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service_impl.h"

namespace tuffite {

std::optional<std::string> WebAuthnKeychainAccessGroup() {
  base::apple::ScopedCFTypeRef<SecTaskRef> task(SecTaskCreateFromSelf(nullptr));
  if (!task) {
    return std::nullopt;
  }
  base::apple::ScopedCFTypeRef<CFTypeRef> value(SecTaskCopyValueForEntitlement(
      task.get(), CFSTR("keychain-access-groups"), nullptr));
  if (!value || CFGetTypeID(value.get()) != CFArrayGetTypeID()) {
    return std::nullopt;
  }
  NSArray* groups = (__bridge NSArray*)value.get();
  for (id group in groups) {
    if ([group isKindOfClass:[NSString class]] &&
        [group hasSuffix:@".webauthn"]) {
      return base::SysNSStringToUTF8(group);
    }
  }
  return std::nullopt;
}

bool WebAuthnBluetoothAllowed() {
  id description = [[NSBundle mainBundle]
      objectForInfoDictionaryKey:@"NSBluetoothAlwaysUsageDescription"];
  return [description isKindOfClass:[NSString class]] &&
         [(NSString*)description length] > 0 &&
         base::DoesResponsibleProcessHaveBluetoothMetadata();
}

}  // namespace tuffite
