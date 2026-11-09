//! Port of packagejson/jsonvalue.go, expected.go, exportsorimports.go,
//! validated.go, packagejson.go and cache.go.
//!
//! PORT: Go decodes package.json with JSON v2 reflection. Here each Go type
//! implements `UnmarshalerFrom` (see `json.rs`) with the same v2 semantics:
//! - `Parse` sets `AllowDuplicateNames(true)`, so for a duplicate member the
//!   field is decoded again into the same Go value (the last one wins, with
//!   the Go merge quirks kept: `Expected.Valid` is never reset, and a scalar
//!   `JSONValue` of a different JSON kind is a fatal error).
//! - Every v2 error is fatal. `Parse` then returns zero `Fields` and the
//!   error. The resolver stores such a file with `Parseable == false` and
//!   empty fields, so an invalid file acts like a file with no fields.
//! - Go `any` in `JSONValue.Value` is the `JsonAny` enum.
//! - Go `*collections.OrderedMap` is `Rc<IndexMap>`. Go slices and pointers
//!   share their data on copy, so `Rc` keeps that.

use crate::frontend::prelude::*;
use std::cell::OnceCell;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// jsonvalue.go
// ---------------------------------------------------------------------------

// Go: jsonvalue.go:10 JSONValueType
// PORT: a newtype with associated consts so it works in `match` patterns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JSONValueType(pub i8);

impl JSONValueType {
    pub const NOT_PRESENT: JSONValueType = JSONValueType(0);
    pub const NULL: JSONValueType = JSONValueType(1);
    pub const STRING: JSONValueType = JSONValueType(2);
    pub const NUMBER: JSONValueType = JSONValueType(3);
    pub const BOOLEAN: JSONValueType = JSONValueType(4);
    pub const ARRAY: JSONValueType = JSONValueType(5);
    pub const OBJECT: JSONValueType = JSONValueType(6);

    // Go: jsonvalue.go:22 String
}

// PORT: Go `fmt.Stringer`, so `args!` can format the type.


// Go: jsonvalue.go:41 JSONValue


// Go: jsonvalue.go:88 UnmarshalJSONFrom





// PORT: the JSON v2 interface arshaler for `json.UnmarshalDecode(dec, &v.Value)`
// with `v.Value` of type `any`. `unmarshalJSONValueV2` handles null, objects
// and arrays itself, so only scalars reach here. A nil value picks the Go
// type from the JSON kind. A non-nil value (from an earlier duplicate
// member) keeps its Go type, so a different JSON kind is an error.

// ---------------------------------------------------------------------------
// expected.go
// ---------------------------------------------------------------------------

/// The Go `reflect.Kind` of `T` in `Expected[T].ExpectedJSONType`.
/// PORT: a trait in place of reflection.
pub trait ExpectedJsonKind {
    const JSON_TYPE: &'static str;
}

impl ExpectedJsonKind for String {
    const JSON_TYPE: &'static str = "string";
}
impl ExpectedJsonKind for bool {
    const JSON_TYPE: &'static str = "boolean";
}
impl<T> ExpectedJsonKind for Vec<T> {
    const JSON_TYPE: &'static str = "array";
}
impl<V> ExpectedJsonKind for FxHashMap<String, V> {
    const JSON_TYPE: &'static str = "object";
}
impl ExpectedJsonKind for i32 {
    const JSON_TYPE: &'static str = "number";
}
impl ExpectedJsonKind for i64 {
    const JSON_TYPE: &'static str = "number";
}
impl ExpectedJsonKind for u32 {
    const JSON_TYPE: &'static str = "number";
}
impl ExpectedJsonKind for u64 {
    const JSON_TYPE: &'static str = "number";
}
// Go `reflect.Float64` falls to the "unknown" default.
impl ExpectedJsonKind for f64 {
    const JSON_TYPE: &'static str = "unknown";
}

// Go: expected.go:9 Expected
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Expected<T> {
    // PORT: Go `string`; every value is a literal.
    actual_json_type: &'static str,
    pub null: bool,
    pub valid: bool,
    pub value: T,
}

impl<T: UnmarshalerFrom + ExpectedJsonKind + Default + Clone> Expected<T> {
    // Go: expected.go:16 UnmarshalJSON
    // PORT: never fails, like Go. `data` is one complete JSON value.

    // Go: expected.go:39 IsPresent
    #[must_use]
    pub fn is_present(&self) -> bool {
        !self.actual_json_type.is_empty()
    }

    // Go: expected.go:43 GetValue
    #[must_use]
    pub fn get_value(&self) -> (T, bool) {
        (self.value.clone(), self.valid)
    }

    // Go: expected.go:47 IsValid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    // Go: expected.go:51 ExpectedJSONType
    #[must_use]
    pub fn expected_json_type(&self) -> &'static str {
        T::JSON_TYPE
    }

    // Go: expected.go:69 ActualJSONType
    #[must_use]
    pub fn actual_json_type(&self) -> &'static str {
        self.actual_json_type
    }
}

// PORT: JSON v2 calls a v1 `UnmarshalJSON` method with the raw bytes of the
// next value.

// Go: expected.go:73 ExpectedOf

// ---------------------------------------------------------------------------
// exportsorimports.go
// ---------------------------------------------------------------------------

// Go: exportsorimports.go:8 objectKind
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ObjectKind {
    #[default]
    Unknown,
    Subpaths,
    Conditions,
    Imports,
    Invalid,
}

// Go: exportsorimports.go:18 ExportsOrImports
// PORT: Go embeds `JSONValue`. It is the `json_value` field plus `Deref`.


// Go: exportsorimports.go:25 UnmarshalJSONFrom


// ---------------------------------------------------------------------------
// validated.go
// ---------------------------------------------------------------------------

// Go: validated.go:3 TypeValidatedField
pub trait TypeValidatedField {
    fn is_present(&self) -> bool;
    fn is_valid(&self) -> bool;
    fn expected_json_type(&self) -> &'static str;
    fn actual_json_type(&self) -> &'static str;
}

impl<T: UnmarshalerFrom + ExpectedJsonKind + Default + Clone> TypeValidatedField for Expected<T> {
    fn is_present(&self) -> bool {
        Expected::is_present(self)
    }
    fn is_valid(&self) -> bool {
        Expected::is_valid(self)
    }
    fn expected_json_type(&self) -> &'static str {
        Expected::expected_json_type(self)
    }
    fn actual_json_type(&self) -> &'static str {
        Expected::actual_json_type(self)
    }
}

// ---------------------------------------------------------------------------
// packagejson.go
// ---------------------------------------------------------------------------

// Go: packagejson.go:8 HeaderFields
#[derive(Clone, Debug, Default)]
pub struct HeaderFields {
    pub name: Expected<String>,
    pub version: Expected<String>,
    pub type_: Expected<String>,
}

// Go: packagejson.go:14 PathFields

// Go: packagejson.go:24 DependencyFields
#[derive(Clone, Debug, Default)]
pub struct DependencyFields {
    pub dependencies: Expected<FxHashMap<String, String>>,
    pub dev_dependencies: Expected<FxHashMap<String, String>>,
    pub peer_dependencies: Expected<FxHashMap<String, String>>,
    pub optional_dependencies: Expected<FxHashMap<String, String>>,
}

impl DependencyFields {
    // Go: packagejson.go:34 HasDependency
    // HasDependency returns true if the package.json has a dependency with the given name
    // under any of the dependency fields (dependencies, devDependencies, peerDependencies,
    // optionalDependencies).
    #[must_use]
    pub fn has_dependency(&self, name: &str) -> bool {
        if self.dependencies.valid && self.dependencies.value.contains_key(name) {
            return true;
        }
        if self.dev_dependencies.valid && self.dev_dependencies.value.contains_key(name) {
            return true;
        }
        if self.peer_dependencies.valid && self.peer_dependencies.value.contains_key(name) {
            return true;
        }
        if self.optional_dependencies.valid && self.optional_dependencies.value.contains_key(name) {
            return true;
        }
        false
    }

    // Go: packagejson.go:58 RangeDependencies
    // PORT: Go map iteration order is random; `FxHashMap` order is arbitrary.
    pub fn range_dependencies(&self, mut f: impl FnMut(&str, &str, &str) -> bool) {
        let fields = [
            (&self.dependencies, "dependencies"),
            (&self.dev_dependencies, "devDependencies"),
            (&self.peer_dependencies, "peerDependencies"),
            (&self.optional_dependencies, "optionalDependencies"),
        ];
        for (field, field_name) in fields {
            if field.valid {
                for (name, version) in &field.value {
                    if !f(name, version, field_name) {
                        return;
                    }
                }
            }
        }
    }

    // Go: packagejson.go:89 GetRuntimeDependencyNames
    // PORT: Go ignores `ok`, so a map that failed validation part way still
    // counts. `collections.Set` is `FxHashSet`.
    #[must_use]
    pub fn get_runtime_dependency_names(&self) -> FxHashSet<String> {
        let deps = &self.dependencies.value;
        let peer_deps = &self.peer_dependencies.value;
        let opt_deps = &self.optional_dependencies.value;
        let count = deps.len() + peer_deps.len() + opt_deps.len();
        let mut names = FxHashSet::default();
        names.reserve(count);
        for name in deps.keys() {
            names.insert(name.clone());
        }
        for name in peer_deps.keys() {
            names.insert(name.clone());
        }
        for name in opt_deps.keys() {
            names.insert(name.clone());
        }
        names
    }
}

// Go: packagejson.go:110 Fields
// PORT: Go embeds the three field groups. They are nested fields here, and
// the promoted `DependencyFields` methods forward below.

// Go: packagejson.go:117 ContentMapperFields (tsgo#4712)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContentMapperFields {
    pub exec: Expected<Vec<String>>,
    pub compiler_options: Expected<Vec<String>>,
    pub dynamic_config: Expected<bool>,
}

// PORT: the JSON v2 struct arshaler for `ContentMapperFields` (tags `exec`,
// `compilerOptions`, `dynamicConfig`).

// Go `reflect.Struct` falls to the "unknown" default.
impl ExpectedJsonKind for ContentMapperFields {
    const JSON_TYPE: &'static str = "unknown";
}

// Go: packagejson.go:123 typeScriptFields (tsgo#4712)
#[derive(Clone, Debug, Default)]
struct TypeScriptFields {
    content_mapper: Expected<ContentMapperFields>,
}

// PORT: the JSON v2 struct arshaler for `typeScriptFields` (tag
// `contentMapper`).

// Go `reflect.Struct` falls to the "unknown" default.
impl ExpectedJsonKind for TypeScriptFields {
    const JSON_TYPE: &'static str = "unknown";
}



// PORT: the JSON v2 struct arshaler for `Fields` with its `json:"..."` tags.
// `ContentMapper` is `json:"-"`, so "contentMapper" is an unknown member.

// Go: packagejson.go:127 Parse
// PORT: Go returns `(Fields{}, err)` on error. The `Result` carries the same
// information; callers use `unwrap_or_default()` for the zero `Fields`.
// tsgo#4712: Go decodes into an anonymous struct that embeds the three field
// groups and adds `TypeScript Expected[typeScriptFields] json:"typescript"`.



// ---------------------------------------------------------------------------
// cache.go
// ---------------------------------------------------------------------------

// Go: cache.go:13 typeScriptVersion

// Go: cache.go:15 PackageJson
// PORT: Go `sync.Once` plus the `versionPaths` and `versionTraces` fields is
// one `OnceCell`. Go embeds `Fields`; `Deref` promotes it.


// Go: cache.go:23 diagnosticAndArgs
// PORT: Go `[]any` args are already strings here (`args!`).


// Go: cache.go:88 VersionPaths
// PORT: Go caches `paths` through a pointer field. Copies made before the
// first `GetPaths` call do not share the cache; `OnceCell` clones the same.


// Go: cache.go:123 InfoCacheEntry
// PORT: Go `*InfoCacheEntry` is `Rc<InfoCacheEntry>`; a nil entry is `None`
// at the call site, so the nil-receiver checks are gone.


/// A package.json read that a parse worker made first in the program load,
/// before the loader's cache has an entry for it (`InfoCache::add_pending`).
// PORT: not in Go. Go's parse tasks share the resolver's one cache, so the
// first read of a load is the cache entry. A worker's entry has `Rc`
// contents on the worker's thread, so the loader keeps the text and parses
// it only when a lookup asks for the entry (`InfoCache::get`).
#[derive(Debug)]
pub struct PendingInfo {
    pub package_directory: String,
    pub directory_exists: bool,
    /// The text of the package.json. `None`: it does not exist.
    pub text: Option<Box<str>>,
}

impl PendingInfo {
}

// Go: cache.go:169 InfoCache
// PORT: Go `collections.SyncMap` is a `RefCell` map. The cache is shared as
// `Rc<InfoCache>`, so its methods take `&self`.

// Go: cache.go:175 NewInfoCache


#[cfg(test)]
mod info_cache_tests {
    use super::*;

    /// A parse worker's read of the package.json `{ "name": name }` in
    /// `/p/<dir>` (`InfoCache::add_pending`).
    fn read(dir: &str, name: &str) -> PendingInfo {
        PendingInfo {
            package_directory: format!("/p/{dir}"),
            directory_exists: true,
            text: Some(format!(r#"{{ "name": "{name}" }}"#).into()),
        }
    }

    /// The `name` field of the package.json of `entry`.
    fn name(entry: &InfoCacheEntry) -> Option<String> {
        let contents = entry.contents.as_ref()?;
        Some(contents.fields.header_fields.name.get_value().0)
    }

    // followups32: Go `InfoCache.Set` (packagejson/cache.go:190
    // `LoadOrStore`) keeps the first stored entry, and Go's parse tasks
    // store their reads in that one cache. A pending read is a parse
    // worker's read from earlier in the load, so a later `set` of its key
    // gets the worker's entry and drops its own, and the pending read goes.
    // With no pending read, `set` stores its entry, and a later pending
    // read of the key is not kept. A `get` takes a pending read first.
    #[test]
    fn set_keeps_a_pending_read_as_the_first_entry() {
        let cache = new_info_cache("/p", true);
        cache.add_pending("/p/a/package.json", read("a", "a-worker"));
        let stored = cache.set("/p/a/package.json", read("a", "a-set").into_entry());
        assert_eq!(name(&stored).as_deref(), Some("a-worker"));
        let got = cache.get("/p/a/package.json").expect("the entry of a");
        assert!(Rc::ptr_eq(&got, &stored), "set stores the pending read");
        assert!(
            cache.pending.borrow().is_empty(),
            "set takes the pending read"
        );

        let stored = cache.set("/p/b/package.json", read("b", "b-set").into_entry());
        assert_eq!(name(&stored).as_deref(), Some("b-set"));
        cache.add_pending("/p/b/package.json", read("b", "b-worker"));
        assert!(
            cache.pending.borrow().is_empty(),
            "the entry of b came first"
        );
        let got = cache.get("/p/b/package.json").expect("the entry of b");
        assert_eq!(name(&got).as_deref(), Some("b-set"));

        cache.add_pending("/p/c/package.json", read("c", "c-worker"));
        let got = cache.get("/p/c/package.json").expect("the entry of c");
        let stored = cache.set("/p/c/package.json", read("c", "c-set").into_entry());
        assert!(Rc::ptr_eq(&got, &stored), "get stored the pending read");
        assert_eq!(name(&stored).as_deref(), Some("c-worker"));

        let mut entries = 0;
        cache.range(|_, _| {
            entries += 1;
            true
        });
        assert_eq!(entries, 3);
    }
}
