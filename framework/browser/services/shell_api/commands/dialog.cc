// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/shell_api/commands/dialog.h"

#include <algorithm>
#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <vector>

#include "base/check.h"
#include "base/files/file_path.h"
#include "base/memory/ref_counted.h"
#include "base/strings/string_util.h"
#include "base/strings/utf_string_conversions.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/shell_api/commands/manifest.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "ui/shell_dialogs/select_file_dialog.h"
#include "ui/shell_dialogs/select_file_policy.h"
#include "ui/shell_dialogs/selected_file_info.h"

namespace tuffite::shell_api::dialog {

using generated::OpenDirectoryDialogOptions;
using generated::OpenFileDialogOptions;
using OpenDirectoryCallback = generated::Dialog::OpenDirectoryCallback;
using OpenDirectoryRequestPtr = generated::Dialog::OpenDirectoryRequestPtr;
using OpenFileCallback = generated::Dialog::OpenFileCallback;
using OpenFileRequestPtr = generated::Dialog::OpenFileRequestPtr;

namespace {

constexpr size_t kMaxExtensionGroups = 32;
constexpr size_t kMaxExtensionsPerGroup = 32;
using SelectionCallback =
    base::OnceCallback<void(rpc::Status, std::optional<std::string>)>;

bool g_dialog_active = false;

bool IsSafeExtension(std::string_view extension) {
  return !extension.empty() && extension.size() <= 32 &&
         std::ranges::all_of(extension, [](char character) {
           return base::IsAsciiAlphaNumeric(character) || character == '_' ||
                  character == '-';
         });
}

std::unique_ptr<ui::SelectFileDialog::FileTypeInfo> MakeFileTypes(
    const std::optional<std::vector<std::vector<std::string>>>& extensions) {
  if (!extensions) {
    return nullptr;
  }
  if (extensions->size() > kMaxExtensionGroups) {
    return nullptr;
  }
  auto result = std::make_unique<ui::SelectFileDialog::FileTypeInfo>();
  for (const auto& source_group : *extensions) {
    if (source_group.empty() || source_group.size() > kMaxExtensionsPerGroup) {
      return nullptr;
    }
    ui::SelectFileDialog::FileTypeInfo::FileExtensionList group;
    for (const auto& extension : source_group) {
      if (!IsSafeExtension(extension)) {
        return nullptr;
      }
      group.push_back(base::FilePath::FromUTF8Unsafe(extension).value());
    }
    result->extensions.push_back(std::move(group));
  }
  return result;
}

class PendingDialog final : public ui::SelectFileDialog::Listener,
                            public base::RefCounted<PendingDialog> {
 public:
  explicit PendingDialog(SelectionCallback callback)
      : callback_(std::move(callback)) {
    dialog_ = ui::SelectFileDialog::Create(this, nullptr);
  }

  bool Show(ui::SelectFileDialog::Type type,
            std::u16string title,
            base::FilePath default_path,
            std::unique_ptr<ui::SelectFileDialog::FileTypeInfo> file_types,
            gfx::NativeWindow owning_window) {
    if (!dialog_) {
      rpc::Reject(callback_, rpc::Status::kUnavailable);
      return false;
    }
    CHECK(!g_dialog_active);
    g_dialog_active = true;
    AddRef();
    file_types_ = std::move(file_types);
    dialog_->SelectFile(type, title, default_path, file_types_.get(),
                        file_types_ ? 1 : 0, base::FilePath::StringType(),
                        owning_window);
    return true;
  }

  void FileSelected(const ui::SelectedFileInfo& file, int) override {
    Complete(file.path().AsUTF8Unsafe());
  }

  void FileSelectionCanceled() override { Complete(std::nullopt); }

 private:
  friend class base::RefCounted<PendingDialog>;

  ~PendingDialog() override {
    if (dialog_) {
      dialog_->ListenerDestroyed();
    }
  }

  void Complete(std::optional<std::string> path) {
    CHECK(g_dialog_active);
    g_dialog_active = false;
    std::move(callback_).Run(rpc::Status::kOk, std::move(path));
    Release();
  }

  SelectionCallback callback_;
  scoped_refptr<ui::SelectFileDialog> dialog_;
  std::unique_ptr<ui::SelectFileDialog::FileTypeInfo> file_types_;
};

bool IsAuthorized(content::RenderFrameHost& frame, bool file) {
  const ManifestService* manifest = ManifestFor(frame);
  content::WebContents* web_contents =
      content::WebContents::FromRenderFrameHost(&frame);
  return manifest && web_contents &&
         WebContentsRoleData::GetRole(web_contents) == WebContentsRole::kApp &&
         &frame == web_contents->GetPrimaryMainFrame() &&
         manifest->IsShellApiCommandAllowed(
             frame.GetLastCommittedOrigin(),
             file ? "tuffite.Dialog.OpenFile" : "tuffite.Dialog.OpenDirectory");
}

void Show(content::RenderFrameHost& frame,
          bool file,
          std::string title,
          std::string default_path,
          std::optional<std::vector<std::vector<std::string>>> extensions,
          SelectionCallback callback) {
  if (!IsAuthorized(frame, file)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  if (g_dialog_active) {
    rpc::Reject(callback, rpc::Status::kUnavailable);
    return;
  }
  if (title.size() > 512 || default_path.size() > 32 * 1024) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  auto file_types = file ? MakeFileTypes(extensions) : nullptr;
  if (file && extensions && !file_types) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (file_types && file_types->extensions.empty()) {
    file_types.reset();
  }

  content::WebContents* web_contents =
      content::WebContents::FromRenderFrameHost(&frame);
  auto pending = base::MakeRefCounted<PendingDialog>(std::move(callback));
  pending->Show(file ? ui::SelectFileDialog::SELECT_OPEN_FILE
                     : ui::SelectFileDialog::SELECT_EXISTING_FOLDER,
                base::UTF8ToUTF16(title),
                base::FilePath::FromUTF8Unsafe(default_path),
                std::move(file_types), web_contents->GetTopLevelNativeWindow());
}

}  // namespace

void DialogImpl::OpenFile(const dispatch::Context& context,
                          OpenFileRequestPtr request,
                          OpenFileCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (!IsUnscopedAllowed(context, generated::Dialog::kOpenFile)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  auto options = request->options ? std::move(request->options)
                                  : OpenFileDialogOptions::New();
  Show(context.frame(), true, options->title.value_or(""),
       options->default_path.value_or(""), std::move(options->extensions),
       std::move(callback));
}

void DialogImpl::OpenDirectory(const dispatch::Context& context,
                               OpenDirectoryRequestPtr request,
                               OpenDirectoryCallback callback) {
  if (!context.sender()) {
    rpc::Reject(callback, rpc::Status::kInvalidArgument);
    return;
  }
  if (!IsUnscopedAllowed(context, generated::Dialog::kOpenDirectory)) {
    rpc::Reject(callback, rpc::Status::kAccessDenied);
    return;
  }
  auto options = request->options ? std::move(request->options)
                                  : OpenDirectoryDialogOptions::New();
  Show(context.frame(), false, options->title.value_or(""), "", std::nullopt,
       std::move(callback));
}

}  // namespace tuffite::shell_api::dialog
