// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/webauthn/request_dialog.h"

#include <utility>

#include "base/functional/bind.h"
#include "base/strings/utf_string_conversions.h"
#include "components/qr_code_generator/bitmap_generator.h"
#include "device/fido/pin.h"
#include "ui/base/ime/text_input_type.h"
#include "ui/base/models/dialog_model.h"
#include "ui/base/models/image_model.h"
#include "ui/gfx/geometry/insets.h"
#include "ui/views/accessibility/view_accessibility.h"
#include "ui/views/bubble/bubble_dialog_model_host.h"
#include "ui/views/controls/button/md_text_button.h"
#include "ui/views/controls/image_view.h"
#include "ui/views/controls/label.h"
#include "ui/views/controls/textfield/textfield.h"
#include "ui/views/layout/box_layout.h"
#include "ui/views/widget/widget.h"

namespace tuffite {

WebAuthnRequestDialog::WebAuthnRequestDialog(const std::string& rp_id,
                                             base::RepeatingClosure cancel,
                                             bool conditional) {
  title_ = u"Passkey · " + base::UTF8ToUTF16(rp_id);
  cancel_ = std::move(cancel);
  conditional_ = conditional;
  SetLayoutManager(std::make_unique<views::BoxLayout>(
      views::BoxLayout::Orientation::kVertical, gfx::Insets(20), 12));
}

WebAuthnRequestDialog::~WebAuthnRequestDialog() = default;

base::WeakPtr<WebAuthnRequestDialog> WebAuthnRequestDialog::GetWeakPtr() {
  return weak_factory_.GetWeakPtr();
}

void WebAuthnRequestDialog::CloseWithoutCancelling() {
  closing_ = true;
  if (GetWidget()) {
    GetWidget()->Close();
  }
}

void WebAuthnRequestDialog::Cancel() {
  if (!closing_) {
    closing_ = true;
    cancel_.Run();
  }
}

void WebAuthnRequestDialog::Show(std::unique_ptr<WebAuthnRequestDialog> dialog,
                                 gfx::NativeWindow context,
                                 gfx::NativeView parent) {
  auto weak = dialog->GetWeakPtr();
  auto cancel = base::BindRepeating(&WebAuthnRequestDialog::Cancel, weak);
  auto title = dialog->title_;
  bool conditional = dialog->conditional_;
  auto model =
      ui::DialogModel::Builder()
          .SetTitle(std::move(title))
          .DisableCloseOnDeactivate()
          .AddCancelButton(cancel)
          .SetCloseActionCallback(cancel)
          .SetDialogDestroyingCallback(cancel)
          .AddCustomField(
              std::make_unique<views::BubbleDialogModelHost::CustomView>(
                  std::move(dialog),
                  views::BubbleDialogModelHost::FieldType::kControl))
          .Build();
  std::unique_ptr<views::BubbleDialogModelHost> host;
  if (conditional) {
    host = std::make_unique<views::BubbleDialogModelHost>(
        std::move(model), static_cast<views::View*>(nullptr),
        views::BubbleBorder::Arrow::NONE);
  } else {
    host = views::BubbleDialogModelHost::CreateModal(
        std::move(model), ui::mojom::ModalType::kWindow);
  }
  auto* widget = views::DialogDelegate::CreateDialogWidget(std::move(host),
                                                           context, parent);
  if (conditional) {
    widget->ShowInactive();
  } else {
    widget->Show();
  }
}

void WebAuthnRequestDialog::AddMessage(const std::u16string& message) {
  auto* label = AddChildView(std::make_unique<views::Label>(message));
  label->SetMultiLine(true);
  label->SetHorizontalAlignment(gfx::ALIGN_LEFT);
  label->SetMaximumWidth(380);
}

void WebAuthnRequestDialog::AddAction(const std::u16string& label,
                                      base::RepeatingClosure action) {
  AddChildView(std::make_unique<views::MdTextButton>(std::move(action), label));
}

bool WebAuthnRequestDialog::AddQRCode(const std::string& qr) {
  auto image = qr_code_generator::GenerateImage(
      base::as_byte_span(qr), qr_code_generator::ModuleStyle::kSquares,
      qr_code_generator::LocatorStyle::kSquare,
      qr_code_generator::CenterImage::kNoCenterImage,
      qr_code_generator::QuietZone::kIncluded);
  if (!image.has_value()) {
    return false;
  }
  auto* view = AddChildView(std::make_unique<views::ImageView>());
  view->SetImage(ui::ImageModel::FromImageSkia(*image));
  return true;
}

void WebAuthnRequestDialog::CollectPIN(
    uint32_t minimum_length,
    bool new_pin,
    base::OnceCallback<void(std::u16string)> callback) {
  pin_ = AddChildView(std::make_unique<views::Textfield>());
  pin_->SetTextInputType(ui::TEXT_INPUT_TYPE_PASSWORD);
  pin_->GetViewAccessibility().SetName(u"Security key PIN");
  if (new_pin) {
    AddMessage(u"Confirm the new PIN");
    confirmation_ = AddChildView(std::make_unique<views::Textfield>());
    confirmation_->SetTextInputType(ui::TEXT_INPUT_TYPE_PASSWORD);
    confirmation_->GetViewAccessibility().SetName(u"Confirm PIN");
  }
  validation_ = AddChildView(std::make_unique<views::Label>());
  validation_->SetMultiLine(true);
  AddAction(
      u"Continue",
      base::BindRepeating(
          [](base::WeakPtr<WebAuthnRequestDialog> dialog, uint32_t minimum,
             base::OnceCallback<void(std::u16string)>* callback) {
            if (!dialog || !*callback) {
              return;
            }
            std::u16string pin(dialog->pin_->GetText());
            if (device::pin::ValidatePIN(pin, minimum) !=
                device::pin::PINEntryError::kNoError) {
              dialog->validation_->SetText(
                  u"Enter a valid PIN with at least " +
                  base::UTF8ToUTF16(std::to_string(minimum)) +
                  u" characters (at most 63 UTF-8 bytes).");
              return;
            }
            if (dialog->confirmation_ &&
                pin != dialog->confirmation_->GetText()) {
              dialog->validation_->SetText(u"The PINs do not match.");
              return;
            }
            dialog->pin_->SetText(u"");
            if (dialog->confirmation_) {
              dialog->confirmation_->SetText(u"");
            }
            // Dispatch may synchronously destroy the request delegate. Defer
            // widget destruction while its button callback stack is still
            // running.
            dialog->CloseWithoutCancelling();
            std::move(*callback).Run(std::move(pin));
            return;
          },
          GetWeakPtr(), minimum_length,
          base::Owned(new base::OnceCallback<void(std::u16string)>(
              std::move(callback)))));
}

}  // namespace tuffite
