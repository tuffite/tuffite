// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/app/content_client.h"

#include "base/command_line.h"
#include "base/memory/ref_counted_memory.h"
#include "tuffite/framework/common/switches.h"
#include "ui/base/l10n/l10n_util.h"
#include "ui/base/resource/resource_bundle.h"

namespace tuffite {

ContentClient::ContentClient() = default;
ContentClient::~ContentClient() = default;

void ContentClient::AddAdditionalSchemes(Schemes* schemes) {
  const std::string scheme =
      base::CommandLine::ForCurrentProcess()->GetSwitchValueASCII(
          kAppSchemeSwitch);
  if (scheme.empty()) {
    return;
  }
  schemes->standard_schemes.push_back(scheme);
  schemes->secure_schemes.push_back(scheme);
  // Application assets have normal tuple origins. Local-scheme file access
  // restrictions would serialize their security origin as null and break
  // standard Window messaging and Workers.
  schemes->cors_enabled_schemes.push_back(scheme);
}

std::u16string ContentClient::GetLocalizedString(int message_id) {
  return l10n_util::GetStringUTF16(message_id);
}

std::string_view ContentClient::GetDataResource(
    int resource_id,
    ui::ResourceScaleFactor scale_factor) {
  return ui::ResourceBundle::GetSharedInstance().GetRawDataResourceForScale(
      resource_id, scale_factor);
}

scoped_refptr<base::RefCountedMemory> ContentClient::GetDataResourceBytes(
    int resource_id) {
  return ui::ResourceBundle::GetSharedInstance().LoadDataResourceBytes(
      resource_id);
}

std::string ContentClient::GetDataResourceString(int resource_id) {
  return ui::ResourceBundle::GetSharedInstance().LoadDataResourceString(
      resource_id);
}

gfx::Image& ContentClient::GetNativeImageNamed(int resource_id) {
  return ui::ResourceBundle::GetSharedInstance().GetNativeImageNamed(
      resource_id);
}

}  // namespace tuffite
