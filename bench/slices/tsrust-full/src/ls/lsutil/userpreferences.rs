use crate::ls::lsutil::prelude::*;

use crate::flags_macros::go_enum;
use crate::frontend::json::{
    JsonDecoder, JsonError, MarshalerTo, UnmarshalerFrom, 
};
use crate::modulespecifiers::{
    ImportModuleSpecifierEndingPreference, ImportModuleSpecifierPreference,
};
use std::borrow::Cow;
use std::sync::LazyLock;

// Port of Go `ls/lsutil/userpreferences.go`.
//
// PORT: Go walks the preference structs with `reflect`. Here a static field
// table (`USER_PREFERENCES_FIELDS`) lists every tagged field in Go field
// order, with the Go `raw` and `config` tag strings and a getter and setter.
// `collectFieldInfos` parses those tags as Go does. The table order decides
// the `withConfig` order and the `MarshalJSONTo` input order, as the Go field
// order does. Go `any` is `LspAny` and Go `map[string]any` is
// `IndexMap<String, LspAny>`. Go `nil` is `LspAny::Null`.
//
// PORT: Go `[]string` fields tell a nil slice from an empty one. Only
// `MarshalJSONTo` (nil is skipped) sees the difference. A Rust `Vec` has no
// nil, so an empty `Vec` counts as nil there.

// Go: ls/lsutil/userpreferences.go:16 NewDefaultUserPreferences

// Go: ls/lsutil/userpreferences.go:47 UserPreferences
// UserPreferences represents TypeScript language service preferences.
//
// Fields are populated using two tags:
//   - `raw:"name"` or `raw:"name,invert"` - TypeScript/raw name for unstable section lookup
//   - `config:"path.to.setting"` or `config:"path.to.setting,invert"` - VS Code nested config path
//
// At least one tag must be present on each preference field.
// The `,invert` modifier inverts boolean values (e.g., VS Code's "suppress" -> our "include").
// PORT: the tags are in `USER_PREFERENCES_FIELDS`.


// Go: ls/lsutil/userpreferences.go:209 InlayHintsPreferences
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InlayHintsPreferences {
    pub include_inlay_parameter_name_hints: IncludeInlayParameterNameHints,
    pub include_inlay_parameter_name_hints_when_argument_matches_name: Tristate,
    pub include_inlay_function_parameter_type_hints: Tristate,
    pub include_inlay_variable_type_hints: Tristate,
    pub include_inlay_variable_type_hints_when_type_matches_name: Tristate,
    pub include_inlay_property_declaration_type_hints: Tristate,
    pub include_inlay_function_like_return_type_hints: Tristate,
    pub include_inlay_enum_member_value_hints: Tristate,
}

// Go: ls/lsutil/userpreferences.go:220 CodeLensUserPreferences
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CodeLensUserPreferences {
    pub references_code_lens_enabled: Tristate,
    pub implementations_code_lens_enabled: Tristate,
    pub references_code_lens_show_on_all_functions: Tristate,
    pub implementations_code_lens_show_on_interface_methods: Tristate,
    pub implementations_code_lens_show_on_all_class_methods: Tristate,
}

// --- Enum Types ---

// Go: ls/lsutil/userpreferences.go:230 QuotePreference
// PORT: a Go string type. The value is always one of the Go constants, so a
// `&'static str` holds it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct QuotePreference(pub &'static str);

impl QuotePreference {
    pub const UNKNOWN: QuotePreference = QuotePreference("");
    pub const AUTO: QuotePreference = QuotePreference("auto");
    pub const DOUBLE: QuotePreference = QuotePreference("double");
    pub const SINGLE: QuotePreference = QuotePreference("single");
}

// Go: ls/lsutil/userpreferences.go:232 WorkspaceSymbolsScope
// PORT: a Go string type. It has no entry in `typeParsers`, so Go sets any
// string (`reflect.Value.SetString`); the value is a `Cow`.


// Go: ls/lsutil/userpreferences.go:246 JsxAttributeCompletionStyle
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JsxAttributeCompletionStyle(pub &'static str);

impl JsxAttributeCompletionStyle {
    pub const UNKNOWN: JsxAttributeCompletionStyle = JsxAttributeCompletionStyle("");
    pub const AUTO: JsxAttributeCompletionStyle = JsxAttributeCompletionStyle("auto");
    pub const BRACES: JsxAttributeCompletionStyle = JsxAttributeCompletionStyle("braces");
    pub const NONE: JsxAttributeCompletionStyle = JsxAttributeCompletionStyle("none");
}

// Go: ls/lsutil/userpreferences.go:255 IncludeInlayParameterNameHints
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct IncludeInlayParameterNameHints(pub &'static str);

impl IncludeInlayParameterNameHints {
    pub const NONE: IncludeInlayParameterNameHints = IncludeInlayParameterNameHints("");
    pub const ALL: IncludeInlayParameterNameHints = IncludeInlayParameterNameHints("all");
    pub const LITERALS: IncludeInlayParameterNameHints = IncludeInlayParameterNameHints("literals");
}

// Go: ls/lsutil/userpreferences.go:263 OrganizeImportsSort
go_enum!(OrganizeImportsSort, i32 {
    AUTO = 0;
    ORDINAL = 1;
    ORDINAL_IGNORE_CASE = 2;
    NATURAL = 3;
    NATURAL_IGNORE_CASE = 4;
});

// Go: ls/lsutil/userpreferences.go:273 OrganizeImportsCollation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct OrganizeImportsCollation(pub bool);

impl OrganizeImportsCollation {
    pub const ORDINAL: OrganizeImportsCollation = OrganizeImportsCollation(false);
    pub const UNICODE: OrganizeImportsCollation = OrganizeImportsCollation(true);
}

// Go: ls/lsutil/userpreferences.go:280 OrganizeImportsCaseFirst
go_enum!(OrganizeImportsCaseFirst, i32 {
    FALSE = 0;
    LOWER = 1;
    UPPER = 2;
});

// Go: ls/lsutil/userpreferences.go:288 OrganizeImportsTypeOrder
go_enum!(OrganizeImportsTypeOrder, i32 {
    AUTO = 0;
    LAST = 1;
    INLINE = 2;
    FIRST = 3;
});

// --- Reflection-based parsing infrastructure ---

// PORT: Go `reflect.Type` of a preference field. It is the key of
// `typeParsers` and `typeSerializers`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldType {
    Tristate,
    IndentStyle,
    SemicolonPreference,
    QuotePreference,
    JsxAttributeCompletionStyle,
    IncludeInlayParameterNameHints,
    OrganizeImportsSort,
    OrganizeImportsCollation,
    OrganizeImportsCaseFirst,
    OrganizeImportsTypeOrder,
    ImportModuleSpecifierPreference,
    ImportModuleSpecifierEndingPreference,
    WorkspaceSymbolsScope,
    Int,
    String,
    Bool,
    StringSlice,
}

// PORT: Go `reflect.Kind` of the preference field types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldKind {
    Bool,
    Int,
    Uint8,
    String,
    Slice,
}

impl FieldType {
    // PORT: Go `reflect.Type.Kind` (`core.Tristate` is a `byte`).
    fn kind(self) -> FieldKind {
        match self {
            FieldType::Tristate => FieldKind::Uint8,
            FieldType::IndentStyle
            | FieldType::OrganizeImportsSort
            | FieldType::OrganizeImportsCaseFirst
            | FieldType::OrganizeImportsTypeOrder
            | FieldType::Int => FieldKind::Int,
            FieldType::OrganizeImportsCollation | FieldType::Bool => FieldKind::Bool,
            FieldType::SemicolonPreference
            | FieldType::QuotePreference
            | FieldType::JsxAttributeCompletionStyle
            | FieldType::IncludeInlayParameterNameHints
            | FieldType::ImportModuleSpecifierPreference
            | FieldType::ImportModuleSpecifierEndingPreference
            | FieldType::WorkspaceSymbolsScope
            | FieldType::String => FieldKind::String,
            FieldType::StringSlice => FieldKind::Slice,
        }
    }
}

// PORT: a Go `reflect.Value` of a preference field, held by value. The
// variant is the Go field type.


// Go: ls/lsutil/userpreferences.go:300 typeParsers
// typeParsers maps reflect.Type to a function that parses a value into that type.
// PORT: the Go map is a match on the field type.

// Go: ls/lsutil/userpreferences.go:426 typeSerializers
// typeSerializers maps reflect.Type to a function that serializes a value of that type.
// For types which do not serialize as-is (tristate, enums, etc).
// PORT: the Go map is a match on the field type. The Go type assertion
// `val.(T)` panics on another type, as the `let .. else` does here.

// Go: ls/lsutil/userpreferences.go:525 configPathParsers
// configPathParsers provides field-specific config value parsers that override the default
// type-based parser when the VS Code config value format differs from the Go field type.
// PORT: the Go map is a match on the config path.

// PORT: one Go struct field as `reflect` sees it: the Go field name, the Go
// `raw`, `config` and `fallbackConfig` tags, and accessors that stand for the
// Go field index path.

// PORT: builds one `StructField`. `$variant` is the Go field type and the
// path is the field inside `UserPreferences`. `set` panics like Go
// `reflect.Value.Set` when the value has another type. The form with
// `fallback:` sets the Go `fallbackConfig` tag.
macro_rules! pref_field {
    ($name:literal, $raw:literal, $config:literal, $variant:ident, $($path:ident).+) => {
        pref_field!($name, $raw, $config, fallback: "", $variant, $($path).+)
    };
    ($name:literal, $raw:literal, $config:literal, fallback: $fallback:literal, $variant:ident, $($path:ident).+) => {
        StructField {
            name: $name,
            raw_tag: $raw,
            config_tag: $config,
            fallback_config_tag: $fallback,
            get: |p| FieldValue::$variant(p.$($path).+.clone()),
            set: |p, v| match v {
                FieldValue::$variant(x) => p.$($path).+ = x,
                other => panic!(
                    "reflect.Set: value of type {:?} is not assignable to type {}",
                    other.type_(),
                    stringify!($variant)
                ),
            },
        }
    };
}

// PORT: the fields of `UserPreferences` in Go field order. Go
// `collectFieldInfos` recurses into the untagged struct fields
// (`FormatCodeSettings`, its embedded `EditorSettings`, `InlayHints`,
// `CodeLens`) at their place in the order; they are listed inline here.

// Go: ls/lsutil/userpreferences.go:547 fieldInfo

// Go: ls/lsutil/userpreferences.go:556 configPathInfo
#[derive(Clone, Copy)]
struct ConfigPathInfo {
    path: &'static str,
    invert: bool,
}

// Go: ls/lsutil/userpreferences.go:561 fieldInfoCache

// Go: ls/lsutil/userpreferences.go:566 unstableNameIndex
// unstableNameIndex maps raw names to fieldInfo index for unstable section lookup.

// Go: ls/lsutil/userpreferences.go:577 collectFieldInfos
// PORT: Go takes a `reflect.Type` and an index path and recurses into
// untagged struct fields. The field table is already flat, so every entry
// is a tagged field; the Go panic for an untagged non-struct field stays.

// Go: ls/lsutil/userpreferences.go:628 parseConfigPathTag
fn parse_config_path_tag(tag: &'static str) -> ConfigPathInfo {
    let mut parts = tag.split(',');
    let mut info = ConfigPathInfo {
        path: parts.next().unwrap_or(""),
        invert: false,
    };
    for part in parts {
        if part == "invert" {
            info.invert = true;
        }
    }
    info
}

// Go: ls/lsutil/userpreferences.go:639 getNestedValue
// PORT: Go starts from `any(config)`. `current` is `None` only for that root
// map, before the first part is read.

// Go: ls/lsutil/userpreferences.go:655 setNestedValue

// Go: ls/lsutil/userpreferences.go:669 setRawFieldsFromConfig
// PORT: Go takes the struct as a `reflect.Value`; here it is `p`.


// Go: ls/lsutil/userpreferences.go:746 getFieldByPath
// PORT: not ported. `FieldInfo.get` and `FieldInfo.set` stand for the field
// that Go finds by its index path.

// Go: ls/lsutil/userpreferences.go:753 setFieldFromValue
// PORT: Go takes the field as a `reflect.Value`. Here the field is `info` on
// `p`. Go also accepts a Go `int` for an int field; an `LspAny` number is
// always the Go `float64` case. Go `int64(float64)` keeps 64 bits; the Rust
// field is `i32` (PORTING `int` -> `i32`), so a value outside the `i32` range
// differs from Go.

// Go: ls/lsutil/userpreferences.go:656 (*UserPreferences).MarshalJSONTo

// PORT: the Go `json.Deterministic(true)` map key order, applied to every
// nested object. `String` `Ord` is the bytewise Go `slices.Sort` order.


// Go: ls/lsutil/userpreferences.go:826 serializeField
// PORT: Go `nil` is `LspAny::Null`. A Go `int` is written as a JSON number;
// `LspAny::Number` is an `f64`, which holds every `i32` exactly.

// Go: ls/lsutil/userpreferences.go:716 (*UserPreferences).UnmarshalJSONFrom

// --- Helper methods ---


// Go: ls/lsutil/userpreferences.go:893 ParseUserPreferences

// PORT: Go `strings.ToLower` maps each rune with the simple (one rune)
// `unicode.ToLower`. The first char of Rust `char::to_lowercase` is that
// mapping (U+0130 is the only multi-char case, and it starts with 'i'). Rust
// `str::to_lowercase` also applies the final-sigma rule, which Go does not.
