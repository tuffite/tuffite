// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_REQUEST_DIALOG_H_
#define TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_REQUEST_DIALOG_H_

#include <cstdint>
#include <memory>
#include <string>

#include "base/functional/callback.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "ui/gfx/native_ui_types.h"
#include "ui/views/view.h"

namespace views {
class Textfield;
class Label;
}  // namespace views

namespace tuffite {

// Browser-owned UI: website scripts cannot read PINs or choose an account.
class WebAuthnRequestDialog final : public views::View {
 public:
  WebAuthnRequestDialog(const std::string& rp_id,
                        base::RepeatingClosure cancel,
                        bool conditional);
  ~WebAuthnRequestDialog() override;
  base::WeakPtr<WebAuthnRequestDialog> GetWeakPtr();
  static void Show(std::unique_ptr<WebAuthnRequestDialog> dialog,
                   gfx::NativeWindow context,
                   gfx::NativeView parent);
  void CloseWithoutCancelling();
  void AddMessage(const std::u16string& message);
  void AddAction(const std::u16string& label, base::RepeatingClosure action);
  bool AddQRCode(const std::string& qr);
  void CollectPIN(uint32_t minimum_length,
                  bool new_pin,
                  base::OnceCallback<void(std::u16string)> callback);

 private:
  void Cancel();
  std::u16string title_;
  base::RepeatingClosure cancel_;
  bool conditional_;
  bool closing_ = false;
  raw_ptr<views::Textfield> pin_ = nullptr;
  raw_ptr<views::Textfield> confirmation_ = nullptr;
  raw_ptr<views::Label> validation_ = nullptr;
  base::WeakPtrFactory<WebAuthnRequestDialog> weak_factory_{this};
};

}  // namespace tuffite

#endif  // TUFFITE_FRAMEWORK_BROWSER_SERVICES_WEBAUTHN_REQUEST_DIALOG_H_
