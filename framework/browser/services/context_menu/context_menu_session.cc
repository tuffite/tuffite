// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/context_menu/context_menu_session.h"

#include <memory>

#include "base/memory/raw_ptr.h"
#include "base/memory/raw_ref.h"
#include "base/strings/utf_string_conversions.h"
#include "build/build_config.h"
#include "content/public/browser/context_menu_params.h"
#include "content/public/browser/global_routing_id.h"
#include "content/public/browser/navigation_controller.h"
#include "content/public/browser/page_navigator.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/render_widget_host_view.h"
#include "content/public/browser/web_contents.h"
#include "third_party/blink/public/common/context_menu_data/context_menu_data.h"
#include "third_party/blink/public/common/context_menu_data/edit_flags.h"
#include "third_party/blink/public/mojom/context_menu/context_menu.mojom.h"
#include "third_party/blink/public/mojom/frame/media_player_action.mojom.h"
#include "tuffite/framework/app/services/policy/application_policy_service.h"
#include "tuffite/framework/browser/services/devtools/devtools_service.h"
#include "ui/base/clipboard/scoped_clipboard_writer.h"
#if defined(USE_AURA)
#include "ui/aura/client/screen_position_client.h"
#include "ui/aura/window.h"
#endif
#include "ui/base/models/menu_separator_types.h"
#include "ui/base/mojom/menu_source_type.mojom.h"
#include "ui/base/page_transition_types.h"
#include "ui/base/window_open_disposition.h"
#include "ui/gfx/geometry/rect.h"
#include "ui/menus/simple_menu_model.h"
#include "ui/views/controls/menu/menu_runner.h"
#include "ui/views/widget/widget.h"

namespace tuffite {
namespace {

enum CommandId {
  kBack = 1,
  kForward,
  kReload,
  kOpenLink,
  kCut,
  kCopy,
  kPaste,
  kSelectAll,
  kInspectElement,
  kCopyLinkAddress,
  kOpenImage,
  kSaveImage,
  kCopyImage,
  kCopyImageAddress,
  kPictureInPicture,
};

class TuffiteContextMenu final : public ui::SimpleMenuModel::Delegate {
 public:
  TuffiteContextMenu(content::WebContents* web_contents,
                     content::RenderFrameHost& render_frame_host,
                     const content::ContextMenuParams& params,
                     DevToolsService& devtools)
      : devtools_(devtools),
        web_contents_(web_contents),
        frame_id_(render_frame_host.GetGlobalId()),
        params_(params) {
    model_ = std::make_unique<ui::SimpleMenuModel>(this);
    model_->AddItem(kBack, u"Back");
    model_->AddItem(kForward, u"Forward");
    model_->AddItem(kReload, u"Reload");
    if (params_.link_url.SchemeIsHTTPOrHTTPS()) {
      model_->AddSeparator(ui::NORMAL_SEPARATOR);
      model_->AddItem(kOpenLink, u"Open link in new Browser tab");
      model_->AddItem(kCopyLinkAddress, u"Copy link address");
    }
    if (params_.media_type == blink::mojom::ContextMenuDataMediaType::kImage) {
      model_->AddSeparator(ui::NORMAL_SEPARATOR);
      model_->AddItem(kOpenImage, u"Open image in new Browser tab");
      model_->AddItem(kSaveImage, u"Save image");
      model_->AddItem(kCopyImage, u"Copy image");
      model_->AddItem(kCopyImageAddress, u"Copy image address");
    }
    if (params_.media_type == blink::mojom::ContextMenuDataMediaType::kVideo) {
      model_->AddSeparator(ui::NORMAL_SEPARATOR);
      model_->AddItem(kPictureInPicture, u"Picture in picture");
    }
    if (params_.is_editable || !params_.selection_text.empty()) {
      model_->AddSeparator(ui::NORMAL_SEPARATOR);
      if (params_.is_editable) {
        model_->AddItem(kCut, u"Cut");
      }
      model_->AddItem(kCopy, u"Copy");
      if (params_.is_editable) {
        model_->AddItem(kPaste, u"Paste");
      }
      model_->AddItem(kSelectAll, u"Select all");
    }
    model_->AddSeparator(ui::NORMAL_SEPARATOR);
    model_->AddItem(kInspectElement, u"Inspect element");
  }

  TuffiteContextMenu(const TuffiteContextMenu&) = delete;
  TuffiteContextMenu& operator=(const TuffiteContextMenu&) = delete;
  ~TuffiteContextMenu() override = default;

  ui::SimpleMenuModel* model() const { return model_.get(); }

  bool IsCommandIdEnabled(int command_id) const override {
    if (!web_contents_) {
      return false;
    }
    switch (command_id) {
      case kCopyImage:
        return params_.has_image_contents &&
               content::RenderFrameHost::FromID(frame_id_);
      case kOpenImage:
      case kCopyImageAddress:
      case kSaveImage:
        return params_.src_url.is_valid();
      case kPictureInPicture:
        return (params_.media_flags &
                blink::ContextMenuData::kMediaCanPictureInPicture) &&
               content::RenderFrameHost::FromID(frame_id_);
      case kBack:
        return web_contents_->GetController().CanGoBack();
      case kForward:
        return web_contents_->GetController().CanGoForward();
      case kCut:
        return params_.edit_flags & blink::ContextMenuDataEditFlags::kCanCut;
      case kCopy:
        return params_.edit_flags & blink::ContextMenuDataEditFlags::kCanCopy;
      case kPaste:
        return params_.edit_flags & blink::ContextMenuDataEditFlags::kCanPaste;
      case kSelectAll:
        return params_.edit_flags &
               blink::ContextMenuDataEditFlags::kCanSelectAll;
      default:
        return true;
    }
  }

  void ExecuteCommand(int command_id, int event_flags) override {
    (void)event_flags;
    if (!web_contents_) {
      return;
    }
    switch (command_id) {
      case kBack:
        web_contents_->GetController().GoBack();
        break;
      case kForward:
        web_contents_->GetController().GoForward();
        break;
      case kReload:
        web_contents_->GetController().Reload(content::ReloadType::NORMAL,
                                              true);
        break;
      case kOpenLink: {
        content::OpenURLParams open_params(
            params_.link_url, content::Referrer(),
            WindowOpenDisposition::NEW_FOREGROUND_TAB, ui::PAGE_TRANSITION_LINK,
            false, true);
        web_contents_->OpenURL(open_params, {});
        break;
      }
      case kCopyLinkAddress:
      case kCopyImageAddress: {
        ui::ScopedClipboardWriter writer(ui::ClipboardBuffer::kCopyPaste);
        writer.WriteText(
            base::UTF8ToUTF16((command_id == kCopyLinkAddress ? params_.link_url
                                                              : params_.src_url)
                                  .spec()));
        break;
      }
      case kOpenImage: {
        content::OpenURLParams open_params(
            params_.src_url, content::Referrer(),
            WindowOpenDisposition::NEW_FOREGROUND_TAB, ui::PAGE_TRANSITION_LINK,
            false, true);
        web_contents_->OpenURL(open_params, {});
        break;
      }
      case kSaveImage:
        if (auto* frame = content::RenderFrameHost::FromID(frame_id_)) {
          web_contents_->SaveFrameWithHeaders(
              params_.src_url, content::Referrer(), "", u"", frame, true);
        }
        break;
      case kCopyImage:
        if (auto* frame = content::RenderFrameHost::FromID(frame_id_)) {
          frame->CopyImageAt(params_.x, params_.y);
        }
        break;
      case kPictureInPicture:
        if (auto* frame = content::RenderFrameHost::FromID(frame_id_)) {
          frame->ExecuteMediaPlayerActionAtLocation(
              gfx::Point(params_.x, params_.y),
              blink::mojom::MediaPlayerAction(
                  blink::mojom::MediaPlayerActionType::kPictureInPicture,
                  !(params_.media_flags &
                    blink::ContextMenuData::kMediaPictureInPicture)));
        }
        break;
      case kCut:
        web_contents_->Cut();
        break;
      case kCopy:
        web_contents_->Copy();
        break;
      case kPaste:
        web_contents_->Paste();
        break;
      case kSelectAll:
        web_contents_->SelectAll();
        break;
      case kInspectElement:
        devtools_->InspectElement(web_contents_,
                                  content::RenderFrameHost::FromID(frame_id_),
                                  params_.x, params_.y);
        break;
    }
  }

 private:
  const raw_ref<DevToolsService> devtools_;
  raw_ptr<content::WebContents> web_contents_;
  content::GlobalRenderFrameHostId frame_id_;
  content::ContextMenuParams params_;
  std::unique_ptr<ui::SimpleMenuModel> model_;
};

class ContextMenuSessionImpl final : public ContextMenuSession {
 public:
  ContextMenuSessionImpl(content::WebContents* web_contents,
                         ApplicationPolicyService& policy,
                         DevToolsService& devtools)
      : web_contents_(web_contents), policy_(policy), devtools_(devtools) {}
  ~ContextMenuSessionImpl() override = default;

  void Show(content::RenderFrameHost& render_frame_host,
            const content::ContextMenuParams& params) override {
    if (!policy_->ContextMenuAllowed(web_contents_)) {
      return;
    }
    if (context_menu_runner_ && context_menu_runner_->IsRunning()) {
      context_menu_runner_->Cancel();
      return;
    }
    context_menu_runner_.reset();
    context_menu_ = std::make_unique<TuffiteContextMenu>(
        web_contents_, render_frame_host, params, *devtools_);
    context_menu_runner_ = std::make_unique<views::MenuRunner>(
        context_menu_->model(), views::MenuRunner::CONTEXT_MENU);

    // ContextMenuParams coordinates have already been transformed from the
    // source RenderFrameHost into the outermost WebContents' root coordinate
    // space, including every nested Explorer offset. Convert through that
    // root's native view/container; an inner container's screen origin would
    // add the Explorer offsets a second time on platforms without Aura.
    content::WebContents* outermost = web_contents_->GetOutermostWebContents();
    gfx::Point screen_point(params.x, params.y);
#if defined(USE_AURA)
    aura::Window* native_view = outermost->GetNativeView();
    aura::Window* root_window =
        native_view ? native_view->GetRootWindow() : nullptr;
    aura::client::ScreenPositionClient* screen_position_client =
        root_window ? aura::client::GetScreenPositionClient(root_window)
                    : nullptr;
    if (screen_position_client) {
      screen_position_client->ConvertPointToScreen(native_view, &screen_point);
    } else {
      const gfx::Point container_origin =
          outermost->GetContainerBounds().origin();
      screen_point.Offset(container_origin.x(), container_origin.y());
    }
#else
    const gfx::Point container_origin =
        outermost->GetContainerBounds().origin();
    screen_point.Offset(container_origin.x(), container_origin.y());
#endif
    views::Widget* widget = views::Widget::GetWidgetForNativeWindow(
        web_contents_->GetTopLevelNativeWindow());
    if (!widget) {
      return;
    }
    context_menu_runner_->RunMenuAt(
        widget, nullptr, gfx::Rect(screen_point, gfx::Size()),
        views::MenuAnchorPosition::kTopLeft, params.source_type,
        web_contents_->GetNativeView());
  }

  void Dismiss() override {
    if (context_menu_runner_ && context_menu_runner_->IsRunning()) {
      context_menu_runner_->Cancel();
    }
    context_menu_runner_.reset();
    context_menu_.reset();
  }

 private:
  raw_ptr<content::WebContents> web_contents_;
  const raw_ref<ApplicationPolicyService> policy_;
  const raw_ref<DevToolsService> devtools_;
  std::unique_ptr<TuffiteContextMenu> context_menu_;
  std::unique_ptr<views::MenuRunner> context_menu_runner_;
};

}  // namespace

std::unique_ptr<ContextMenuSession> CreateContextMenuSession(
    content::WebContents* web_contents,
    ApplicationPolicyService& policy,
    DevToolsService& devtools) {
  return std::make_unique<ContextMenuSessionImpl>(web_contents, policy,
                                                  devtools);
}

}  // namespace tuffite
