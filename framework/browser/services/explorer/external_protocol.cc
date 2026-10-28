// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/explorer/external_protocol.h"

#include "base/functional/bind.h"
#include "base/functional/callback_helpers.h"
#include "base/logging.h"
#include "base/process/launch.h"
#include "base/strings/utf_string_conversions.h"
#include "base/task/single_thread_task_runner.h"
#include "base/task/thread_pool.h"
#include "build/build_config.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/weak_document_ptr.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/common/external_protocol.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/browser_context.h"
#include "ui/base/models/dialog_model.h"
#include "ui/views/bubble/bubble_dialog_model_host.h"
#include "ui/views/widget/widget.h"

#if BUILDFLAG(IS_WIN)
#include <windows.h>

#include <shellapi.h>

#include "base/base_paths.h"
#include "base/path_service.h"
#endif

namespace tuffite {
namespace {
void LaunchExternal(const GURL& url) {
#if BUILDFLAG(IS_WIN)
  base::ThreadPool::CreateCOMSTATaskRunner({base::MayBlock()})
      ->PostTask(FROM_HERE,
                 base::BindOnce(
                     [](const GURL& url) {
                       base::FilePath system;
                       if (!base::PathService::Get(base::DIR_SYSTEM, &system)) {
                         return;
                       }
                       const auto quoted =
                           base::UTF8ToWide("\"" + url.spec() + "\"");
                       ShellExecuteW(nullptr, L"open", quoted.c_str(), nullptr,
                                     system.value().c_str(), SW_SHOWNORMAL);
                     },
                     url));
#else
  base::ThreadPool::PostTask(
      FROM_HERE, {base::MayBlock()},
      base::BindOnce(
          [](const GURL& url) {
            base::LaunchOptions options;
            options.current_directory = base::FilePath("/");
#if BUILDFLAG(IS_MAC)
            base::LaunchProcess({"/usr/bin/open", "--", url.spec()}, options);
#else
            base::LaunchProcess({"/usr/bin/xdg-open", url.spec()}, options);
#endif
          },
          url));
#endif
}
}  // namespace

void ConfirmExternalProtocol(
    content::WebContents* contents,
    const GURL& url,
    const std::optional<url::Origin>& initiating_origin,
    content::RenderFrameHost* initiator_document) {
  if (!contents ||
      !IsExternalProtocol(
          url, static_cast<BrowserContext*>(contents->GetBrowserContext())
                   ->manifest_service()
                   .app_info()
                   .schemes) ||
      contents->GetVisibility() != content::Visibility::VISIBLE) {
    VLOG(1) << "External protocol confirmation suppressed: page is not visible";
    return;
  }
  auto document = (initiator_document ? initiator_document
                                      : contents->GetPrimaryMainFrame())
                      ->GetWeakDocumentPtr();
  if (ExplorerHost::DelegateExternalProtocol(
          contents, url,
          initiating_origin && !initiating_origin->opaque()
              ? initiating_origin->Serialize()
              : "",
          document.AsRenderFrameHostIfValid(),
          base::BindOnce(
              [](content::WeakDocumentPtr document, GURL url) {
                if (document.AsRenderFrameHostIfValid()) {
                  LaunchExternal(url);
                }
              },
              document, url))) {
    return;
  }
  while (contents->GetOuterWebContents()) {
    contents = contents->GetOuterWebContents();
  }
  auto* window =
      views::Widget::GetTopLevelWidgetForNativeView(contents->GetNativeView());
  if (!window || !window->IsVisible()) {
    VLOG(1)
        << "External protocol confirmation suppressed: no visible owner widget";
    return;
  }
  const std::string origin = initiating_origin && !initiating_origin->opaque()
                                 ? initiating_origin->Serialize()
                                 : "This page";
  auto model =
      ui::DialogModel::Builder()
          .SetTitle(u"Open an external application?")
          .AddParagraph(ui::DialogModelLabel(base::UTF8ToUTF16(
              origin + " wants to open " + std::string(url.scheme()) + ":\n" +
              url.spec())))
          .AddOkButton(
              base::BindOnce(
                  [](content::WeakDocumentPtr document, GURL url) {
                    if (document.AsRenderFrameHostIfValid()) {
                      LaunchExternal(url);
                    }
                  },
                  document, url),
              ui::DialogModel::Button::Params().SetLabel(u"Open application"))
          .AddCancelButton(base::DoNothing())
          .OverrideDefaultButton(ui::mojom::DialogButton::kCancel)
          .Build();
  auto host = views::BubbleDialogModelHost::CreateModal(
      std::move(model), ui::mojom::ModalType::kWindow);
  views::DialogDelegate::CreateDialogWidget(
      std::move(host), window->GetNativeWindow(), window->GetNativeView())
      ->Show();
}
}  // namespace tuffite
