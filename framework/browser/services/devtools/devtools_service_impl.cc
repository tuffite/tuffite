// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/devtools/devtools_service_impl.h"

#include <memory>
#include <string>
#include <utility>

#include "base/check.h"
#include "base/command_line.h"
#include "base/files/file_path.h"
#include "base/functional/bind.h"
#include "base/logging.h"
#include "base/no_destructor.h"
#include "base/strings/string_number_conversions.h"
#include "base/task/single_thread_task_runner.h"
#include "base/time/time.h"
#include "content/public/browser/browser_context.h"
#include "content/public/browser/devtools_agent_host.h"
#include "content/public/browser/devtools_socket_factory.h"
#include "content/public/browser/global_routing_id.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/web_contents.h"
#include "content/public/browser/web_contents_observer.h"
#include "content/public/browser/web_contents_user_data.h"
#include "content/public/common/content_switches.h"
#include "net/base/net_errors.h"
#include "net/log/net_log_source.h"
#include "net/socket/tcp_server_socket.h"
#include "tuffite/framework/app/services/manifest/manifest_service.h"
#include "tuffite/framework/browser/services/devtools/devtools_frontend.h"
#include "tuffite/framework/browser/services/devtools/devtools_service_factory.h"
#include "tuffite/framework/browser/services/window/window_service.h"

namespace tuffite {
namespace {
// A session admitted before a navigation commits must not keep inspecting a
// document whose new origin is denied by the manifest.
class DevToolsPolicyObserver final
    : public content::WebContentsObserver,
      public content::WebContentsUserData<DevToolsPolicyObserver> {
 public:
  ~DevToolsPolicyObserver() override = default;

 private:
  friend class content::WebContentsUserData<DevToolsPolicyObserver>;
  DevToolsPolicyObserver(content::WebContents* contents,
                         const ManifestService& manifest)
      : content::WebContentsObserver(contents),
        content::WebContentsUserData<DevToolsPolicyObserver>(*contents),
        manifest_(manifest.GetWeakPtr()) {}
  void DidFinishNavigation(content::NavigationHandle* navigation) override {
    if (!navigation->HasCommitted() || !navigation->IsInPrimaryMainFrame() ||
        !manifest_ ||
        manifest_->IsDevToolsAllowed(
            navigation->GetRenderFrameHost()->GetLastCommittedOrigin())) {
      return;
    }
    auto* contents = web_contents();
    if (content::DevToolsAgentHost::HasFor(contents)) {
      content::DevToolsAgentHost::GetOrCreateFor(contents)
          ->ForceDetachAllSessions();
    }
    if (auto tab = content::DevToolsAgentHost::GetForTab(contents)) {
      tab->ForceDetachAllSessions();
    }
  }
  base::WeakPtr<const ManifestService> manifest_;
  WEB_CONTENTS_USER_DATA_KEY_DECL();
};
WEB_CONTENTS_USER_DATA_KEY_IMPL(DevToolsPolicyObserver);

// Chromium provides one remote debugging server per process. This weak lease
// prevents one profile's shutdown from stopping another profile's
// server.
base::WeakPtr<DevToolsServiceImpl>& ServerOwner() {
  static base::NoDestructor<base::WeakPtr<DevToolsServiceImpl>> owner;
  return *owner;
}
class DevToolsManagerDelegate final : public content::DevToolsManagerDelegate {
 public:
  content::BrowserContext* GetDefaultBrowserContext() override;
  content::BrowserContext* GetBrowserContext(const std::string& id) override;
  std::string GetDiscoveryPageHTML() override;
  bool HasBundledFrontendResources() override;
  bool AllowInspectingRenderFrameHost(content::RenderFrameHost* frame) override;
  bool AllowInspectingTarget(content::DevToolsAgentHost* target) override;
};
content::BrowserContext* DevToolsManagerDelegate::GetDefaultBrowserContext() {
  return ServerOwner() ? ServerOwner()->browser_context() : nullptr;
}
content::BrowserContext* DevToolsManagerDelegate::GetBrowserContext(
    const std::string& id) {
  auto* context = GetDefaultBrowserContext();
  return context && context->UniqueId() == id ? context : nullptr;
}
class LoopbackSocketFactory final : public content::DevToolsSocketFactory {
 public:
  explicit LoopbackSocketFactory(uint16_t port) : port_(port) {}
  ~LoopbackSocketFactory() override = default;

  std::unique_ptr<net::ServerSocket> CreateForHttpServer() override {
    auto socket =
        std::make_unique<net::TCPServerSocket>(nullptr, net::NetLogSource());
    if (socket->ListenWithAddressAndPort("127.0.0.1", port_, 10) != net::OK) {
      return nullptr;
    }
    return socket;
  }

  std::unique_ptr<net::ServerSocket> CreateForTethering(
      std::string* out_name) override {
    return nullptr;
  }

 private:
  const uint16_t port_;
};

std::string DevToolsManagerDelegate::GetDiscoveryPageHTML() {
  return R"HTML(<!doctype html>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Tuffite DevTools</title>
<style>
  :root { color-scheme: light dark; font: 14px/1.5 system-ui, sans-serif; }
  body { margin: 0; padding: 32px; background: Canvas; color: CanvasText; }
  main { max-width: 760px; margin: 0 auto; }
  h1 { font-size: 22px; }
  article { margin: 12px 0; padding: 14px 16px; border: 1px solid GrayText; border-radius: 10px; }
  a { color: LinkText; font-weight: 600; }
  code { overflow-wrap: anywhere; color: GrayText; }
</style>
<main><h1>Tuffite DevTools</h1><p>Select a local target to inspect.</p><div id="targets">Loading...</div></main>
<script>
fetch('/json/list').then(response => response.json()).then(targets => {
  const container = document.querySelector('#targets');
  container.replaceChildren(...targets.map(target => {
    const card = document.createElement('article');
    const link = document.createElement('a');
    const frontend = String(target.devtoolsFrontendUrl || '');
    link.textContent = target.title || target.url || target.id;
    if (frontend.startsWith('/devtools/')) link.href = frontend;
    const detail = document.createElement('div');
    const code = document.createElement('code');
    code.textContent = target.url || '';
    detail.append(code);
    card.append(link, detail);
    return card;
  }));
}).catch(error => { document.querySelector('#targets').textContent = String(error); });
</script>)HTML";
}

bool DevToolsManagerDelegate::AllowInspectingRenderFrameHost(
    content::RenderFrameHost* frame) {
  if (!frame) {
    return false;
  }
  auto* contents = content::WebContents::FromRenderFrameHost(frame);
  if (!contents) {
    return false;
  }
  auto* service = DevToolsServiceFactory::GetForBrowserContext(
      contents->GetBrowserContext());
  return service && service->IsInspectionAllowed(contents, frame);
}
bool DevToolsManagerDelegate::AllowInspectingTarget(
    content::DevToolsAgentHost* target) {
  if (!target) {
    return false;
  }
  auto* contents = target->GetWebContents();
  if (!contents) {
    return true;
  }
  auto* service = DevToolsServiceFactory::GetForBrowserContext(
      contents->GetBrowserContext());
  return service && service->IsInspectionAllowed(contents);
}

bool DevToolsManagerDelegate::HasBundledFrontendResources() {
  return true;
}

}  // namespace
DevToolsServiceImpl::DevToolsServiceImpl(content::BrowserContext& context,
                                         window::WindowService& windows,
                                         const ManifestService& manifest)
    : browser_context_(&context), windows_(windows), manifest_(manifest) {}
DevToolsServiceImpl::~DevToolsServiceImpl() {
  Shutdown();
}
content::BrowserContext* DevToolsServiceImpl::browser_context() const {
  return browser_context_;
}
std::unique_ptr<content::DevToolsManagerDelegate>
CreateDevToolsManagerDelegate() {
  return std::make_unique<DevToolsManagerDelegate>();
}
bool DevToolsServiceImpl::Start(content::BrowserContext* context,
                                uint16_t port) {
  if (stopped_ || !context || context != browser_context_) {
    return false;
  }
  if (ServerOwner()) {
    return ServerOwner().get() == this && browser_context_ == context;
  }
  browser_context_ = context;
  ServerOwner() = weak_factory_.GetWeakPtr();
  content::DevToolsAgentHost::StartRemoteDebuggingServer(
      std::make_unique<LoopbackSocketFactory>(port), context->GetPath(),
      base::FilePath());
  return true;
}
base::WeakPtr<DevToolsService> DevToolsServiceImpl::GetWeakPtr() {
  return weak_factory_.GetWeakPtr();
}
void DevToolsServiceImpl::StartIfRequested(content::BrowserContext* context) {
  const auto& command_line = *base::CommandLine::ForCurrentProcess();
  int port = 0;
  if (command_line.HasSwitch(switches::kRemoteDebuggingPort) &&
      base::StringToInt(
          command_line.GetSwitchValueASCII(switches::kRemoteDebuggingPort),
          &port) &&
      port > 0 && port <= 65535 &&
      !Start(context, static_cast<uint16_t>(port))) {
    LOG(ERROR) << "The DevTools server is already owned by another profile";
  }
}
bool DevToolsServiceImpl::IsInspectionAllowed(
    content::WebContents* contents,
    content::RenderFrameHost* frame) const {
  if (stopped_ || !contents) {
    return false;
  }
  DevToolsPolicyObserver::CreateForWebContents(contents, manifest_.get());
  auto* main = contents->GetPrimaryMainFrame();
  return main && manifest_->IsDevToolsAllowed(main->GetLastCommittedOrigin()) &&
         (!frame ||
          manifest_->IsDevToolsAllowed(frame->GetLastCommittedOrigin()));
}

void DevToolsServiceImpl::InspectElement(content::WebContents* contents,
                                         content::RenderFrameHost* frame,
                                         int x,
                                         int y) {
  if (contents && frame && IsInspectionAllowed(contents, frame) &&
      Start(contents->GetBrowserContext(), 0)) {
    ShowFrontendWhenReady(contents->GetWeakPtr(), frame->GetGlobalId(), x, y,
                          200);
  }
}
void DevToolsServiceImpl::ShowFrontendWhenReady(
    base::WeakPtr<content::WebContents> inspected,
    content::GlobalRenderFrameHostId frame_id,
    int x,
    int y,
    int attempts) {
  if (!inspected || stopped_ || !IsInspectionAllowed(inspected.get())) {
    return;
  }
  if (content::DevToolsAgentHost::GetRemoteDebuggingServerAddress().empty()) {
    if (attempts <= 0) {
      LOG(ERROR) << "Timed out waiting for the Tuffite DevTools server";
      return;
    }
    base::SingleThreadTaskRunner::GetCurrentDefault()->PostDelayedTask(
        FROM_HERE,
        base::BindOnce(&DevToolsServiceImpl::ShowFrontendWhenReady,
                       weak_factory_.GetWeakPtr(), inspected, frame_id, x, y,
                       attempts - 1),
        base::Milliseconds(10));
    return;
  }
  auto* frame = content::RenderFrameHost::FromID(frame_id);
  if (!frame) {
    frame = inspected->GetFocusedFrame();
  }
  if (frame) {
    ShowDevToolsFrontend(*windows_, inspected.get(), frame, x, y);
  }
}
void DevToolsServiceImpl::Shutdown() {
  if (stopped_) {
    return;
  }
  stopped_ = true;
  if (ServerOwner().get() == this) {
    content::DevToolsAgentHost::StopRemoteDebuggingServer();
    ServerOwner().reset();
  }
  browser_context_ = nullptr;
  weak_factory_.InvalidateWeakPtrs();
}
}  // namespace tuffite
