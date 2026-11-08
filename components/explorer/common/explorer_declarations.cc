// Copyright 2026 The Tuffite Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
#include "tuffite/components/explorer/common/explorer_declarations.h"

#include <utility>

namespace tuffite {
namespace {
bool IsSpace(char c) {
  return c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == '\f';
}
bool IsDigit(char c) {
  return c >= '0' && c <= '9';
}
bool IsNameStart(char c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' ||
         c == '-';
}
bool IsName(char c) {
  return IsNameStart(c) || IsDigit(c);
}

class Parser {
 public:
  explicit Parser(std::string_view text) : text_(text) {}
  const ExplorerDeclarationError& error() const { return error_; }
  std::optional<std::vector<ExplorerDeclaration>> Parse() {
    if (text_.size() > 4096) {
      Fail("Declarations exceed 4096 bytes");
      return std::nullopt;
    }
    std::vector<ExplorerDeclaration> result;
    while (Trivia() && position_ < text_.size()) {
      if (text_[position_] == ';') {
        ++position_;
        continue;
      }
      const auto offset = position_;
      auto name = Identifier();
      if (!name || !Trivia() || !Consume(':')) {
        Fail("Expected a property followed by ':'");
        return std::nullopt;
      }
      auto values = Values(';', 0);
      if (!values || values->empty()) {
        Fail("Expected a property value");
        return std::nullopt;
      }
      result.push_back({std::move(*name), offset, std::move(*values)});
      Consume(';');
    }
    return failed_ ? std::nullopt : std::optional(std::move(result));
  }

 private:
  void Fail(std::string_view message) {
    if (!failed_) {
      error_ = {position_, std::string(message)};
    }
    failed_ = true;
  }
  bool Consume(char c) {
    if (position_ < text_.size() && text_[position_] == c) {
      ++position_;
      return true;
    }
    return false;
  }
  bool Trivia() {
    while (position_ < text_.size()) {
      if (IsSpace(text_[position_])) {
        ++position_;
      } else if (text_.substr(position_).starts_with("/*")) {
        const auto end = text_.find("*/", position_ + 2);
        if (end == text_.npos) {
          Fail("Unterminated comment");
          return false;
        }
        position_ = end + 2;
      } else {
        break;
      }
    }
    return !failed_;
  }
  std::optional<std::string> Identifier() {
    if (position_ == text_.size() || !IsNameStart(text_[position_])) {
      return std::nullopt;
    }
    const auto start = position_++;
    while (position_ < text_.size() && IsName(text_[position_])) {
      ++position_;
    }
    return std::string(text_.substr(start, position_ - start));
  }
  std::optional<std::vector<ExplorerValue>> Values(char end, size_t depth) {
    if (depth > 8) {
      Fail("Functions are nested too deeply");
      return std::nullopt;
    }
    std::vector<ExplorerValue> result;
    bool after_comma = false;
    while (Trivia() && position_ < text_.size() && text_[position_] != end) {
      if (++value_count_ > 256) {
        Fail("Too many component values");
        return std::nullopt;
      }
      const auto start = position_;
      const char c = text_[position_];
      if (c == ',') {
        if (result.empty() || after_comma) {
          Fail("Expected a value before ','");
          return std::nullopt;
        }
        ++position_;
        result.push_back({ExplorerValue::Kind::kComma, ",", start, {}});
        after_comma = true;
        continue;
      }
      after_comma = false;
      if (c == '\'' || c == '"') {
        ++position_;
        std::string value;
        while (position_ < text_.size() && text_[position_] != c) {
          if (text_[position_] == '\\') {
            ++position_;
            if (position_ == text_.size()) {
              break;
            }
          }
          value.push_back(text_[position_++]);
        }
        if (!Consume(c)) {
          Fail("Unterminated string");
          return std::nullopt;
        }
        result.push_back(
            {ExplorerValue::Kind::kString, std::move(value), start, {}});
        continue;
      }
      size_t number = position_;
      if (c == '+' || c == '-') {
        ++number;
      }
      const bool numeric =
          number < text_.size() &&
          (IsDigit(text_[number]) ||
           (text_[number] == '.' && number + 1 < text_.size() &&
            IsDigit(text_[number + 1])));
      if (numeric) {
        position_ = number;
        while (position_ < text_.size() && IsDigit(text_[position_])) {
          ++position_;
        }
        if (Consume('.')) {
          if (position_ == text_.size() || !IsDigit(text_[position_])) {
            Fail("Expected digits after decimal point");
            return std::nullopt;
          }
          while (position_ < text_.size() && IsDigit(text_[position_])) {
            ++position_;
          }
        }
        const auto unit = position_;
        while (position_ < text_.size() && IsName(text_[position_])) {
          ++position_;
        }
        if (position_ < text_.size() && text_[position_] == '%') {
          ++position_;
        }
        result.push_back({unit == position_ ? ExplorerValue::Kind::kNumber
                                            : ExplorerValue::Kind::kDimension,
                          std::string(text_.substr(start, position_ - start)),
                          start,
                          {}});
        continue;
      }
      auto name = Identifier();
      if (!name) {
        Fail("Unexpected token in declaration value");
        return std::nullopt;
      }
      if (Consume('(')) {
        auto arguments = Values(')', depth + 1);
        if (!arguments || !Consume(')')) {
          Fail("Unterminated function");
          return std::nullopt;
        }
        result.push_back({ExplorerValue::Kind::kFunction, std::move(*name),
                          start, std::move(*arguments)});
      } else {
        result.push_back(
            {ExplorerValue::Kind::kIdentifier, std::move(*name), start, {}});
      }
    }
    if (failed_ || after_comma) {
      Fail("Expected a value after ','");
      return std::nullopt;
    }
    return result;
  }
  const std::string_view text_;
  ExplorerDeclarationError error_;
  size_t position_ = 0;
  size_t value_count_ = 0;
  bool failed_ = false;
};
}  // namespace
std::optional<std::vector<ExplorerDeclaration>> ParseExplorerDeclarations(
    std::string_view text,
    ExplorerDeclarationError* error) {
  Parser parser(text);
  auto result = parser.Parse();
  if (!result && error) {
    *error = parser.error();
  }
  return result;
}
}  // namespace tuffite
