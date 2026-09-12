// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/browser/services/window/devtools_frontend.h"

#include <algorithm>
#include <memory>
#include <set>
#include <string>
#include <string_view>
#include <utility>

#include "base/base64.h"
#include "base/containers/span.h"
#include "base/containers/unique_ptr_adapters.h"
#include "base/functional/bind.h"
#include "base/functional/callback_helpers.h"
#include "base/memory/weak_ptr.h"
#include "base/no_destructor.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/utf_string_conversions.h"
#include "base/values.h"
#include "content/public/browser/devtools_agent_host.h"
#include "content/public/browser/devtools_frontend_host.h"
#include "content/public/browser/global_routing_id.h"
#include "content/public/browser/navigation_handle.h"
#include "content/public/browser/render_frame_host.h"
#include "content/public/browser/storage_partition.h"
#include "content/public/browser/web_contents.h"
#include "ipc/constants.mojom.h"
#include "net/base/net_errors.h"
#include "net/http/http_response_headers.h"
#include "net/traffic_annotation/network_traffic_annotation.h"
#include "services/network/public/cpp/resource_request.h"
#include "services/network/public/cpp/simple_url_loader.h"
#include "services/network/public/cpp/simple_url_loader_stream_consumer.h"
#include "services/network/public/mojom/url_response_head.mojom.h"
#include "tuffite/framework/browser/services/window/platform.h"
#include "tuffite/framework/browser/services/window/window.h"

namespace tuffite {
namespace {

base::DictValue BuildResourceResponse(const net::HttpResponseHeaders* headers,
                                      bool success,
                                      int net_error) {
  base::DictValue response;
  response.Set("statusCode", headers   ? headers->response_code()
                             : success ? 200
                                       : 404);
  response.Set("netError", net_error);
  response.Set("netErrorName", net::ErrorToString(net_error));

  base::DictValue response_headers;
  size_t iterator = 0;
  std::string name;
  std::string value;
  while (headers && headers->EnumerateHeaderLines(&iterator, &name, &value)) {
    response_headers.Set(name, value);
  }
  response.Set("headers", std::move(response_headers));
  return response;
}

class DevToolsFrontend final : public content::WebContentsObserver,
                               public content::DevToolsAgentHostClient {
 public:
  DevToolsFrontend(Window* window, content::WebContents* inspected);

  DevToolsFrontend(const DevToolsFrontend&) = delete;
  DevToolsFrontend& operator=(const DevToolsFrontend&) = delete;

  ~DevToolsFrontend() override;

  bool MayAccessAllCookies() override;

  static DevToolsFrontend* FindFor(content::WebContents* inspected);

  void Focus() {
    if (window_) {
      if (window::GetState(window_).minimized) {
        window::Restore(window_);
      }
      window::Focus(window_);
    }
  }

 private:
  static std::set<DevToolsFrontend*>& Frontends() {
    static base::NoDestructor<std::set<DevToolsFrontend*>> frontends;
    return *frontends;
  }

  class NetworkResourceLoader final
      : public network::SimpleURLLoaderStreamConsumer {
   public:
    NetworkResourceLoader(int stream_id,
                          int request_id,
                          DevToolsFrontend* frontend,
                          std::unique_ptr<network::SimpleURLLoader> loader,
                          network::mojom::URLLoaderFactory* factory);

    NetworkResourceLoader(const NetworkResourceLoader&) = delete;
    NetworkResourceLoader& operator=(const NetworkResourceLoader&) = delete;
    ~NetworkResourceLoader() override;

   private:
    void OnResponseStarted(
        const GURL&,
        const network::mojom::URLResponseHead& response_head);

    void OnDataReceived(std::string_view chunk,
                        base::OnceClosure resume) override;

    void OnComplete(bool success) override;

    void OnRetry(base::OnceClosure) override;

    const int stream_id_;
    const int request_id_;
    const raw_ptr<DevToolsFrontend> frontend_;
    std::unique_ptr<network::SimpleURLLoader> loader_;
    scoped_refptr<net::HttpResponseHeaders> response_headers_;
  };

  void ReadyToCommitNavigation(
      content::NavigationHandle* navigation_handle) override;

  void PrimaryMainDocumentElementAvailable() override;

  void WebContentsDestroyed() override;

  void AgentHostClosed(content::DevToolsAgentHost*) override;

  void DispatchProtocolMessage(content::DevToolsAgentHost*,
                               base::span<const uint8_t> message) override;

  void HandleFrontendMessage(base::DictValue message) {
    const std::string* method = message.FindString("method");
    if (!method) {
      return;
    }
    const int request_id = message.FindInt("id").value_or(0);
    base::ListValue params;
    if (base::ListValue* message_params = message.FindList("params")) {
      params = std::move(*message_params);
    }

    if (*method == "dispatchProtocolMessage" && params.size() == 1) {
      const std::string* protocol_message = params[0].GetIfString();
      if (!agent_host_ || !protocol_message) {
        return;
      }
      agent_host_->DispatchProtocolMessage(
          this, base::as_byte_span(*protocol_message));
    } else if (*method == "bringToFront") {
      if (!inspected_) {
        return;
      }
      content::WebContents* outermost = inspected_->GetOutermostWebContents();
      Window* inspected_window = Window::FromWebContents(outermost);
      if (inspected_window) {
        window::Focus(inspected_window);
      } else {
        inspected_->Focus();
      }
    } else if (*method == "closeWindow") {
      if (window_) {
        window_->Close();
      }
      return;
    } else if (*method == "loadCompleted") {
      CallClient("setUseSoftMenu", base::Value(true));
    } else if (*method == "setIsDocked") {
      // Tuffite DevTools are always hosted in a separate top-level window.
      // Acknowledge the frontend request so its DockController can finish
      // initialization while keeping the frontend in undocked mode.
      SendAck(request_id, {});
      return;
    } else if (*method == "inspectElementCompleted") {
      if (window_) {
        window::Focus(window_);
      }
    } else if (*method == "setInspectedPageBounds" ||
               *method == "inspectedURLChanged") {
      // These messages are meaningful only for a docked frontend or browser
      // chrome. Tuffite currently hosts DevTools as an undocked window.
    } else if (*method == "loadNetworkResource" && params.size() == 3) {
      LoadNetworkResource(request_id, params);
      return;
    } else if (*method == "getPreferences") {
      SendAck(request_id, preferences_.Clone());
      return;
    } else if (*method == "getHostConfig") {
      SendAck(request_id, {});
      return;
    } else if (*method == "setPreference" && params.size() >= 2) {
      const std::string* name = params[0].GetIfString();
      if (!name || !params[1].is_string()) {
        return;
      }
      preferences_.Set(*name, std::move(params[1]));
    } else if (*method == "removePreference" && !params.empty()) {
      const std::string* name = params[0].GetIfString();
      if (!name) {
        return;
      }
      preferences_.Remove(*name);
    } else if (*method == "requestFileSystems") {
      CallClient("fileSystemsLoaded", base::Value(base::Value::Type::LIST));
    } else if (*method == "reattach") {
      if (!agent_host_) {
        return;
      }
      agent_host_->DetachClient(this);
      agent_host_->AttachClient(this);
    } else {
      return;
    }

    if (request_id) {
      SendAck(request_id, {});
    }
  }

  void LoadNetworkResource(int request_id, const base::ListValue& params) {
    const std::string* url = params[0].GetIfString();
    const std::string* headers = params[1].GetIfString();
    const std::optional<int> stream_id = params[2].GetIfInt();
    if (!url || !headers || !stream_id || !inspected_) {
      return;
    }
    const GURL resource_url(*url);
    if (!resource_url.is_valid()) {
      base::DictValue response;
      response.Set("statusCode", 404);
      response.Set("urlValid", false);
      SendAck(request_id, std::move(response));
      return;
    }

    auto request = std::make_unique<network::ResourceRequest>();
    request->url = resource_url;
    request->site_for_cookies = net::SiteForCookies::FromUrl(resource_url);
    request->headers.AddHeadersFromString(*headers);
    static constexpr net::NetworkTrafficAnnotationTag kTrafficAnnotation =
        net::DefineNetworkTrafficAnnotation("tuffite_devtools_resource", R"(
          semantics {
            sender: "Tuffite Developer Tools"
            description: "Loads resources requested by Developer Tools."
            trigger: "The user opens Developer Tools."
            data: "Resource URLs and request headers."
            destination: OTHER
          }
          policy {
            cookies_allowed: YES
            cookies_store: "user"
            setting: "This is only active while Developer Tools is open."
          })");
    auto loader = network::SimpleURLLoader::Create(std::move(request),
                                                   kTrafficAnnotation);
    auto factory = inspected_->GetPrimaryMainFrame()
                       ->GetStoragePartition()
                       ->GetURLLoaderFactoryForBrowserProcess();
    loaders_.insert(std::make_unique<NetworkResourceLoader>(
        *stream_id, request_id, this, std::move(loader), factory.get()));
  }

  void CallClient(const std::string& method,
                  base::Value arg1 = {},
                  base::Value arg2 = {},
                  base::Value arg3 = {}) {
    if (!web_contents() || !web_contents()->GetPrimaryMainFrame()) {
      return;
    }
    content::RenderFrameHost* frame = web_contents()->GetPrimaryMainFrame();
    frame->AllowInjectingJavaScript();
    base::ListValue arguments;
    if (!arg1.is_none()) {
      arguments.Append(std::move(arg1));
      if (!arg2.is_none()) {
        arguments.Append(std::move(arg2));
        if (!arg3.is_none()) {
          arguments.Append(std::move(arg3));
        }
      }
    }
    frame->ExecuteJavaScriptMethod(u"DevToolsAPI", base::ASCIIToUTF16(method),
                                   std::move(arguments), base::NullCallback());
  }

  void SendAck(int request_id, base::DictValue response) {
    // This name belongs to Chromium's DevToolsAPI: the native embedder replies
    // to a frontend request, resolving its callback by request id.
    CallClient("embedderMessageAck", base::Value(request_id),
               base::Value(std::move(response)));
  }

  raw_ptr<Window> window_;
  base::WeakPtr<content::WebContents> inspected_;
  scoped_refptr<content::DevToolsAgentHost> agent_host_;
  std::unique_ptr<content::DevToolsFrontendHost> frontend_host_;
  std::set<std::unique_ptr<NetworkResourceLoader>, base::UniquePtrComparator>
      loaders_;
  base::DictValue preferences_;
};

DevToolsFrontend::NetworkResourceLoader::NetworkResourceLoader(
    int stream_id,
    int request_id,
    DevToolsFrontend* frontend,
    std::unique_ptr<network::SimpleURLLoader> loader,
    network::mojom::URLLoaderFactory* factory)
    : stream_id_(stream_id),
      request_id_(request_id),
      frontend_(frontend),
      loader_(std::move(loader)) {
  loader_->SetOnResponseStartedCallback(base::BindOnce(
      &NetworkResourceLoader::OnResponseStarted, base::Unretained(this)));
  loader_->DownloadAsStream(factory, this);
}

DevToolsFrontend::NetworkResourceLoader::~NetworkResourceLoader() = default;

void DevToolsFrontend::NetworkResourceLoader::OnResponseStarted(
    const GURL&,
    const network::mojom::URLResponseHead& response_head) {
  response_headers_ = response_head.headers;
}

void DevToolsFrontend::NetworkResourceLoader::OnDataReceived(
    std::string_view chunk,
    base::OnceClosure resume) {
  const bool encoded = !base::IsStringUTF8(chunk);
  frontend_->CallClient(
      "streamWrite", base::Value(stream_id_),
      base::Value(encoded ? base::Base64Encode(chunk) : std::string(chunk)),
      base::Value(encoded));
  std::move(resume).Run();
}

void DevToolsFrontend::NetworkResourceLoader::OnComplete(bool success) {
  frontend_->SendAck(request_id_,
                     BuildResourceResponse(response_headers_.get(), success,
                                           loader_->NetError()));
  frontend_->loaders_.erase(frontend_->loaders_.find(this));
}

void DevToolsFrontend::NetworkResourceLoader::OnRetry(base::OnceClosure) {
  NOTREACHED();
}

DevToolsFrontend::DevToolsFrontend(Window* window,
                                   content::WebContents* inspected)
    : content::WebContentsObserver(window->web_contents()),
      window_(window),
      inspected_(inspected->GetWeakPtr()) {
  // Register before navigation so repeated Inspect requests also reuse a
  // frontend whose document has not finished initializing yet.
  Frontends().insert(this);
}

DevToolsFrontend::~DevToolsFrontend() {
  Frontends().erase(this);
  if (agent_host_) {
    agent_host_->DetachClient(this);
  }
}

DevToolsFrontend* DevToolsFrontend::FindFor(content::WebContents* inspected) {
  for (DevToolsFrontend* frontend : Frontends()) {
    if (frontend->inspected_.get() == inspected && frontend->window_) {
      return frontend;
    }
  }
  return nullptr;
}

bool DevToolsFrontend::MayAccessAllCookies() {
  return true;
}

void DevToolsFrontend::ReadyToCommitNavigation(
    content::NavigationHandle* navigation_handle) {
  if (!navigation_handle->IsInPrimaryMainFrame()) {
    return;
  }
  frontend_host_ = content::DevToolsFrontendHost::Create(
      navigation_handle->GetRenderFrameHost(),
      base::BindRepeating(&DevToolsFrontend::HandleFrontendMessage,
                          base::Unretained(this)));
}

void DevToolsFrontend::PrimaryMainDocumentElementAvailable() {
  if (!inspected_) {
    return;
  }
  if (!agent_host_) {
    agent_host_ =
        content::DevToolsAgentHost::GetOrCreateForTab(inspected_.get());
    if (!agent_host_ || !agent_host_->AttachClient(this)) {
      agent_host_ = nullptr;
      if (window_) {
        window_->Close();
      }
      return;
    }
  }
}

void DevToolsFrontend::WebContentsDestroyed() {
  window_ = nullptr;
  delete this;
}

void DevToolsFrontend::AgentHostClosed(content::DevToolsAgentHost*) {
  agent_host_ = nullptr;
  inspected_ = nullptr;
  if (window_) {
    window_->Close();
  }
}

void DevToolsFrontend::DispatchProtocolMessage(
    content::DevToolsAgentHost*,
    base::span<const uint8_t> message) {
  constexpr size_t kMaxChunkSize = IPC::mojom::kChannelMaximumMessageSize / 4;
  const std::string_view text(reinterpret_cast<const char*>(message.data()),
                              message.size());
  if (text.size() < kMaxChunkSize) {
    CallClient("dispatchMessage", base::Value(std::string(text)));
    return;
  }
  for (size_t offset = 0; offset < text.size(); offset += kMaxChunkSize) {
    CallClient(
        "dispatchMessageChunk",
        base::Value(std::string(text.substr(offset, kMaxChunkSize))),
        base::Value(base::NumberToString(offset == 0 ? text.size() : 0)));
  }
}

}  // namespace

void ShowDevToolsFrontend(content::WebContents* inspected,
                          content::RenderFrameHost* inspected_frame,
                          int x,
                          int y) {
  if (!inspected || !inspected_frame) {
    return;
  }
  const std::string address =
      content::DevToolsAgentHost::GetRemoteDebuggingServerAddress();
  if (address.empty()) {
    return;
  }
  // Match Chrome's DevToolsWindow::InspectElement ordering. The renderer must
  // enter inspect mode before the frontend starts attaching and restoring its
  // own Overlay state; doing this from the frontend's document callback makes
  // the initial highlight race with frontend initialization.
  scoped_refptr<content::DevToolsAgentHost> agent_host =
      content::DevToolsAgentHost::GetOrCreateForTab(inspected);
  if (!agent_host) {
    return;
  }
  agent_host->InspectElement(inspected_frame, x, y);
  if (DevToolsFrontend* frontend = DevToolsFrontend::FindFor(inspected)) {
    frontend->Focus();
    return;
  }
  Window* window = Window::Create(inspected->GetBrowserContext(), GURL(),
                                  gfx::Size(1100, 760));
  new DevToolsFrontend(window, inspected);
  window->LoadURL(
      GURL("http://" + address + "/devtools/devtools_app.html?targetType=tab"));
}

}  // namespace tuffite
