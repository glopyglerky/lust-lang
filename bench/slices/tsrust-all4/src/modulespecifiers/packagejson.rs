//! The parts of Go `internal/packagejson` that modulespecifiers uses.
//!
//! PORT: the full port in `src/frontend/packagejson.rs` is not thread-safe
//! (`Rc`), and module specifier generation runs on checker threads. This
//! file keeps the same data model (JSONValue, Expected, ExportsOrImports,
//! VersionPaths, InfoCacheEntry) as a thread-safe copy of a frontend value
//! (`PackageJson::of_frontend`), or with a small strict JSON parser:
//! - Duplicate object members are allowed and the last one wins, as with
//!   `json.AllowDuplicateNames(true)`. In objects kept as `JSONValue`, the
//!   member keeps the position of its first occurrence (OrderedMap.Set).
//! - Any syntax error makes the whole file unparseable (`parseable ==
//!   false`, empty fields), as a fatal Go v2 error does.

use crate::prelude::*;

use super::semver;
use std::sync::Arc;

// Go: packagejson/jsonvalue.go:10 JSONValueType
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JSONValueType {
    #[default]
    NotPresent,
    Null,
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl JSONValueType {
    // Go: packagejson/jsonvalue.go:22 String
    pub fn string(self) -> &'static str {
        match self {
            JSONValueType::Null => "null",
            JSONValueType::String => "string",
            JSONValueType::Number => "number",
            JSONValueType::Boolean => "boolean",
            JSONValueType::Array => "array",
            JSONValueType::Object => "object",
            JSONValueType::NotPresent => "unknown(0)",
        }
    }
}

// Go: packagejson/jsonvalue.go:41 JSONValue
// PORT: Go keeps `Type` plus `Value any`. The Rust enum holds both.
// `ExportsOrImports` is the same tree (see below).


// Go: packagejson/exportsorimports.go:8 objectKind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObjectKind {
    Unknown,
    Subpaths,
    Conditions,
    Imports,
    Invalid,
}

// Go: packagejson/exportsorimports.go:18 ExportsOrImports
// PORT: Go embeds JSONValue and adds a cached object kind. The kind is
// computed on demand, so the type is the same tree.

// Go: packagejson/expected.go:9 Expected
#[derive(Clone, Debug, Default)]
pub struct Expected<T> {
    pub null: bool,
    pub valid: bool,
    pub value: T,
}

impl<T: Clone> Expected<T> {
    // Go: packagejson/expected.go:43 GetValue
    pub fn get_value(&self) -> (T, bool) {
        (self.value.clone(), self.valid)
    }
}

// Go: packagejson/packagejson.go:8 HeaderFields, PathFields, DependencyFields


// Go: packagejson/packagejson.go:127 Parse
// PORT: `data` is the file's Go bytes. Go fails on invalid UTF-8 in a
// string. Each string value is in the port form (see
// `scanner_util::GO_STRING_MARKER`).

// Go: packagejson/expected.go:16 UnmarshalJSON

// Go: packagejson/expected.go:16 UnmarshalJSON
// PORT: Go decodes the map up to the first non-string value and keeps that
// partial map with `Valid == false`. This keeps the entries before the
// first non-string value too.

// Go: packagejson/cache.go:15 PackageJson


// Go: core/version.go:8 version

// Go: packagejson/cache.go:88 VersionPaths


// Go: packagejson/cache.go:123 InfoCacheEntry
// PORT: Go `Contents *PackageJson` is an `Arc`, so the copy that
// `with_package_directory` makes shares it, as Go's shallow copy does.


/// Strict RFC 8259 JSON parser for package.json text.
struct JsonParser<'a> {
    s: &'a [u8],
    pos: usize,
}

impl JsonParser<'_> {

    fn err(&self, msg: &str) -> String {
        format!("{msg} at offset {}", self.pos)
    }

    fn skip_ws(&mut self) {
        while let Some(&c) = self.s.get(self.pos) {
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }







}
