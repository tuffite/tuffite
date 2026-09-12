// Copyright 2026 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#ifndef TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_HTML_EXPLORER_ELEMENT_H_
#define TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_HTML_EXPLORER_ELEMENT_H_

#include <cstdint>

#include "tuffite/buildflags.h"

#if BUILDFLAG(TUFFITE_IN_CHROMIUM)
#include "third_party/blink/renderer/bindings/core/v8/script_promise.h"
#include "third_party/blink/renderer/core/html/html_frame_owner_element.h"
#include "third_party/blink/renderer/platform/mojo/heap_mojo_receiver.h"
#include "third_party/blink/renderer/platform/mojo/heap_mojo_remote.h"
#include "tuffite/components/explorer/common/explorer.mojom-blink.h"

namespace blink {

class ExplorerDialog;
class ExplorerTransfer;
template <typename>
class ScriptPromiseResolver;
class ExplorerExecuteJavaScriptOptions;
class ExplorerReloadOptions;
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
  String dialogs() const;
  void setDialogs(const String& value);
  bool contextMenuEnabled() const;
  void setContextMenuEnabled(bool enabled);
  bool deferred() const;
  void setDeferred(bool value);
  ScriptPromise<ExplorerTransfer> detach(ScriptState*);
  ScriptPromise<IDLUndefined> attach(ScriptState*, const String& token);
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
  ScriptPromise<IDLAny> executeJavaScript(
      ScriptState*,
      const String& source,
      const ExplorerExecuteJavaScriptOptions*);
  ScriptPromise<IDLBoolean> postMessage(ScriptState*, ScriptValue data);
  ScriptPromise<IDLBoolean> RespondToDialog(ScriptState*,
                                            uint64_t request_id,
                                            bool accept,
                                            const String& prompt_text);

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

  bool PrepareHost();
  tuffite::mojom::blink::ExplorerCreateParamsPtr CreateParams() const;
  void EnsureAttached();
  void OnCreateComplete(bool accepted);
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
  void DidRequestOpen(uint64_t request_id,
                      const String& event_type,
                      const String& request_json) override;
  void DidRequestDialog(uint64_t request_id,
                        const String& type,
                        const String& message,
                        const String& default_value,
                        bool is_reload,
                        const String& from) override;
  void DidReceiveMessage(const String& json, const String& from) override;

  HeapMojoRemote<tuffite::mojom::blink::ExplorerHost> host_;
  HeapMojoReceiver<tuffite::mojom::blink::ExplorerClient, HTMLExplorerElement>
      client_receiver_;
  String create_initial_url_;
  bool create_pending_ = false;
  bool attached_ = false;
  bool context_menu_enabled_ = false;
  bool detached_empty_ = false;
  Member<ScriptPromiseResolver<ExplorerTransfer>> detach_resolver_;
  Member<ScriptPromiseResolver<IDLUndefined>> attach_resolver_;
};

}  // namespace blink

#endif  // BUILDFLAG(TUFFITE_IN_CHROMIUM)
#endif  // TUFFITE_COMPONENTS_EXPLORER_RENDERER_BLINK_HTML_EXPLORER_ELEMENT_H_
