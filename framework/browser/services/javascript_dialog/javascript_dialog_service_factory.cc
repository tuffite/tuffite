// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/javascript_dialog/javascript_dialog_service_factory.h"

#include <map>

#include "base/functional/bind.h"
#include "base/memory/weak_ptr.h"
#include "base/strings/utf_string_conversions.h"
#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/browser/services/javascript_dialog/javascript_dialog_service.h"
#include "ui/base/models/dialog_model.h"
#include "ui/views/bubble/bubble_dialog_model_host.h"
#include "ui/views/controls/message_box_view.h"
#include "ui/views/layout/fill_layout.h"
#include "ui/views/widget/widget.h"
#include "url/origin.h"

namespace tuffite {
namespace {
class DialogView final : public views::View {
 public:
  DialogView(const std::u16string& message,
             const std::u16string& prompt,
             bool is_prompt,
             content::JavaScriptDialogManager::DialogClosedCallback callback)
      : callback_(std::move(callback)) {
    SetUseDefaultFillLayout(true);
    message_ = AddChildView(
        std::make_unique<views::MessageBoxView>(message.substr(0, 4096), true));
    message_->SetMessageWidth(380);
    if (is_prompt) {
      message_->SetPromptField(prompt);
    }
  }
  base::WeakPtr<DialogView> GetWeakPtr() { return weak_factory_.GetWeakPtr(); }
  void Finish(bool accept, const std::u16string* override_text = nullptr) {
    if (!callback_) {
      return;
    }
    std::u16string input =
        accept ? (override_text ? *override_text
                                : std::u16string(message_->GetInputText()))
               : std::u16string();
    auto callback = std::move(callback_);
    if (GetWidget()) {
      GetWidget()->Close();
    }
    std::move(callback).Run(accept, input);
  }

 private:
  raw_ptr<views::MessageBoxView> message_ = nullptr;
  content::JavaScriptDialogManager::DialogClosedCallback callback_;
  base::WeakPtrFactory<DialogView> weak_factory_{this};
};

class JavaScriptDialogServiceImpl final : public JavaScriptDialogService {
 public:
  void RunJavaScriptDialog(content::WebContents* contents,
                           content::RenderFrameHost* frame,
                           content::JavaScriptDialogType type,
                           const std::u16string& message,
                           const std::u16string& prompt,
                           DialogClosedCallback callback,
                           bool* suppressed) override {
    *suppressed = shutting_down_ || dialogs_.contains(contents) ||
                  contents->GetVisibility() != content::Visibility::VISIBLE ||
                  !contents->GetTopLevelNativeWindow();
    if (*suppressed) {
      return;
    }
    Show(contents, frame, type, message, prompt, std::move(callback));
  }
  void RunBeforeUnloadDialog(content::WebContents* contents,
                             content::RenderFrameHost* frame,
                             bool is_reload,
                             DialogClosedCallback callback) override {
    if (shutting_down_ || !contents->GetTopLevelNativeWindow()) {
      std::move(callback).Run(false, u"");
      return;
    }
    CancelDialogs(contents, false);
    Show(contents, frame, content::JAVASCRIPT_DIALOG_TYPE_CONFIRM,
         u"Leave this page? Changes you made may not be saved.", u"",
         std::move(callback));
  }
  bool HandleJavaScriptDialog(content::WebContents* contents,
                              bool accept,
                              const std::u16string* override_text) override {
    auto it = dialogs_.find(contents);
    if (it == dialogs_.end() || !it->second) {
      return false;
    }
    it->second->Finish(accept, override_text);
    return true;
  }
  void CancelDialogs(content::WebContents* contents,
                     bool reset_state) override {
    HandleJavaScriptDialog(contents, false, nullptr);
  }
  void Shutdown() override {
    shutting_down_ = true;
    while (!dialogs_.empty()) {
      auto it = dialogs_.begin();
      if (it->second) {
        it->second->Finish(false);
      } else {
        dialogs_.erase(it);
      }
    }
  }

 private:
  void Show(content::WebContents* contents,
            content::RenderFrameHost* frame,
            content::JavaScriptDialogType type,
            const std::u16string& message,
            const std::u16string& prompt,
            DialogClosedCallback callback) {
    auto view = std::make_unique<DialogView>(
        message, prompt, type == content::JAVASCRIPT_DIALOG_TYPE_PROMPT,
        base::BindOnce(
            [](base::WeakPtr<JavaScriptDialogServiceImpl> service,
               content::WebContents* contents, DialogClosedCallback callback,
               bool accept, const std::u16string& input) {
              if (service) {
                service->dialogs_.erase(contents);
              }
              std::move(callback).Run(accept, input);
            },
            weak_factory_.GetWeakPtr(), contents, std::move(callback)));
    auto weak_view = view->GetWeakPtr();
    dialogs_[contents] = weak_view;
    auto cancel = base::BindRepeating(
        [](base::WeakPtr<DialogView> view) {
          if (view) {
            view->Finish(false);
          }
        },
        weak_view);
    ui::DialogModel::Builder builder;
    builder
        .SetTitle(
            base::UTF8ToUTF16(frame->GetLastCommittedOrigin().Serialize()))
        .DisableCloseOnDeactivate()
        .AddOkButton(base::BindRepeating(
            [](base::WeakPtr<DialogView> view) {
              if (view) {
                view->Finish(true);
              }
            },
            weak_view))
        .SetCloseActionCallback(cancel)
        .SetDialogDestroyingCallback(cancel)
        .AddCustomField(
            std::make_unique<views::BubbleDialogModelHost::CustomView>(
                std::move(view),
                views::BubbleDialogModelHost::FieldType::kControl));
    if (type != content::JAVASCRIPT_DIALOG_TYPE_ALERT) {
      builder.AddCancelButton(cancel);
    }
    auto host = views::BubbleDialogModelHost::CreateModal(
        builder.Build(), ui::mojom::ModalType::kWindow);
    views::DialogDelegate::CreateDialogWidget(
        std::move(host), contents->GetTopLevelNativeWindow(),
        contents->GetNativeView())
        ->Show();
  }
  bool shutting_down_ = false;
  std::map<content::WebContents*, base::WeakPtr<DialogView>> dialogs_;
  base::WeakPtrFactory<JavaScriptDialogServiceImpl> weak_factory_{this};
};
}  // namespace

JavaScriptDialogServiceFactory* JavaScriptDialogServiceFactory::Get() {
  static base::NoDestructor<JavaScriptDialogServiceFactory> instance;
  return instance.get();
}
JavaScriptDialogServiceFactory::JavaScriptDialogServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteJavaScriptDialog",
          BrowserContextDependencyManager::GetInstance()) {}
JavaScriptDialogService* JavaScriptDialogServiceFactory::GetForBrowserContext(
    content::BrowserContext* context,
    bool create) {
  return static_cast<JavaScriptDialogService*>(
      Get()->GetServiceForBrowserContext(context, create));
}
std::unique_ptr<KeyedService>
JavaScriptDialogServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext* context) const {
  return std::make_unique<JavaScriptDialogServiceImpl>();
}
}  // namespace tuffite
