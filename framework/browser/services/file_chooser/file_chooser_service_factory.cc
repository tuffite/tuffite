// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/file_chooser/file_chooser_service_factory.h"

#include <algorithm>
#include <utility>

#include "base/functional/bind.h"
#include "base/logging.h"
#include "base/memory/weak_ptr.h"
#include "base/strings/string_util.h"
#include "base/strings/utf_string_conversions.h"
#include "components/keyed_service/content/browser_context_dependency_manager.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_observer.h"
#include "net/base/directory_lister.h"
#include "net/base/mime_util.h"
#include "tuffite/framework/browser/services/file_chooser/file_chooser_service.h"
#include "ui/shell_dialogs/select_file_dialog.h"
#include "ui/shell_dialogs/select_file_policy.h"
#include "ui/shell_dialogs/selected_file_info.h"

namespace tuffite {
namespace {
using Mode = blink::mojom::FileChooserParams::Mode;

class ChooserSession final
    : public base::RefCounted<ChooserSession>,
      public content::WebContentsObserver,
      public ui::SelectFileDialog::Listener,
      public net::DirectoryLister::DirectoryListerDelegate {
 public:
  ChooserSession(content::RenderFrameHost* frame,
                 scoped_refptr<content::FileSelectListener> listener)
      : WebContentsObserver(content::WebContents::FromRenderFrameHost(frame)),
        frame_id_(frame->GetGlobalId()),
        listener_(std::move(listener)) {}

  void SetCompletion(base::OnceClosure completion) {
    completion_ = std::move(completion);
  }

  void Show(const blink::mojom::FileChooserParams& params) {
    mode_ = params.mode;
    dialog_ = ui::SelectFileDialog::Create(this, nullptr);
    if (!dialog_) {
      Cancel();
      return;
    }
    file_types_ = std::make_unique<ui::SelectFileDialog::FileTypeInfo>();
    file_types_->include_all_files = true;
    file_types_->allowed_paths =
        ui::SelectFileDialog::FileTypeInfo::NATIVE_PATH;
    ui::SelectFileDialog::FileTypeInfo::FileExtensionList extensions;
    for (const auto& accept : params.accept_types) {
      if (accept.size() > 1 && accept.front() == '.' &&
          accept.find_first_of(u"/\\") == std::u16string::npos) {
        extensions.push_back(
            base::FilePath::FromUTF16Unsafe(accept.substr(1)).value());
      } else if (base::IsStringASCII(accept)) {
        net::GetExtensionsForMimeType(base::UTF16ToASCII(accept), &extensions);
      }
    }
    if (!extensions.empty()) {
      file_types_->extensions.push_back(std::move(extensions));
    }
    ui::SelectFileDialog::Type type;
    switch (mode_) {
      case Mode::kOpen:
        type = ui::SelectFileDialog::SELECT_OPEN_FILE;
        break;
      case Mode::kOpenMultiple:
        type = ui::SelectFileDialog::SELECT_OPEN_MULTI_FILE;
        break;
      case Mode::kUploadFolder:
        type = ui::SelectFileDialog::SELECT_UPLOAD_FOLDER;
        break;
      case Mode::kOpenDirectory:
        type = ui::SelectFileDialog::SELECT_FOLDER;
        break;
      case Mode::kSave:
        type = ui::SelectFileDialog::SELECT_SAVEAS_FILE;
        break;
    }
    VLOG(1) << "File chooser: opened mode=" << static_cast<int>(mode_);
    dialog_->SelectFile(type, params.title, params.default_file_name.BaseName(),
                        file_types_.get(),
                        file_types_->extensions.empty() ? 0 : 1,
                        base::FilePath::StringType(),
                        web_contents()->GetTopLevelNativeWindow());
  }

  void Enumerate(const base::FilePath& path) {
    mode_ = Mode::kUploadFolder;
    base_dir_ = path;
    lister_ = std::make_unique<net::DirectoryLister>(
        path, net::DirectoryLister::NO_SORT_RECURSIVE, this);
    lister_->Start();
  }

  void Cancel() {
    scoped_refptr<ChooserSession> keep_alive(this);
    auto listener = std::move(listener_);
    Finish();
    if (listener) {
      VLOG(1) << "File chooser: canceled";
      listener->FileSelectionCanceled();
    }
  }

  void FileSelected(const ui::SelectedFileInfo& file, int) override {
    if (mode_ == Mode::kUploadFolder) {
      Enumerate(file.local_path);
      return;
    }
    MultiFilesSelected({file});
  }

  void MultiFilesSelected(
      const std::vector<ui::SelectedFileInfo>& files) override {
    for (const auto& file : files) {
      files_.push_back(blink::mojom::FileChooserFileInfo::NewNativeFile(
          blink::mojom::NativeFileInfo::New(
              file.local_path,
              base::FilePath(file.display_name).AsUTF16Unsafe(),
              std::vector<std::u16string>())));
    }
    Complete();
  }

  void FileSelectionCanceled() override { Cancel(); }

  void OnListFile(
      const net::DirectoryLister::DirectoryListerData& data) override {
    if (!data.info.IsDirectory()) {
      files_.push_back(blink::mojom::FileChooserFileInfo::NewNativeFile(
          blink::mojom::NativeFileInfo::New(data.path, std::u16string(),
                                            std::vector<std::u16string>())));
    }
  }

  void OnListDone(int error) override {
    if (error) {
      Cancel();
    } else {
      Complete();
    }
  }

  void WebContentsDestroyed() override { Cancel(); }

  void RenderFrameDeleted(content::RenderFrameHost* frame) override {
    if (frame->GetGlobalId() == frame_id_) {
      Cancel();
    }
  }

  void RenderFrameHostChanged(content::RenderFrameHost* old_frame,
                              content::RenderFrameHost* new_frame) override {
    if (old_frame && old_frame->GetGlobalId() == frame_id_) {
      Cancel();
    }
  }

  void DidFinishNavigation(content::NavigationHandle* navigation) override {
    if (navigation->HasCommitted() && !navigation->IsSameDocument() &&
        navigation->GetRenderFrameHost()->GetGlobalId() == frame_id_) {
      Cancel();
    }
  }

 private:
  friend class base::RefCounted<ChooserSession>;
  ~ChooserSession() override {
    if (dialog_) {
      dialog_->ListenerDestroyed();
    }
    if (listener_) {
      listener_->FileSelectionCanceled();
    }
  }

  void Complete() {
    scoped_refptr<ChooserSession> keep_alive(this);
    auto listener = std::move(listener_);
    auto files = std::move(files_);
    const auto base_dir = base_dir_;
    const auto mode = mode_;
    Finish();
    if (listener) {
      VLOG(1) << "File chooser: selected count=" << files.size();
      listener->FileSelected(std::move(files), base_dir, mode);
    }
  }

  void Finish() {
    Observe(nullptr);
    if (dialog_) {
      dialog_->ListenerDestroyed();
      dialog_.reset();
    }
    lister_.reset();
    if (completion_) {
      std::move(completion_).Run();
    }
  }

  const content::GlobalRenderFrameHostId frame_id_;
  scoped_refptr<content::FileSelectListener> listener_;
  scoped_refptr<ui::SelectFileDialog> dialog_;
  std::unique_ptr<ui::SelectFileDialog::FileTypeInfo> file_types_;
  std::unique_ptr<net::DirectoryLister> lister_;
  base::OnceClosure completion_;
  Mode mode_ = Mode::kOpen;
  base::FilePath base_dir_;
  std::vector<blink::mojom::FileChooserFileInfoPtr> files_;
};

class FileChooserServiceImpl final : public FileChooserService {
 public:
  void Run(content::RenderFrameHost* frame,
           scoped_refptr<content::FileSelectListener> listener,
           const blink::mojom::FileChooserParams& params) override {
    auto session = Create(frame, std::move(listener));
    if (session) {
      session->Show(params);
    }
  }

  void Enumerate(content::WebContents* contents,
                 scoped_refptr<content::FileSelectListener> listener,
                 const base::FilePath& path) override {
    auto session = Create(contents ? contents->GetPrimaryMainFrame() : nullptr,
                          std::move(listener));
    if (session) {
      session->Enumerate(path);
    }
  }

  void Shutdown() override {
    shutting_down_ = true;
    weak_factory_.InvalidateWeakPtrs();
    auto sessions = std::move(sessions_);
    for (const auto& session : sessions) {
      session->Cancel();
    }
  }

 private:
  scoped_refptr<ChooserSession> Create(
      content::RenderFrameHost* frame,
      scoped_refptr<content::FileSelectListener> listener) {
    auto* contents =
        frame ? content::WebContents::FromRenderFrameHost(frame) : nullptr;
    if (shutting_down_ || !frame || !frame->IsActive() ||
        frame->IsNestedWithinFencedFrame() || !contents ||
        std::ranges::any_of(sessions_, [&](const auto& session) {
          return session->web_contents() == contents;
        })) {
      listener->FileSelectionCanceled();
      return nullptr;
    }
    auto session =
        base::MakeRefCounted<ChooserSession>(frame, std::move(listener));
    // The service owns the session, and Finish() holds a local reference while
    // removing it. Retaining it in its own completion would create a cycle.
    session->SetCompletion(base::BindOnce(&FileChooserServiceImpl::Remove,
                                          weak_factory_.GetWeakPtr(),
                                          base::Unretained(session.get())));
    sessions_.push_back(session);
    return session;
  }

  void Remove(ChooserSession* session) {
    std::erase_if(sessions_,
                  [&](const auto& owned) { return owned.get() == session; });
  }

  bool shutting_down_ = false;
  std::vector<scoped_refptr<ChooserSession>> sessions_;
  base::WeakPtrFactory<FileChooserServiceImpl> weak_factory_{this};
};
}  // namespace

FileChooserServiceFactory* FileChooserServiceFactory::Get() {
  static base::NoDestructor<FileChooserServiceFactory> instance;
  return instance.get();
}
FileChooserServiceFactory::FileChooserServiceFactory()
    : BrowserContextKeyedServiceFactory(
          "TuffiteFileChooser",
          BrowserContextDependencyManager::GetInstance()) {}
FileChooserService* FileChooserServiceFactory::GetForBrowserContext(
    content::BrowserContext* context) {
  return static_cast<FileChooserService*>(
      Get()->GetServiceForBrowserContext(context, true));
}
std::unique_ptr<KeyedService>
FileChooserServiceFactory::BuildServiceInstanceForBrowserContext(
    content::BrowserContext*) const {
  return std::make_unique<FileChooserServiceImpl>();
}
}  // namespace tuffite
