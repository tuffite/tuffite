// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#ifndef TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_HTML_EXPLORER_ELEMENT_H_
#define TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_HTML_EXPLORER_ELEMENT_H_

#include <cstdint>

#include "third_party/blink/renderer/platform/heap/collection_support/heap_hash_map.h"
#include "tuffite/buildflags.h"

#if BUILDFLAG(TUFFITE_IN_CHROMIUM)
#include "third_party/blink/renderer/bindings/core/v8/script_promise.h"
#include "third_party/blink/renderer/core/html/html_frame_owner_element.h"
#include "third_party/blink/renderer/core/messaging/blink_transferable_message.h"
#include "third_party/blink/renderer/platform/mojo/heap_mojo_receiver.h"
#include "third_party/blink/renderer/platform/mojo/heap_mojo_remote.h"
#include "tuffite/components/explorer/common/explorer.mojom-blink.h"

namespace blink {
class WindowPostMessageOptions;
class ExceptionState;

class ExplorerDialog;
class ExplorerNewWindow;
class ExplorerHandle;
template <typename>
class ScriptPromiseResolver;
class ExplorerExecuteJavaScriptOptions;
class ExplorerReloadOptions;
class ExplorerFindOptions;
class ScriptState;
class ScriptValue;

class CORE_EXPORT HTMLExplorerElement final
    : public HTMLFrameOwnerElement,
      public tuffite::mojom::blink::ExplorerClient {
  DEFINE_WRAPPERTYPEINFO();

 public:
  static bool IsEnabled(const Document& document);

  explicit HTMLExplorerElement(Document& document);
  ~HTMLExplorerElement() override;

  void Trace(Visitor* visitor) const override;

  String src() const;
  void setSrc(const String& value);
  String partition() const;
  void setPartition(const String& value);
  String navigation() const;
  void setNavigation(const String& value);
  bool isolated() const;
  void setIsolated(bool);
  String dialogs() const;
  void setDialogs(const String& value);
  bool contextMenuEnabled() const;
  void setContextMenuEnabled(bool enabled);
  ScriptPromise<ExplorerHandle> detach(ScriptState*);
  ScriptPromise<IDLUndefined> attach(ScriptState*, ExplorerHandle*);
  ScriptPromise<IDLUndefined> AttachToken(ScriptState*, const String& token);
  EventListener* ondetached() {
    return GetAttributeEventListener(AtomicString("detached"));
  }
  void setOndetached(EventListener* listener) {
    SetAttributeEventListener(AtomicString("detached"), listener);
  }
  EventListener* onattached() {
    return GetAttributeEventListener(AtomicString("attached"));
  }
  void setOnattached(EventListener* listener) {
    SetAttributeEventListener(AtomicString("attached"), listener);
  }
  EventListener* onloadstop() {
    return GetAttributeEventListener(AtomicString("loadstop"));
  }
  void setOnloadstop(EventListener* listener) {
    SetAttributeEventListener(AtomicString("loadstop"), listener);
  }
  EventListener* onloadcommit() {
    return GetAttributeEventListener(AtomicString("loadcommit"));
  }
  void setOnloadcommit(EventListener* listener) {
    SetAttributeEventListener(AtomicString("loadcommit"), listener);
  }
  EventListener* onloaderror() {
    return GetAttributeEventListener(AtomicString("loaderror"));
  }
  void setOnloaderror(EventListener* listener) {
    SetAttributeEventListener(AtomicString("loaderror"), listener);
  }
  EventListener* ontitlechange() {
    return GetAttributeEventListener(AtomicString("titlechange"));
  }
  void setOntitlechange(EventListener* listener) {
    SetAttributeEventListener(AtomicString("titlechange"), listener);
  }
  EventListener* ondownload() {
    return GetAttributeEventListener(AtomicString("download"));
  }
  void setOndownload(EventListener* listener) {
    SetAttributeEventListener(AtomicString("download"), listener);
  }
  EventListener* onfindresult() {
    return GetAttributeEventListener(AtomicString("findresult"));
  }
  void setOnfindresult(EventListener* listener) {
    SetAttributeEventListener(AtomicString("findresult"), listener);
  }
  EventListener* onzoomchange() {
    return GetAttributeEventListener(AtomicString("zoomchange"));
  }
  void setOnzoomchange(EventListener* listener) {
    SetAttributeEventListener(AtomicString("zoomchange"), listener);
  }
  EventListener* onaudiostatechange() {
    return GetAttributeEventListener(AtomicString("audiostatechange"));
  }
  void setOnaudiostatechange(EventListener* listener) {
    SetAttributeEventListener(AtomicString("audiostatechange"), listener);
  }
  EventListener* onfaviconchange() {
    return GetAttributeEventListener(AtomicString("faviconchange"));
  }
  void setOnfaviconchange(EventListener* listener) {
    SetAttributeEventListener(AtomicString("faviconchange"), listener);
  }
  EventListener* onrenderprocessgone() {
    return GetAttributeEventListener(AtomicString("renderprocessgone"));
  }
  void setOnrenderprocessgone(EventListener* listener) {
    SetAttributeEventListener(AtomicString("renderprocessgone"), listener);
  }
  EventListener* onpictureinpicturechange() {
    return GetAttributeEventListener(AtomicString("pictureinpicturechange"));
  }
  void setOnpictureinpicturechange(EventListener* listener) {
    SetAttributeEventListener(AtomicString("pictureinpicturechange"), listener);
  }
  EventListener* onnavigation() {
    return GetAttributeEventListener(AtomicString("navigation"));
  }
  void setOnnavigation(EventListener* listener) {
    SetAttributeEventListener(AtomicString("navigation"), listener);
  }
  EventListener* onnewwindow() {
    return GetAttributeEventListener(AtomicString("newwindow"));
  }
  void setOnnewwindow(EventListener* listener) {
    SetAttributeEventListener(AtomicString("newwindow"), listener);
  }
  EventListener* ondialog() {
    return GetAttributeEventListener(AtomicString("dialog"));
  }
  void setOndialog(EventListener* listener) {
    SetAttributeEventListener(AtomicString("dialog"), listener);
  }
  void load(const String& value);
  ScriptPromise<IDLBoolean> cancelDownload(ScriptState*, uint32_t id);
  ScriptPromise<IDLBoolean> download(ScriptState*, const String& url);
  ScriptPromise<IDLAny> getDownloads(ScriptState*);
  ScriptPromise<IDLString> readDownload(ScriptState*, uint32_t id);
  ScriptPromise<IDLString> capturePreview(ScriptState*);
  ScriptPromise<IDLBoolean> back(ScriptState*);
  ScriptPromise<IDLBoolean> forward(ScriptState*);
  ScriptPromise<IDLBoolean> go(ScriptState*, int32_t offset);
  ScriptPromise<IDLBoolean> reload(ScriptState*, const ExplorerReloadOptions*);
  ScriptPromise<IDLBoolean> stop(ScriptState*);
  ScriptPromise<IDLAny> getNavigationState(ScriptState*);
  ScriptPromise<IDLLong> findInPage(ScriptState*,
                                    const String&,
                                    const ExplorerFindOptions*);
  ScriptPromise<IDLBoolean> stopFindInPage(ScriptState*, const String& action);
  ScriptPromise<IDLBoolean> setZoomFactor(ScriptState*, double factor);
  ScriptPromise<IDLDouble> getZoomFactor(ScriptState*);
  ScriptPromise<IDLBoolean> setAudioMuted(ScriptState*, bool muted);
  ScriptPromise<IDLAny> executeJavaScript(
      ScriptState*,
      const String& source,
      const ExplorerExecuteJavaScriptOptions*);
  void postMessage(ScriptState*,
                   ScriptValue,
                   const WindowPostMessageOptions*,
                   ExceptionState&);
  void postMessage(ScriptState*,
                   ScriptValue,
                   const String&,
                   HeapVector<ScriptObject>,
                   ExceptionState&);
  ScriptPromise<IDLBoolean> RespondToDialog(ScriptState*,
                                            uint64_t request_id,
                                            bool accept,
                                            const String& prompt_text);

  ScriptPromise<IDLBoolean> DiscardWindow(ScriptState*, const String& token);
  FrameOwnerElementType OwnerType() const final;

 private:
  InsertionNotificationRequest InsertedInto(ContainerNode&) override;
  void DidNotifySubtreeInsertionsToDocument() final;
  void RemovedFrom(ContainerNode&) override;
  void ParseAttribute(const AttributeModificationParams&) override;
  bool LayoutObjectIsNeeded(const DisplayStyle&) const override;
  LayoutObject* CreateLayoutObject(const ComputedStyle&) override;
  bool IsInteractiveContent() const override;
  network::ParsedPermissionsPolicy ConstructContainerPolicy() const override;
  scoped_refptr<const SecurityOrigin> MakeOriginForPermissionsPolicy()
      const override;

  bool PrepareHost();
  tuffite::mojom::blink::ExplorerCreateParamsPtr CreateParams() const;
  void EnsureAttached();
  void OnCreateComplete(bool accepted);
  void BeginAttach(const String& token);
  void FinishAttach(const String& error);
  void OnConnectionError();
  String FrozenAttribute(const char* name, const char* default_value) const;
  void DispatchJsonEvent(const AtomicString& type, const String& json);

  void DidDownload(const String& detail_json) override;
  void DidStartLoading(const String& state_json) override;
  void DidStopLoading(const String& state_json) override;
  void DidFailLoad(int32_t error_code,
                   const String& error_description,
                   const String& state_json) override;
  void DidCommitNavigation(const String& state_json) override;
  void DidChangePageState(const String& event_type,
                          const String& state_json) override;
  void DidFindResult(const String& detail_json) override;
  void DidRequestOpen(uint64_t request_id,
                      const String& event_type,
                      const String& request_json) override;
  void DidRequestDialog(uint64_t request_id,
                        const String& type,
                        const String& message,
                        const String& default_value,
                        bool is_reload,
                        const String& from) override;
  void DidFinishRequest(const String& kind,
                        const String& key,
                        bool canceled) override;

  HeapMojoRemote<tuffite::mojom::blink::ExplorerHost> host_;
  HeapMojoReceiver<tuffite::mojom::blink::ExplorerClient, HTMLExplorerElement>
      client_receiver_;
  HeapHashMap<String, Member<ExplorerDialog>> dialogs_pending_;
  HeapHashMap<String, Member<ExplorerNewWindow>> windows_pending_;
  String create_initial_url_;
  bool create_pending_ = false;
  bool attached_ = false;
  bool context_menu_enabled_ = false;
  bool detached_empty_ = false;
  Member<ScriptPromiseResolver<ExplorerHandle>> detach_resolver_;
  Member<ScriptPromiseResolver<IDLUndefined>> attach_resolver_;
  Member<ExplorerHandle> attaching_handle_;
  Member<ExplorerHandle> replaced_handle_;
  String attach_error_;
  bool restoring_attachment_ = false;
  bool clearing_src_ = false;
};

}  // namespace blink

#endif  // BUILDFLAG(TUFFITE_IN_CHROMIUM)
#endif  // TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_HTML_EXPLORER_ELEMENT_H_
