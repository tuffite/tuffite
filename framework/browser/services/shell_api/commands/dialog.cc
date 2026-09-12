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
#include "tuffite/framework/browser/services/shell_api/commands/application_policy.h"
#include "tuffite/framework/browser/services/shell_api/commands/registry.h"
#include "tuffite/framework/browser/web_contents_role.h"
#include "tuffite/framework/common/app_manifest.h"
#include "ui/shell_dialogs/select_file_dialog.h"
#include "ui/shell_dialogs/select_file_policy.h"
#include "ui/shell_dialogs/selected_file_info.h"

namespace tuffite::shell_api::dialog {

using generated::OpenDirectoryDialogOptions;
using generated::OpenFileDialogOptions;
using generated::dialog::OpenDirectoryCallback;
using generated::dialog::OpenDirectoryRequest;
using generated::dialog::OpenFileCallback;
using generated::dialog::OpenFileRequest;

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
      std::move(callback_).Run(kStatusUnavailable, {});
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
    std::move(callback_).Run(kStatusOk, std::move(path));
    Release();
  }

  SelectionCallback callback_;
  scoped_refptr<ui::SelectFileDialog> dialog_;
  std::unique_ptr<ui::SelectFileDialog::FileTypeInfo> file_types_;
};

bool IsAuthorized(content::RenderFrameHost& frame, bool file) {
  const AppManifest* manifest = ManifestFor(frame);
  content::WebContents* web_contents =
      content::WebContents::FromRenderFrameHost(&frame);
  return manifest && web_contents &&
         WebContentsRoleData::GetRole(web_contents) == WebContentsRole::kApp &&
         &frame == web_contents->GetPrimaryMainFrame() &&
         manifest->IsShellApiCommandAllowed(
             frame.GetLastCommittedOrigin(),
             file ? "tuffite.dialog.openFile" : "tuffite.dialog.openDirectory");
}

void Show(content::RenderFrameHost& frame,
          bool file,
          std::string title,
          std::string default_path,
          std::optional<std::vector<std::vector<std::string>>> extensions,
          SelectionCallback callback) {
  if (!IsAuthorized(frame, file)) {
    std::move(callback).Run(kStatusAccessDenied, {});
    return;
  }
  if (g_dialog_active) {
    std::move(callback).Run(kStatusUnavailable, {});
    return;
  }
  if (title.size() > 512 || default_path.size() > 32 * 1024) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  auto file_types = file ? MakeFileTypes(extensions) : nullptr;
  if (file && extensions && !file_types) {
    std::move(callback).Run(kStatusInvalidArgument, {});
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

void OpenFile(const ::tuffite::dispatch::Context& context,
              OpenFileRequest&& request,
              OpenFileCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  OpenFileDialogOptions options =
      request.options.value_or(OpenFileDialogOptions());
  Show(context.frame(), true, options.title.value_or(""),
       options.default_path.value_or(""), std::move(options.extensions),
       std::move(callback));
}

void OpenDirectory(const ::tuffite::dispatch::Context& context,
                   OpenDirectoryRequest&& request,
                   OpenDirectoryCallback&& callback) {
  if (!context.sender) {
    std::move(callback).Run(kStatusInvalidArgument, {});
    return;
  }
  OpenDirectoryDialogOptions options =
      request.options.value_or(OpenDirectoryDialogOptions());
  Show(context.frame(), false, options.title.value_or(""), "", std::nullopt,
       std::move(callback));
}

}  // namespace tuffite::shell_api::dialog
