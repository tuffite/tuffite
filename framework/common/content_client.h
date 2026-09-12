// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_COMMON_CONTENT_CLIENT_H_
#define TUFFITE_FRAMEWORK_COMMON_CONTENT_CLIENT_H_

#include "content/public/common/content_client.h"

namespace tuffite {

class ContentClient final : public content::ContentClient {
 public:
  ContentClient();
  ~ContentClient() override;

  void AddAdditionalSchemes(Schemes* schemes) override;
  std::u16string GetLocalizedString(int message_id) override;
  std::string_view GetDataResource(
      int resource_id,
      ui::ResourceScaleFactor scale_factor) override;
  scoped_refptr<base::RefCountedMemory> GetDataResourceBytes(
      int resource_id) override;
  std::string GetDataResourceString(int resource_id) override;
  gfx::Image& GetNativeImageNamed(int resource_id) override;
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_COMMON_CONTENT_CLIENT_H_
