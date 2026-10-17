// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include <algorithm>

#include "base/atomic_sequence_num.h"
#include "base/functional/callback_helpers.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/strings/utf_string_conversions.h"
#include "content/public/browser/web_contents.h"
#include "third_party/blink/public/common/page/page_zoom.h"
#include "third_party/blink/public/mojom/favicon/favicon_url.mojom.h"
#include "third_party/blink/public/mojom/frame/find_in_page.mojom.h"
#include "tuffite/components/explorer/browser/explorer_host.h"
#include "tuffite/components/explorer/browser/explorer_router.h"

namespace tuffite {

void ExplorerHost::FindInPage(const std::string& text,
                              bool forward,
                              bool match_case,
                              bool find_next,
                              FindInPageCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  if (!contents || contents->IsCrashed() || !render_frame_host().IsActive() ||
      text.empty() || text.size() > 64 * 1024) {
    std::move(callback).Run(-1);
    return;
  }
  static base::AtomicSequenceNumber ids;
  active_find_request_id_ = ids.GetNext();
  auto options = blink::mojom::FindOptions::New();
  options->forward = forward;
  options->match_case = match_case;
  options->new_session =
      !find_next || find_text_ != text || find_match_case_ != match_case;
  find_text_ = text;
  find_match_case_ = match_case;
  // Publish the accepted id before the renderer's asynchronous result updates.
  std::move(callback).Run(active_find_request_id_);
  contents->Find(active_find_request_id_, base::UTF8ToUTF16(text),
                 std::move(options), true);
}

void ExplorerHost::StopFindInPage(const std::string& action,
                                  StopFindInPageCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  content::StopFindAction native_action;
  if (action == "clear") {
    native_action = content::STOP_FIND_ACTION_CLEAR_SELECTION;
  } else if (action == "keep") {
    native_action = content::STOP_FIND_ACTION_KEEP_SELECTION;
  } else if (action == "activate") {
    native_action = content::STOP_FIND_ACTION_ACTIVATE_SELECTION;
  } else {
    std::move(callback).Run(false);
    return;
  }
  if (!contents || !render_frame_host().IsActive()) {
    std::move(callback).Run(false);
    return;
  }
  active_find_request_id_ = -1;
  find_text_.clear();
  contents->StopFinding(native_action);
  std::move(callback).Run(true);
}

void ExplorerHost::FindReply(content::WebContents* contents,
                             int request_id,
                             int matches,
                             const gfx::Rect& rect,
                             int ordinal,
                             bool final_update) {
  if (!client_ || contents != WebContentsObserver::web_contents() ||
      request_id != active_find_request_id_) {
    return;
  }
  client_->DidFindResult(
      base::WriteJson(
          base::DictValue()
              .Set("requestId", request_id)
              .Set("matches", matches)
              .Set("activeMatchOrdinal", ordinal)
              .Set("finalUpdate", final_update)
              .Set("selectionRect", base::DictValue()
                                        .Set("x", rect.x())
                                        .Set("y", rect.y())
                                        .Set("width", rect.width())
                                        .Set("height", rect.height())))
          .value_or("{}"));
}

void ExplorerHost::SetZoomFactor(double factor,
                                 SetZoomFactorCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  const double old_factor =
      contents ? ExplorerSurfaceData::GetZoomFactor(contents) : 1.0;
  const bool accepted = contents && render_frame_host().IsActive() &&
                        ExplorerSurfaceData::SetZoomFactor(contents, factor);
  if (accepted && client_ && old_factor != factor) {
    auto state =
        base::JSONReader::ReadDict(NavigationStateJson(), base::JSON_PARSE_RFC)
            .value_or(base::DictValue());
    state.Set("oldFactor", old_factor);
    state.Set("newFactor", factor);
    client_->DidChangePageState("zoomchange",
                                base::WriteJson(state).value_or("{}"));
  }
  std::move(callback).Run(accepted);
}
void ExplorerHost::GetZoomFactor(GetZoomFactorCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  std::move(callback).Run(
      contents ? ExplorerSurfaceData::GetZoomFactor(contents) : 1.0);
}
void ExplorerHost::ContentsZoomChange(bool zoom_in) {
  auto* contents = WebContentsObserver::web_contents();
  if (!contents) {
    return;
  }
  const double factor = ExplorerSurfaceData::GetZoomFactor(contents);
  SetZoomFactor(std::clamp(factor * (zoom_in ? 1.2 : 1.0 / 1.2), 0.25, 5.0),
                base::DoNothing());
}
void ExplorerHost::SetAudioMuted(bool muted, SetAudioMutedCallback callback) {
  auto* contents = WebContentsObserver::web_contents();
  const bool accepted = contents && render_frame_host().IsActive();
  if (accepted) {
    contents->SetAudioMuted(muted);
  }
  std::move(callback).Run(accepted);
}
void ExplorerHost::OnAudioStateChanged(bool audible) {
  if (client_) {
    client_->DidChangePageState("audiostatechange", NavigationStateJson());
  }
}
void ExplorerHost::DidUpdateAudioMutingState(bool muted) {
  if (client_) {
    client_->DidChangePageState("audiostatechange", NavigationStateJson());
  }
}
void ExplorerHost::PrimaryMainFrameRenderProcessGone(
    base::TerminationStatus status) {
  active_find_request_id_ = -1;
  find_text_.clear();
  if (dialog_manager_) {
    GetJavaScriptDialogManager(WebContentsObserver::web_contents())
        ->CancelDialogs(nullptr, true);
  }
  if (client_) {
    auto state =
        base::JSONReader::ReadDict(NavigationStateJson(), base::JSON_PARSE_RFC)
            .value_or(base::DictValue());
    state.Set("terminationStatus", static_cast<int>(status));
    const char* reason = "abnormal";
    switch (status) {
      case base::TERMINATION_STATUS_NORMAL_TERMINATION:
        reason = "normal";
        break;
      case base::TERMINATION_STATUS_PROCESS_WAS_KILLED:
        reason = "killed";
        break;
      case base::TERMINATION_STATUS_PROCESS_CRASHED:
        reason = "crashed";
        break;
      case base::TERMINATION_STATUS_LAUNCH_FAILED:
        reason = "launch-failed";
        break;
      case base::TERMINATION_STATUS_OOM:
        reason = "oom";
        break;
      case base::TERMINATION_STATUS_EVICTED_FOR_MEMORY:
        reason = "evicted";
        break;
      default:
        break;
    }
    state.Set("reason", reason);
    client_->DidChangePageState("renderprocessgone",
                                base::WriteJson(state).value_or("{}"));
  }
}
void ExplorerHost::DidUpdateFaviconURL(
    content::RenderFrameHost* frame,
    const std::vector<blink::mojom::FaviconURLPtr>& candidates,
    blink::mojom::FaviconUpdateReason reason) {
  auto* contents = WebContentsObserver::web_contents();
  if (!contents || frame != contents->GetPrimaryMainFrame()) {
    return;
  }
  base::ListValue icons;
  for (const auto& candidate : candidates) {
    base::ListValue sizes;
    for (const auto& size : candidate->icon_sizes) {
      sizes.Append(base::DictValue()
                       .Set("width", size.width())
                       .Set("height", size.height()));
    }
    icons.Append(base::DictValue()
                     .Set("url", candidate->icon_url.spec())
                     .Set("type", static_cast<int>(candidate->icon_type))
                     .Set("sizes", std::move(sizes))
                     .Set("isDefault", candidate->is_default_icon));
  }
  ExplorerSurfaceData::SetFavicons(contents, std::move(icons));
  if (client_) {
    client_->DidChangePageState("faviconchange", NavigationStateJson());
  }
}

void ExplorerHost::DidToggleFullscreenModeForTab(bool entered, bool) {
  if (client_) {
    client_->DidChangePageState(
        "fullscreenchange",
        base::WriteJson(base::DictValue().Set("entered", entered))
            .value_or("{}"));
  }
}
void ExplorerHost::MediaPictureInPictureChanged(bool entered) {
  NotifyPictureInPicture(WebContentsObserver::web_contents(), "video", entered);
}
void ExplorerHost::NotifyPictureInPicture(content::WebContents* contents,
                                          const std::string& type,
                                          bool entered) {
  auto* host = ExplorerRouter::Get(contents).FindHostForContents(contents);
  if (host && host->client_) {
    host->client_->DidChangePageState(
        "pictureinpicturechange",
        base::WriteJson(
            base::DictValue().Set("type", type).Set("entered", entered))
            .value_or("{}"));
  }
}
void ExplorerHost::ActivateContents(content::WebContents* contents) {
  auto* owner = content::WebContents::FromRenderFrameHost(&render_frame_host());
  if (owner && owner->GetDelegate() && owner->GetDelegate() != this) {
    owner->GetDelegate()->ActivateContents(contents);
  }
}
bool ExplorerHost::HandleKeyboardEvent(
    content::WebContents* source,
    const input::NativeWebKeyboardEvent& event) {
  auto* owner = content::WebContents::FromRenderFrameHost(&render_frame_host());
  return owner && owner->GetDelegate() && owner->GetDelegate() != this &&
         owner->GetDelegate()->HandleKeyboardEvent(source, event);
}
std::optional<gfx::Rect> ExplorerHost::GetWindowBoundsInScreen() {
  auto* owner = content::WebContents::FromRenderFrameHost(&render_frame_host());
  return owner && owner->GetDelegate() && owner->GetDelegate() != this
             ? owner->GetDelegate()->GetWindowBoundsInScreen()
             : std::nullopt;
}
}  // namespace tuffite
