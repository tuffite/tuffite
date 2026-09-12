// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#import <Foundation/Foundation.h>

#include "base/process/process_info.h"
#include "tuffite/framework/browser/services/webauthn/web_authentication_service_impl.h"

namespace tuffite {

bool WebAuthnBluetoothAllowed() {
  id description = [[NSBundle mainBundle]
      objectForInfoDictionaryKey:@"NSBluetoothAlwaysUsageDescription"];
  return [description isKindOfClass:[NSString class]] &&
         [(NSString*)description length] > 0 &&
         base::DoesResponsibleProcessHaveBluetoothMetadata();
}

}  // namespace tuffite
