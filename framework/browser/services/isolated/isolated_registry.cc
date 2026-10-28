// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/framework/browser/services/isolated/isolated_registry.h"

#include <algorithm>
#include <limits>

#include "base/base64.h"
#include "base/strings/string_number_conversions.h"
#include "base/strings/string_util.h"
#include "tuffite/framework/common/access.h"

namespace tuffite {
IsolatedRegistry::IsolatedRegistry() = default;
IsolatedRegistry::~IsolatedRegistry() = default;
void IsolatedRegistry::SetDevelopmentAlias(const url::Origin& packaged,
                                           const url::Origin& development) {
  packaged_origin_ = packaged;
  development_origin_ = development;
}
void IsolatedRegistry::AddBundled(const std::string& name,
                                  const Definition& definition) {
  entries_.emplace(name, Entry{std::make_shared<const Definition>(definition),
                               next_generation_++, true});
}
std::shared_ptr<const IsolatedRegistry::Definition> IsolatedRegistry::Find(
    std::string_view name,
    const url::Origin& origin) const {
  auto it = entries_.find(std::string(name));
  if (it == entries_.end() ||
      !std::ranges::any_of(
          it->second.definition->origins, [&](const auto& pattern) {
            return access::MatchesOrigin(origin, pattern) ||
                   (development_origin_ && origin == *development_origin_ &&
                    access::MatchesOrigin(*packaged_origin_, pattern));
          })) {
    return {};
  }
  return it->second.definition;
}
std::optional<std::string> IsolatedRegistry::Source(
    std::string_view name,
    const url::Origin& origin) const {
  auto definition = Find(name, origin);
  if (!definition) {
    return std::nullopt;
  }
  auto resource = definition->resources.find(definition->entry);
  return resource == definition->resources.end() ? definition->source
                                                 : resource->second;
}
std::optional<std::string> IsolatedRegistry::Apply(
    const base::DictValue& request) {
  const auto* op = request.FindString("op");
  const auto* name = request.FindString("name");
  if (!op || !name || !isolated::IsValidWorldName(*name)) {
    return std::nullopt;
  }
  auto found = entries_.find(*name);
  if (*op == "register") {
    if (found != entries_.end()) {
      return std::nullopt;
    }
  } else if (*op == "replace" || *op == "unregister") {
    const auto* generation = request.FindString("generation");
    if (found == entries_.end() || found->second.bundled || !generation ||
        *generation != base::NumberToString(found->second.generation)) {
      return std::nullopt;
    }
    if (*op == "unregister") {
      entries_.erase(found);
      return "0";
    }
  } else {
    return std::nullopt;
  }
  const auto* source = request.FindString("source");
  const auto* origins = request.FindList("origins");
  if (!source || (source->empty() && !request.FindDict("resources")) ||
      source->size() > 8 * 1024 * 1024 || !base::IsStringUTF8(*source) ||
      !origins || origins->empty() || origins->size() > 32 ||
      next_generation_ == std::numeric_limits<uint64_t>::max()) {
    return std::nullopt;
  }
  Definition definition;
  definition.source = *source;
  for (const auto& value : *origins) {
    std::string normalized;
    if (!value.is_string() || value.GetString().size() > 2048 ||
        !access::NormalizeOrigin(value.GetString(), &normalized) ||
        normalized == "*" || normalized.find('*') != std::string::npos) {
      return std::nullopt;
    }
    definition.origins.push_back(std::move(normalized));
  }
  if (const auto* entry = request.FindString("entry")) {
    const auto* resources = request.FindDict("resources");
    if (!isolated::IsValidResourcePath(*entry) || !resources ||
        resources->empty() || resources->size() > 1024) {
      return std::nullopt;
    }
    definition.entry = *entry;
    for (const auto [path, value] : *resources) {
      if (!isolated::IsValidResourcePath(path) || !value.is_string()) {
        return std::nullopt;
      }
      std::string contents;
      if (!base::Base64Decode(value.GetString(), &contents) ||
          contents.size() > 8 * 1024 * 1024) {
        return std::nullopt;
      }
      definition.resources.emplace(path, std::move(contents));
    }
    if (!definition.resources.contains(definition.entry)) {
      return std::nullopt;
    }
    definition.source.clear();
  }
  size_t bytes = definition.ByteSize();
  for (const auto& [key, entry] : entries_) {
    if (key != *name) {
      bytes += entry.definition->ByteSize();
    }
  }
  if (bytes > 32 * 1024 * 1024 ||
      (found == entries_.end() && entries_.size() >= 32)) {
    return std::nullopt;
  }
  const auto generation = next_generation_++;
  entries_.insert_or_assign(
      *name, Entry{std::make_shared<const Definition>(std::move(definition)),
                   generation, false});
  return base::NumberToString(generation);
}
}  // namespace tuffite
