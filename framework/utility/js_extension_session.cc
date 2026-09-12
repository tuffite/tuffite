// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

#include "tuffite/framework/utility/js_extension_session.h"

#include <algorithm>
#include <atomic>
#include <map>
#include <memory>
#include <optional>
#include <set>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "base/json/json_reader.h"
#include "base/logging.h"
#include "base/memory/raw_ptr.h"
#include "base/memory/ref_counted.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_split.h"
#include "base/strings/string_util.h"
#include "base/synchronization/lock.h"
#include "base/task/single_thread_task_runner.h"
#include "base/task/task_traits.h"
#include "base/task/thread_pool.h"
#include "base/threading/sequence_bound.h"
#include "gin/array_buffer.h"
#include "gin/public/isolate_holder.h"
#include "tuffite/framework/public/abi.h"
#include "url/gurl.h"
#include "v8/include/v8-context.h"
#include "v8/include/v8-exception.h"
#include "v8/include/v8-function.h"
#include "v8/include/v8-isolate.h"
#include "v8/include/v8-json.h"
#include "v8/include/v8-message.h"
#include "v8/include/v8-object.h"
#include "v8/include/v8-primitive.h"
#include "v8/include/v8-promise.h"
#include "v8/include/v8-script.h"

namespace tuffite {
namespace {

constexpr size_t kMaxInitBytes = 1024 * 1024;
constexpr size_t kMaxModules = 64;
constexpr size_t kMaxModuleNameBytes = 512;
constexpr size_t kMaxModuleSourceBytes = 512 * 1024;
constexpr size_t kMaxModuleGraphBytes = 1024 * 1024;
constexpr size_t kMaxCommandBytes = 1024 * 1024;
constexpr size_t kMaxArguments = 16;
constexpr size_t kMaxResponseBytes = 1024 * 1024;
constexpr size_t kIsolateHeapBytes = 32 * 1024 * 1024;
constexpr int kSessionFrameworkDataIndex = 1;

bool IsLogicalModuleName(std::string_view name) {
  if (name.empty() || name.size() > kMaxModuleNameBytes ||
      name.front() == '/' || name.back() == '/' || name.contains('\\') ||
      name.contains('\0')) {
    return false;
  }
  for (std::string_view part : base::SplitStringPiece(
           name, "/", base::KEEP_WHITESPACE, base::SPLIT_WANT_ALL)) {
    if (part.empty() || part == "." || part == "..") {
      return false;
    }
  }
  return base::EndsWith(name, ".mjs", base::CompareCase::SENSITIVE);
}

bool IsModuleURL(std::string_view value) {
  const GURL url(value);
  if (value.size() > kMaxModuleNameBytes || !url.is_valid() ||
      url.spec() != value || url.has_username() || url.has_password()) {
    return false;
  }
  if (url.SchemeIs("https") || url.SchemeIsFile()) {
    return true;
  }
  return !url.SchemeIs("http") && !url.SchemeIs("data") &&
         !url.SchemeIs("javascript") && !url.SchemeIs("ftp") &&
         !url.SchemeIs("ws") && !url.SchemeIs("wss") && !url.has_port() &&
         base::EndsWith(url.host(), ".utility");
}

std::optional<std::string> ResolveModuleName(std::string_view referrer,
                                             std::string_view specifier) {
  if (IsModuleURL(referrer)) {
    // Relative/absolute URL references are supported, bare package names are
    // not.
    const GURL absolute(specifier);
    if (!(specifier.starts_with("./") || specifier.starts_with("../") ||
          specifier.starts_with("/") || absolute.is_valid()) ||
        specifier.contains('\\') || specifier.contains('\0')) {
      return std::nullopt;
    }
    const GURL resolved = GURL(referrer).Resolve(specifier);
    return IsModuleURL(resolved.spec()) ? std::make_optional(resolved.spec())
                                        : std::nullopt;
  }
  // Preserve the legacy schemaVersion 1 logical-path contract.
  if (!(specifier.starts_with("./") || specifier.starts_with("../")) ||
      specifier.contains('\\') || specifier.contains('\0')) {
    return std::nullopt;
  }
  std::vector<std::string_view> segments = base::SplitStringPiece(
      referrer, "/", base::KEEP_WHITESPACE, base::SPLIT_WANT_NONEMPTY);
  if (segments.empty()) {
    return std::nullopt;
  }
  segments.pop_back();
  for (std::string_view part : base::SplitStringPiece(
           specifier, "/", base::KEEP_WHITESPACE, base::SPLIT_WANT_ALL)) {
    if (part.empty() || part == ".") {
      continue;
    }
    if (part == "..") {
      if (segments.empty()) {
        return std::nullopt;
      }
      segments.pop_back();
    } else {
      segments.push_back(part);
    }
  }
  std::string resolved = base::JoinString(segments, "/");
  return IsLogicalModuleName(resolved) ? std::make_optional(std::move(resolved))
                                       : std::nullopt;
}

bool IsMethodName(std::string_view name) {
  return !name.empty() && name.size() <= 128 &&
         std::ranges::all_of(name, [](unsigned char value) {
           return (value >= 'a' && value <= 'z') ||
                  (value >= 'A' && value <= 'Z') ||
                  (value >= '0' && value <= '9') || value == '_';
         });
}

std::string V8String(v8::Isolate* isolate, v8::Local<v8::Value> value) {
  if (value.IsEmpty()) {
    return {};
  }
  v8::String::Utf8Value utf8(isolate, value);
  return *utf8 ? std::string(*utf8, utf8.length()) : std::string();
}

std::string ExceptionMessage(v8::Isolate* isolate, v8::TryCatch& caught) {
  std::string message = V8String(isolate, caught.Exception());
  if (message.empty()) {
    message = "JavaScript extension execution failed";
  }
  v8::Local<v8::Message> detail = caught.Message();
  if (!detail.IsEmpty()) {
    const std::string resource =
        V8String(isolate, detail->GetScriptOrigin().ResourceName());
    const int line =
        detail->GetLineNumber(isolate->GetCurrentContext()).FromMaybe(0);
    if (!resource.empty() && line > 0) {
      message.append(" at ").append(resource).append(":").append(
          base::NumberToString(line));
    }
  }
  return message;
}

void ThrowTypeError(v8::Isolate* isolate, std::string_view message) {
  isolate->ThrowException(v8::Exception::TypeError(
      v8::String::NewFromUtf8(isolate, message.data(),
                              v8::NewStringType::kNormal,
                              static_cast<int>(message.size()))
          .ToLocalChecked()));
}

bool DenyWasm(v8::Local<v8::Context>, v8::Local<v8::String>) {
  return false;
}

v8::MaybeLocal<v8::Promise> DenyDynamicImport(v8::Local<v8::Context> context,
                                              v8::Local<v8::Data>,
                                              v8::Local<v8::Value>,
                                              v8::Local<v8::String>,
                                              v8::Local<v8::FixedArray>) {
  v8::Isolate* isolate = v8::Isolate::GetCurrent();
  v8::Local<v8::Promise::Resolver> resolver;
  if (!v8::Promise::Resolver::New(context).ToLocal(&resolver)) {
    return {};
  }
  const v8::Local<v8::String> message = v8::String::NewFromUtf8Literal(
      isolate, "dynamic import is disabled for extensions");
  if (resolver->Reject(context, v8::Exception::TypeError(message))
          .IsNothing()) {
    return {};
  }
  return resolver->GetPromise();
}

}  // namespace

struct JavaScriptExtensionSession::ExecutionControl
    : public base::RefCountedThreadSafe<ExecutionControl> {
  void Begin(uint64_t command_id) {
    base::AutoLock guard(lock);
    active_command = command_id;
  }

  bool IsCancelled(uint64_t command_id) {
    base::AutoLock guard(lock);
    return closing || cancelled.contains(command_id);
  }

  void Finish(uint64_t command_id) {
    base::AutoLock guard(lock);
    cancelled.erase(command_id);
    if (active_command == command_id) {
      active_command = 0;
    }
  }

  void Cancel(uint64_t command_id) {
    base::AutoLock guard(lock);
    cancelled.insert(command_id);
    if (active_command == command_id && isolate) {
      isolate->TerminateExecution();
    }
  }

  void Close() {
    base::AutoLock guard(lock);
    closing = true;
    if (isolate) {
      isolate->TerminateExecution();
    }
  }

  base::Lock lock;
  raw_ptr<v8::Isolate> isolate GUARDED_BY(lock) = nullptr;
  uint64_t active_command GUARDED_BY(lock) = 0;
  std::set<uint64_t> cancelled GUARDED_BY(lock);
  bool closing GUARDED_BY(lock) = false;

 private:
  friend class base::RefCountedThreadSafe<ExecutionControl>;
  ~ExecutionControl() = default;
};

class JavaScriptExtensionSession::Realm {
 public:
  Realm(std::map<std::string, std::string> sources,
        std::string entry,
        scoped_refptr<ExecutionControl> control)
      : sources_(std::move(sources)),
        entry_(std::move(entry)),
        control_(std::move(control)) {}

  Realm(const Realm&) = delete;
  Realm& operator=(const Realm&) = delete;

  ~Realm() { Reset(); }

  int32_t Initialize() {
    std::unique_ptr<v8::Isolate::CreateParams> params =
        gin::IsolateHolder::getDefaultIsolateParams();
    params->constraints.ConfigureDefaultsFromHeapSize(0, kIsolateHeapBytes);
    isolate_holder_ = std::make_unique<gin::IsolateHolder>(
        base::SingleThreadTaskRunner::GetCurrentDefault(),
        gin::IsolateHolder::AccessMode::kSingleThread,
        gin::IsolateHolder::IsolateType::kUtility, std::move(params));
    v8::Isolate* isolate = isolate_holder_->isolate();
    {
      base::AutoLock guard(control_->lock);
      control_->isolate = isolate;
    }
    isolate->SetMicrotasksPolicy(v8::MicrotasksPolicy::kExplicit);
    isolate->SetAllowWasmCodeGenerationCallback(&DenyWasm);
    isolate->SetHostImportModuleDynamicallyCallback(&DenyDynamicImport);

    v8::Isolate::Scope isolate_scope(isolate);
    v8::HandleScope handle_scope(isolate);
    v8::Local<v8::Context> context = v8::Context::New(isolate);
    context->AllowCodeGenerationFromStrings(false);
    context->SetErrorMessageForCodeGenerationFromStrings(
        v8::String::NewFromUtf8Literal(
            isolate, "eval and Function are disabled for extensions"));
    context->SetAlignedPointerInEmbedderData(kSessionFrameworkDataIndex, this,
                                             v8::kEmbedderDataTypeTagDefault);
    v8::Context::Scope context_scope(context);
    v8::TryCatch caught(isolate);

    context_.Reset(isolate, context);
    for (const auto& [name, source_text] : sources_) {
      v8::Local<v8::String> source;
      v8::Local<v8::String> resource_name;
      if (!v8::String::NewFromUtf8(isolate, source_text.data(),
                                   v8::NewStringType::kNormal,
                                   static_cast<int>(source_text.size()))
               .ToLocal(&source) ||
          !v8::String::NewFromUtf8(isolate, name.data(),
                                   v8::NewStringType::kNormal,
                                   static_cast<int>(name.size()))
               .ToLocal(&resource_name)) {
        return TUFFITE_STATUS_RESOURCE_EXHAUSTED;
      }
      v8::ScriptOrigin origin(resource_name, 0, 0, false, -1, {}, false, false,
                              true);
      v8::ScriptCompiler::Source module_source(source, origin);
      v8::Local<v8::Module> module;
      if (!v8::ScriptCompiler::CompileModule(isolate, &module_source)
               .ToLocal(&module)) {
        return Fail(caught);
      }
      modules_.emplace(name, v8::Global<v8::Module>(isolate, module));
    }

    return sources_.empty() ? TUFFITE_STATUS_OK : FinishLoading();
  }

  int32_t FinishLoading() {
    v8::Isolate* isolate = isolate_holder_->isolate();
    v8::Isolate::Scope isolate_scope(isolate);
    v8::HandleScope handle_scope(isolate);
    auto context = context_.Get(isolate);
    v8::Context::Scope context_scope(context);
    v8::TryCatch caught(isolate);
    if (!modules_.contains(entry_)) {
      return TUFFITE_STATUS_INVALID_ARGUMENT;
    }
    v8::Local<v8::Module> entry_module = modules_.at(entry_).Get(isolate);
    if (!entry_module->InstantiateModule(context, &Realm::ResolveModule)
             .FromMaybe(false)) {
      return Fail(caught);
    }
    v8::Local<v8::Promise> evaluation;
    if (!entry_module->Evaluate(context).ToLocal(&evaluation) ||
        evaluation->State() != v8::Promise::kFulfilled) {
      last_error_ = evaluation.IsEmpty()
                        ? ExceptionMessage(isolate, caught)
                        : V8String(isolate, evaluation->Result());
      LOG(ERROR) << "Tuffite JS extension evaluation failed: " << last_error_;
      return TUFFITE_STATUS_INVALID_ARGUMENT;
    }
    v8::Local<v8::Object> module_namespace =
        entry_module->GetModuleNamespace().As<v8::Object>();
    v8::Local<v8::Value> default_export;
    if (!module_namespace
             ->Get(context, v8::String::NewFromUtf8Literal(isolate, "default"))
             .ToLocal(&default_export) ||
        !default_export->IsObject()) {
      last_error_ = "extension entry must have an object default export";
      LOG(ERROR) << "Tuffite JS extension validation failed: " << last_error_;
      return TUFFITE_STATUS_INVALID_ARGUMENT;
    }
    extension_.Reset(isolate, default_export.As<v8::Object>());
    return TUFFITE_STATUS_OK;
  }

  std::pair<int32_t, std::vector<std::string>> LoadModule(std::string name,
                                                          std::string text) {
    if (!isolate_holder_ || modules_.size() >= kMaxModules) {
      return {TUFFITE_STATUS_RESOURCE_EXHAUSTED, {}};
    }
    v8::Isolate* isolate = isolate_holder_->isolate();
    v8::Isolate::Scope isolate_scope(isolate);
    v8::HandleScope handle_scope(isolate);
    auto context = context_.Get(isolate);
    v8::Context::Scope context_scope(context);
    v8::TryCatch caught(isolate);
    v8::Local<v8::String> source, resource;
    if (!v8::String::NewFromUtf8(isolate, text.data(),
                                 v8::NewStringType::kNormal,
                                 static_cast<int>(text.size()))
             .ToLocal(&source) ||
        !v8::String::NewFromUtf8(isolate, name.data(),
                                 v8::NewStringType::kNormal,
                                 static_cast<int>(name.size()))
             .ToLocal(&resource)) {
      return {TUFFITE_STATUS_RESOURCE_EXHAUSTED, {}};
    }
    v8::ScriptOrigin origin(resource, 0, 0, false, -1, {}, false, false, true);
    v8::ScriptCompiler::Source input(source, origin);
    v8::Local<v8::Module> module;
    if (!v8::ScriptCompiler::CompileModule(isolate, &input).ToLocal(&module)) {
      return {Fail(caught), {}};
    }
    // Cache before traversing imports so cyclic graphs never request a URL
    // twice.
    modules_.emplace(name, v8::Global<v8::Module>(isolate, module));
    std::vector<std::string> dependencies;
    auto requests = module->GetModuleRequests();
    for (int i = 0; i < requests->Length(); ++i) {
      auto request = requests->Get(i).As<v8::ModuleRequest>();
      auto resolved =
          ResolveModuleName(name, V8String(isolate, request->GetSpecifier()));
      if (!resolved || request->GetImportAttributes()->Length() != 0) {
        return {TUFFITE_STATUS_INVALID_ARGUMENT, {}};
      }
      dependencies.push_back(std::move(*resolved));
    }
    return {TUFFITE_STATUS_OK, std::move(dependencies)};
  }

  std::pair<int32_t, std::vector<uint8_t>> Execute(
      uint64_t command_id,
      std::vector<uint8_t> command) {
    if (!isolate_holder_ || control_->IsCancelled(command_id)) {
      control_->Finish(command_id);
      return {TUFFITE_STATUS_INVALID_STATE, {}};
    }
    control_->Begin(command_id);
    if (control_->IsCancelled(command_id)) {
      control_->Finish(command_id);
      return {TUFFITE_STATUS_INVALID_STATE, {}};
    }

    v8::Isolate* isolate = isolate_holder_->isolate();
    if (isolate->IsExecutionTerminating()) {
      isolate->CancelTerminateExecution();
    }
    v8::Isolate::Scope isolate_scope(isolate);
    v8::HandleScope handle_scope(isolate);
    v8::Local<v8::Context> context = context_.Get(isolate);
    v8::Context::Scope context_scope(context);
    v8::TryCatch caught(isolate);

    auto finish = [this, command_id](int32_t status,
                                     std::vector<uint8_t> response = {}) {
      if (isolate_holder_->isolate()->IsExecutionTerminating()) {
        isolate_holder_->isolate()->CancelTerminateExecution();
      }
      control_->Finish(command_id);
      return std::pair(status, std::move(response));
    };

    v8::Local<v8::String> command_json;
    if (!v8::String::NewFromUtf8(
             isolate, reinterpret_cast<const char*>(command.data()),
             v8::NewStringType::kNormal, static_cast<int>(command.size()))
             .ToLocal(&command_json)) {
      return finish(TUFFITE_STATUS_RESOURCE_EXHAUSTED);
    }
    v8::Local<v8::Value> parsed;
    if (!v8::JSON::Parse(context, command_json).ToLocal(&parsed) ||
        !parsed->IsObject()) {
      return finish(Fail(caught));
    }
    v8::Local<v8::Object> envelope = parsed.As<v8::Object>();
    v8::Local<v8::Value> method_value;
    v8::Local<v8::Value> args_value;
    if (!envelope
             ->Get(context, v8::String::NewFromUtf8Literal(isolate, "method"))
             .ToLocal(&method_value) ||
        !method_value->IsString() ||
        !envelope->Get(context, v8::String::NewFromUtf8Literal(isolate, "args"))
             .ToLocal(&args_value) ||
        !args_value->IsArray() ||
        !IsMethodName(V8String(isolate, method_value))) {
      return finish(TUFFITE_STATUS_INVALID_ARGUMENT);
    }
    v8::Local<v8::Object> extension = extension_.Get(isolate);
    v8::Local<v8::Value> callable;
    if (!extension->Get(context, method_value).ToLocal(&callable) ||
        !callable->IsFunction()) {
      return finish(TUFFITE_STATUS_INVALID_ARGUMENT);
    }
    v8::Local<v8::Array> args = args_value.As<v8::Array>();
    if (args->Length() > kMaxArguments) {
      return finish(TUFFITE_STATUS_INVALID_ARGUMENT);
    }
    std::vector<v8::Local<v8::Value>> argv;
    argv.reserve(args->Length());
    for (uint32_t index = 0; index < args->Length(); ++index) {
      v8::Local<v8::Value> argument;
      if (!args->Get(context, index).ToLocal(&argument)) {
        return finish(TUFFITE_STATUS_INVALID_ARGUMENT);
      }
      argv.push_back(argument);
    }
    v8::Local<v8::Value> result;
    if (!callable.As<v8::Function>()
             ->Call(context, extension, static_cast<int>(argv.size()),
                    argv.data())
             .ToLocal(&result)) {
      const int32_t status = isolate->IsExecutionTerminating()
                                 ? TUFFITE_STATUS_INVALID_STATE
                                 : Fail(caught);
      return finish(status);
    }
    if (result->IsPromise() || result->IsUndefined()) {
      last_error_ = result->IsPromise()
                        ? "extension methods must be synchronous in API v1"
                        : "extension method returned undefined";
      LOG(ERROR) << "Tuffite JS extension contract failed: " << last_error_;
      return finish(TUFFITE_STATUS_INVALID_ARGUMENT);
    }
    v8::Local<v8::String> serialized;
    if (!v8::JSON::Stringify(context, result).ToLocal(&serialized)) {
      return finish(Fail(caught));
    }
    v8::String::Utf8Value utf8(isolate, serialized);
    if (!*utf8 || utf8.length() < 0 ||
        static_cast<size_t>(utf8.length()) > kMaxResponseBytes) {
      return finish(TUFFITE_STATUS_RESOURCE_EXHAUSTED);
    }
    const std::string encoded(*utf8, utf8.length());
    std::vector<uint8_t> response(encoded.begin(), encoded.end());
    ++command_count_;
    return finish(TUFFITE_STATUS_OK, std::move(response));
  }

  std::pair<int32_t, std::vector<uint8_t>> Close(uint32_t reason) {
    const std::string summary =
        "{\"commands\":" + base::NumberToString(command_count_) +
        ",\"reason\":" + base::NumberToString(reason) + "}";
    Reset();
    return {TUFFITE_STATUS_OK,
            std::vector<uint8_t>(summary.begin(), summary.end())};
  }

 private:
  static v8::MaybeLocal<v8::Module> ResolveModule(
      v8::Local<v8::Context> context,
      v8::Local<v8::String> specifier,
      v8::Local<v8::FixedArray> import_attributes,
      v8::Local<v8::Module> referrer) {
    v8::Isolate* isolate = v8::Isolate::GetCurrent();
    auto* realm =
        static_cast<Realm*>(context->GetAlignedPointerFromEmbedderData(
            kSessionFrameworkDataIndex, v8::kEmbedderDataTypeTagDefault));
    if (!realm || import_attributes->Length() != 0) {
      ThrowTypeError(isolate, "extension import attributes are not supported");
      return {};
    }
    const std::optional<std::string> resolved =
        ResolveModuleName(V8String(isolate, referrer->GetResourceName()),
                          V8String(isolate, specifier));
    if (!resolved) {
      ThrowTypeError(isolate,
                     "extension imports must be relative and confined");
      return {};
    }
    const auto found = realm->modules_.find(*resolved);
    if (found == realm->modules_.end()) {
      ThrowTypeError(isolate,
                     "extension import is absent from the module graph");
      return {};
    }
    return found->second.Get(isolate);
  }

  int32_t Fail(v8::TryCatch& caught) {
    last_error_ = ExceptionMessage(isolate_holder_->isolate(), caught);
    LOG(ERROR) << "Tuffite JS extension failed: " << last_error_;
    return TUFFITE_STATUS_INVALID_ARGUMENT;
  }

  void Reset() {
    if (!isolate_holder_) {
      return;
    }
    v8::Isolate* isolate = isolate_holder_->isolate();
    if (isolate->IsExecutionTerminating()) {
      isolate->CancelTerminateExecution();
    }
    extension_.Reset();
    context_.Reset();
    modules_.clear();
    {
      base::AutoLock guard(control_->lock);
      control_->isolate = nullptr;
    }
    isolate_holder_.reset();
  }

  std::map<std::string, std::string> sources_;
  const std::string entry_;
  scoped_refptr<ExecutionControl> control_;
  std::unique_ptr<gin::IsolateHolder> isolate_holder_;
  v8::Global<v8::Context> context_;
  std::map<std::string, v8::Global<v8::Module>> modules_;
  v8::Global<v8::Object> extension_;
  std::string last_error_;
  uint64_t command_count_ = 0;
};

class JavaScriptExtensionSession::Impl {
 public:
  Impl(std::map<std::string, std::string> sources, std::string entry)
      : control(base::MakeRefCounted<ExecutionControl>()),
        realm(base::ThreadPool::CreateSingleThreadTaskRunner(
                  {base::TaskPriority::USER_VISIBLE,
                   base::TaskShutdownBehavior::BLOCK_SHUTDOWN}),
              std::move(sources),
              std::move(entry),
              control) {}

  scoped_refptr<ExecutionControl> control;
  base::SequenceBound<Realm> realm;
};

JavaScriptExtensionSession::JavaScriptExtensionSession(
    std::map<std::string, std::string> sources,
    std::string entry,
    mojo::PendingRemote<mojom::UtilitySessionClient> client,
    bool resolved)
    : impl_(std::make_unique<Impl>(std::move(sources), entry)),
      entry_(std::move(entry)),
      resolved_(resolved),
      client_(std::move(client)) {}

JavaScriptExtensionSession::~JavaScriptExtensionSession() {
  impl_->control->Close();
  impl_->realm.Reset();
}

JavaScriptExtensionSession::CreateResult JavaScriptExtensionSession::Create(
    base::span<const uint8_t> init,
    mojo::PendingRemote<mojom::UtilitySessionClient> client) {
  if (init.empty() || init.size() > kMaxInitBytes) {
    return {TUFFITE_STATUS_INVALID_ARGUMENT, nullptr};
  }
  const std::string serialized(reinterpret_cast<const char*>(init.data()),
                               init.size());
  std::optional<base::DictValue> envelope =
      base::JSONReader::ReadDict(serialized, base::JSON_PARSE_RFC);
  const int version =
      envelope ? envelope->FindInt("schemaVersion").value_or(0) : 0;
  if (version != 1 && version != 2) {
    return {TUFFITE_STATUS_INVALID_ARGUMENT, nullptr};
  }
  const std::string* entry = envelope->FindString("entry");
  const base::ListValue* modules = envelope->FindList("modules");
  if (!entry ||
      (version == 2 ? !IsModuleURL(*entry) : !IsLogicalModuleName(*entry)) ||
      (version == 2 && (!client.is_valid() || modules)) ||
      (version == 1 &&
       (!modules || modules->empty() || modules->size() > kMaxModules))) {
    return {TUFFITE_STATUS_INVALID_ARGUMENT, nullptr};
  }
  std::map<std::string, std::string> sources;
  size_t total_source_bytes = 0;
  if (modules) {
    for (const base::Value& module_value : *modules) {
      const base::DictValue* module = module_value.GetIfDict();
      const std::string* name = module ? module->FindString("name") : nullptr;
      const std::string* source =
          module ? module->FindString("source") : nullptr;
      if (!name || !source || !IsLogicalModuleName(*name) || source->empty() ||
          source->size() > kMaxModuleSourceBytes || sources.contains(*name) ||
          total_source_bytes > kMaxModuleGraphBytes - source->size()) {
        return {TUFFITE_STATUS_INVALID_ARGUMENT, nullptr};
      }
      total_source_bytes += source->size();
      sources.emplace(*name, *source);
    }
  }
  if (version == 1 && !sources.contains(*entry)) {
    return {TUFFITE_STATUS_INVALID_ARGUMENT, nullptr};
  }
  if (!gin::IsolateHolder::Initialized()) {
    gin::IsolateHolder::Initialize(gin::IsolateHolder::kStrictMode,
                                   gin::ArrayBufferAllocator::SharedInstance(),
                                   nullptr, "--jitless", true);
  }
  return {TUFFITE_STATUS_OK,
          std::unique_ptr<JavaScriptExtensionSession>(
              new JavaScriptExtensionSession(std::move(sources), *entry,
                                             std::move(client), version == 2))};
}

void JavaScriptExtensionSession::Start(
    base::OnceCallback<void(int32_t)> callback) {
  if (!resolved_) {
    impl_->realm.AsyncCall(&Realm::Initialize)
        .Then(base::BindOnce(&JavaScriptExtensionSession::OnStarted,
                             weak_factory_.GetWeakPtr(), std::move(callback)));
    return;
  }
  start_callback_ = std::move(callback);
  client_.set_disconnect_handler(
      base::BindOnce(&JavaScriptExtensionSession::CompleteStart,
                     weak_factory_.GetWeakPtr(), TUFFITE_STATUS_INVALID_STATE));
  start_timeout_.Start(
      FROM_HERE, base::Seconds(30),
      base::BindOnce(&JavaScriptExtensionSession::CompleteStart,
                     weak_factory_.GetWeakPtr(), TUFFITE_STATUS_INVALID_STATE));
  pending_modules_.push_back(entry_);
  requested_modules_.insert(entry_);
  impl_->realm.AsyncCall(&Realm::Initialize)
      .Then(base::BindOnce(&JavaScriptExtensionSession::OnInitialized,
                           weak_factory_.GetWeakPtr()));
}

void JavaScriptExtensionSession::OnInitialized(int32_t status) {
  if (!start_callback_ || closed_) {
    return;
  }
  if (status != TUFFITE_STATUS_OK) {
    CompleteStart(status);
    return;
  }
  RequestNextModule();
}

void JavaScriptExtensionSession::RequestNextModule() {
  if (!start_callback_ || closed_) {
    return;
  }
  if (pending_modules_.empty()) {
    impl_->realm.AsyncCall(&Realm::FinishLoading)
        .Then(base::BindOnce(&JavaScriptExtensionSession::CompleteStart,
                             weak_factory_.GetWeakPtr()));
    return;
  }
  std::string url = std::move(pending_modules_.front());
  pending_modules_.pop_front();
  client_->ResolveModule(
      url, base::BindOnce(&JavaScriptExtensionSession::OnModuleSource,
                          weak_factory_.GetWeakPtr(), url));
}

void JavaScriptExtensionSession::OnModuleSource(
    std::string url,
    int32_t status,
    const std::vector<uint8_t>& source) {
  if (!start_callback_ || closed_) {
    return;
  }
  if (status != TUFFITE_STATUS_OK) {
    CompleteStart(status);
    return;
  }
  std::string text(source.begin(), source.end());
  if (source.empty() || source.size() > kMaxModuleSourceBytes ||
      source.size() > kMaxModuleGraphBytes - source_bytes_ ||
      !base::IsStringUTF8(text)) {
    CompleteStart(TUFFITE_STATUS_INVALID_ARGUMENT);
    return;
  }
  source_bytes_ += source.size();
  impl_->realm.AsyncCall(&Realm::LoadModule)
      .WithArgs(std::move(url), std::move(text))
      .Then(base::BindOnce(&JavaScriptExtensionSession::OnModuleCompiled,
                           weak_factory_.GetWeakPtr()));
}

void JavaScriptExtensionSession::OnModuleCompiled(
    std::pair<int32_t, std::vector<std::string>> result) {
  if (!start_callback_ || closed_) {
    return;
  }
  if (result.first != TUFFITE_STATUS_OK) {
    CompleteStart(result.first);
    return;
  }
  for (auto& url : result.second) {
    if (!requested_modules_.insert(url).second) {
      continue;
    }
    if (requested_modules_.size() > kMaxModules) {
      CompleteStart(TUFFITE_STATUS_RESOURCE_EXHAUSTED);
      return;
    }
    pending_modules_.push_back(std::move(url));
  }
  RequestNextModule();
}

void JavaScriptExtensionSession::CompleteStart(int32_t status) {
  if (!start_callback_) {
    return;
  }
  start_timeout_.Stop();
  started_ = status == TUFFITE_STATUS_OK;
  if (!started_) {
    impl_->control->Close();
  }
  std::move(start_callback_).Run(status);
}

void JavaScriptExtensionSession::Send(uint64_t command_id,
                                      const std::vector<uint8_t>& command,
                                      SendCallback callback) {
  if (!started_ || closed_ || command_id == 0 || command.empty() ||
      command.size() > kMaxCommandBytes) {
    std::move(callback).Run(TUFFITE_STATUS_INVALID_ARGUMENT, {});
    return;
  }
  impl_->realm.AsyncCall(&Realm::Execute)
      .WithArgs(command_id, command)
      .Then(base::BindOnce(&JavaScriptExtensionSession::OnExecuted,
                           weak_factory_.GetWeakPtr(), std::move(callback)));
}

void JavaScriptExtensionSession::Acknowledge(uint64_t, uint32_t) {}

void JavaScriptExtensionSession::Cancel(uint64_t command_id) {
  if (started_ && !closed_ && command_id != 0) {
    impl_->control->Cancel(command_id);
  }
}

void JavaScriptExtensionSession::Close(uint32_t reason,
                                       CloseCallback callback) {
  if (closed_) {
    std::move(callback).Run(TUFFITE_STATUS_INVALID_STATE, {});
    return;
  }
  closed_ = true;
  CompleteStart(TUFFITE_STATUS_INVALID_STATE);
  impl_->control->Close();
  impl_->realm.AsyncCall(&Realm::Close)
      .WithArgs(reason)
      .Then(base::BindOnce(&JavaScriptExtensionSession::OnClosed,
                           weak_factory_.GetWeakPtr(), std::move(callback)));
}

void JavaScriptExtensionSession::OnStarted(
    base::OnceCallback<void(int32_t)> callback,
    int32_t status) {
  started_ = status == TUFFITE_STATUS_OK;
  std::move(callback).Run(status);
}

void JavaScriptExtensionSession::OnExecuted(
    SendCallback callback,
    std::pair<int32_t, std::vector<uint8_t>> result) {
  std::move(callback).Run(result.first, std::move(result.second));
}

void JavaScriptExtensionSession::OnClosed(
    CloseCallback callback,
    std::pair<int32_t, std::vector<uint8_t>> result) {
  impl_->realm.Reset();
  std::move(callback).Run(result.first, std::move(result.second));
}

}  // namespace tuffite
