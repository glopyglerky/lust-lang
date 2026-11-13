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
pub fn new_default_user_preferences() -> UserPreferences {
    UserPreferences {
        format_code_settings: get_default_format_code_settings(),

        include_completions_for_module_exports: Tristate::True,
        include_completions_for_import_statements: Tristate::True,
        enable_auto_closing_tags: Tristate::True,
        enable_js_doc_completions: Tristate::True,
        generate_return_in_doc_template: Tristate::True,

        allow_rename_of_import_path: Tristate::True,
        provide_refactor_not_applicable_reason: Tristate::True,
        enable_formatting: Tristate::True,
        enable_validation: Tristate::True,
        display_parts_for_js_doc: Tristate::True,
        disable_line_text_in_references: Tristate::True,
        report_style_checks_as_warnings: Tristate::True,

        exclude_library_symbols_in_nav_to: Tristate::True,
        workspace_symbols_scope: WorkspaceSymbolsScope::ALL_OPEN_PROJECTS,
        ..Default::default()
    }
}

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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserPreferences {
    pub format_code_settings: FormatCodeSettings,

    pub quote_preference: QuotePreference,
    pub lazy_configured_projects_from_external_project: Tristate, // !!!

    // A positive integer indicating the maximum length of a hover text before it is truncated.
    //
    // Default: `500`
    pub maximum_hover_length: i32, // !!!

    // ------- Completions -------

    // If enabled, TypeScript will search through all external modules' exports and add them to the completions list.
    // This affects lone identifier completions but not completions on the right hand side of `obj.`.
    pub include_completions_for_module_exports: Tristate,
    // Enables auto-import-style completions on partially-typed import statements. E.g., allows
    // `import write|` to be completed to `import { writeFile } from "fs"`.
    pub include_completions_for_import_statements: Tristate,
    // Unless this option is `false`,  member completion lists triggered with `.` will include entries
    // on potentially-null and potentially-undefined values, with insertion text to replace
    // preceding `.` tokens with `?.`.
    pub include_automatic_optional_chain_completions: Tristate,
    // If enabled, completions for class members (e.g. methods and properties) will include
    // a whole declaration for the member.
    // E.g., `class A { f| }` could be completed to `class A { foo(): number {} }`, instead of
    // `class A { foo }`.
    pub include_completions_with_class_member_snippets: Tristate,
    // If enabled, object literal methods will have a method declaration completion entry in addition
    // to the regular completion entry containing just the method name.
    // E.g., `const objectLiteral: T = { f| }` could be completed to `const objectLiteral: T = { foo(): void {} }`,
    // in addition to `const objectLiteral: T = { foo }`.
    pub include_completions_with_object_literal_method_snippets: Tristate,
    pub jsx_attribute_completion_style: JsxAttributeCompletionStyle,
    pub enable_auto_closing_tags: Tristate,
    pub enable_js_doc_completions: Tristate,
    pub generate_return_in_doc_template: Tristate,

    // ------- AutoImports --------
    pub import_module_specifier_preference: ImportModuleSpecifierPreference, // !!!
    // Determines whether we import `foo/index.ts` as "foo", "foo/index", or "foo/index.js"
    pub import_module_specifier_ending: ImportModuleSpecifierEndingPreference, // !!!
    pub auto_import_specifier_exclude_regexes: Vec<String>,                    // !!!
    pub auto_import_file_exclude_patterns: Vec<String>,
    pub auto_import_entrypoint_directory_search: Tristate,
    pub prefer_type_only_auto_imports: Tristate,

    // ------- OrganizeImports -------

    // Indicates which deterministic preset should be used to sort imports.
    // "auto" detects the existing ordinal case sensitivity where possible.
    pub organize_imports_sort: OrganizeImportsSort, // !!!
    // Indicates whether imports should be organized in a case-insensitive manner.
    //
    // Default: TSUnknown ("auto" in strada), will perform detection
    pub organize_imports_ignore_case: Tristate, // !!!
    // Indicates whether imports should be organized via an "ordinal" (binary) comparison using the numeric value of their
    // code points, or via "unicode" natural sorting. This implementation is locale-agnostic and approximates the practical
    // import-sorting behavior rather than the full Unicode Collation Algorithm.
    //
    // Default: Ordinal
    pub organize_imports_collation: OrganizeImportsCollation, // !!!
    // Indicates the locale to use for "unicode" collation in legacy clients. This is accepted for compatibility, but
    // currently ignored because organize-import sorting is deterministic and locale-agnostic.
    //
    // This preference is ignored if organizeImportsCollation is not `unicode`.
    //
    // Default: `"en"`
    pub organize_imports_locale: String, // !!!
    // Indicates whether numeric collation should be used for digit sequences in strings. When `true`, will collate
    // strings such that `a1z < a2z < a100z`. When `false`, will collate strings such that `a1z < a100z < a2z`.
    //
    // This preference is ignored if organizeImportsCollation is not `unicode`.
    //
    // Default: `false`
    pub organize_imports_numeric_collation: Tristate, // !!!
    // Indicates whether accents and other diacritic marks are considered unequal for the purpose of sorting.
    //
    // This preference is ignored if organizeImportsCollation is not `unicode`.
    //
    // Default: `true`
    pub organize_imports_accent_collation: Tristate, // !!!
    // Indicates whether upper case or lower case should sort first.
    //
    // This permission is ignored if:
    //	- organizeImportsCollation is not `unicode`
    //	- organizeImportsIgnoreCase is `true`
    //	- organizeImportsIgnoreCase is `auto` and the auto-detected case sensitivity is case-insensitive.
    //
    // Default: `false`
    pub organize_imports_case_first: OrganizeImportsCaseFirst, // !!!
    // Indicates where named type-only imports should sort. "inline" sorts named imports without regard to if the import is type-only.
    //
    // Default: `auto`, which defaults to `last`
    pub organize_imports_type_order: OrganizeImportsTypeOrder, // !!!

    // ------- MoveToFile -------
    pub allow_text_changes_in_new_files: Tristate, // !!!

    // ------- Rename -------
    pub use_aliases_for_rename: Tristate,
    pub allow_rename_of_import_path: Tristate,

    // ------- CodeFixes/Refactors -------
    pub provide_refactor_not_applicable_reason: Tristate, // !!!

    // ------- InlayHints -------
    pub inlay_hints: InlayHintsPreferences,

    // ------- CodeLens -------
    pub code_lens: CodeLensUserPreferences,

    // ------- Definition -------
    pub prefer_go_to_source_definition: bool,

    // ------- Symbols -------
    pub exclude_library_symbols_in_nav_to: Tristate,
    pub workspace_symbols_scope: WorkspaceSymbolsScope,

    // ------- Misc -------
    pub enable_formatting: Tristate,
    pub enable_validation: Tristate,
    pub disable_suggestions: Tristate,             // !!!
    pub disable_line_text_in_references: Tristate, // !!!
    pub display_parts_for_js_doc: Tristate,        // !!!
    pub report_style_checks_as_warnings: Tristate,
    pub locale: String,

    // ------- ATA -------

    // DisableAutomaticTypeAcquisition is the deprecated setting from typescript.disableAutomaticTypeAcquisition.
    pub disable_automatic_type_acquisition: Tristate,
    // AutomaticTypeAcquisitionEnabled is the unified setting from tsserver.automaticTypeAcquisition.enabled under the js/ts section.
    // When set, it takes precedence over DisableAutomaticTypeAcquisition.
    pub automatic_type_acquisition_enabled: Tristate,
    // TODO: add tsserver.web.typeAcquisition.enabled under the js/ts section for the web variant when web support is implemented.

    // ------- Project Configuration -------

    // CustomConfigFileName specifies a custom config file name to use before defaulting to tsconfig.json/jsconfig.json.
    pub custom_config_file_name: String,
}

impl UserPreferences {
    // Go: ls/lsutil/userpreferences.go:189 (UserPreferences).IsATADisabled
    // IsATADisabled returns whether Automatic Type Acquisition is disabled based on user preferences.
    // It checks the unified setting (tsserver.automaticTypeAcquisition.enabled) first,
    // then falls back to the deprecated setting (disableAutomaticTypeAcquisition).
    pub fn is_ata_disabled(&self) -> bool {
        if !self.automatic_type_acquisition_enabled.is_unknown() {
            return !self.automatic_type_acquisition_enabled.is_true();
        }
        self.disable_automatic_type_acquisition.is_true()
    }
}

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
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct WorkspaceSymbolsScope(pub Cow<'static, str>);

impl WorkspaceSymbolsScope {
    pub const ALL_OPEN_PROJECTS: WorkspaceSymbolsScope =
        WorkspaceSymbolsScope(Cow::Borrowed("allOpenProjects"));
    pub const CURRENT_PROJECT: WorkspaceSymbolsScope =
        WorkspaceSymbolsScope(Cow::Borrowed("currentProject"));
}

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
#[derive(Clone, Debug, PartialEq)]
enum FieldValue {
    Tristate(Tristate),
    IndentStyle(IndentStyle),
    SemicolonPreference(SemicolonPreference),
    QuotePreference(QuotePreference),
    JsxAttributeCompletionStyle(JsxAttributeCompletionStyle),
    IncludeInlayParameterNameHints(IncludeInlayParameterNameHints),
    OrganizeImportsSort(OrganizeImportsSort),
    OrganizeImportsCollation(OrganizeImportsCollation),
    OrganizeImportsCaseFirst(OrganizeImportsCaseFirst),
    OrganizeImportsTypeOrder(OrganizeImportsTypeOrder),
    ImportModuleSpecifierPreference(ImportModuleSpecifierPreference),
    ImportModuleSpecifierEndingPreference(ImportModuleSpecifierEndingPreference),
    WorkspaceSymbolsScope(WorkspaceSymbolsScope),
    Int(i32),
    String(String),
    Bool(bool),
    StringSlice(Vec<String>),
}

impl FieldValue {
    // PORT: Go `reflect.Value.Type`.
    fn type_(&self) -> FieldType {
        match self {
            FieldValue::Tristate(_) => FieldType::Tristate,
            FieldValue::IndentStyle(_) => FieldType::IndentStyle,
            FieldValue::SemicolonPreference(_) => FieldType::SemicolonPreference,
            FieldValue::QuotePreference(_) => FieldType::QuotePreference,
            FieldValue::JsxAttributeCompletionStyle(_) => FieldType::JsxAttributeCompletionStyle,
            FieldValue::IncludeInlayParameterNameHints(_) => {
                FieldType::IncludeInlayParameterNameHints
            }
            FieldValue::OrganizeImportsSort(_) => FieldType::OrganizeImportsSort,
            FieldValue::OrganizeImportsCollation(_) => FieldType::OrganizeImportsCollation,
            FieldValue::OrganizeImportsCaseFirst(_) => FieldType::OrganizeImportsCaseFirst,
            FieldValue::OrganizeImportsTypeOrder(_) => FieldType::OrganizeImportsTypeOrder,
            FieldValue::ImportModuleSpecifierPreference(_) => {
                FieldType::ImportModuleSpecifierPreference
            }
            FieldValue::ImportModuleSpecifierEndingPreference(_) => {
                FieldType::ImportModuleSpecifierEndingPreference
            }
            FieldValue::WorkspaceSymbolsScope(_) => FieldType::WorkspaceSymbolsScope,
            FieldValue::Int(_) => FieldType::Int,
            FieldValue::String(_) => FieldType::String,
            FieldValue::Bool(_) => FieldType::Bool,
            FieldValue::StringSlice(_) => FieldType::StringSlice,
        }
    }

    // PORT: Go `reflect.Value.Bool` (Bool kinds).
    fn bool_(&self) -> bool {
        match self {
            FieldValue::OrganizeImportsCollation(v) => v.0,
            FieldValue::Bool(v) => *v,
            _ => panic!(
                "reflect: call of reflect.Value.Bool on {:?} Value",
                self.type_()
            ),
        }
    }

    // PORT: Go `reflect.Value.Int` (Int kinds).
    fn int(&self) -> i64 {
        match self {
            FieldValue::IndentStyle(v) => i64::from(v.0),
            FieldValue::OrganizeImportsSort(v) => i64::from(v.0),
            FieldValue::OrganizeImportsCaseFirst(v) => i64::from(v.0),
            FieldValue::OrganizeImportsTypeOrder(v) => i64::from(v.0),
            FieldValue::Int(v) => i64::from(*v),
            _ => panic!(
                "reflect: call of reflect.Value.Int on {:?} Value",
                self.type_()
            ),
        }
    }

    // PORT: Go `reflect.Value.String` (String kinds). The two
    // modulespecifiers enums are Go string types; these are their Go values.
    fn string(&self) -> String {
        match self {
            FieldValue::SemicolonPreference(v) => v.0.to_string(),
            FieldValue::QuotePreference(v) => v.0.to_string(),
            FieldValue::JsxAttributeCompletionStyle(v) => v.0.to_string(),
            FieldValue::IncludeInlayParameterNameHints(v) => v.0.to_string(),
            FieldValue::ImportModuleSpecifierPreference(v) => match v {
                ImportModuleSpecifierPreference::None => "",
                ImportModuleSpecifierPreference::Shortest => "shortest",
                ImportModuleSpecifierPreference::ProjectRelative => "project-relative",
                ImportModuleSpecifierPreference::Relative => "relative",
                ImportModuleSpecifierPreference::NonRelative => "non-relative",
            }
            .to_string(),
            FieldValue::ImportModuleSpecifierEndingPreference(v) => match v {
                ImportModuleSpecifierEndingPreference::None => "",
                ImportModuleSpecifierEndingPreference::Auto => "auto",
                ImportModuleSpecifierEndingPreference::Minimal => "minimal",
                ImportModuleSpecifierEndingPreference::Index => "index",
                ImportModuleSpecifierEndingPreference::Js => "js",
            }
            .to_string(),
            FieldValue::WorkspaceSymbolsScope(v) => v.0.to_string(),
            FieldValue::String(v) => v.clone(),
            _ => format!("<{:?} Value>", self.type_()),
        }
    }

    // PORT: Go `reflect.Value.IsZero`. An empty `Vec` stands for a nil slice
    // (see the file note).
    fn is_zero(&self) -> bool {
        match self {
            FieldValue::Tristate(v) => *v == Tristate::Unknown,
            FieldValue::IndentStyle(v) => v.0 == 0,
            FieldValue::SemicolonPreference(v) => v.0.is_empty(),
            FieldValue::QuotePreference(v) => v.0.is_empty(),
            FieldValue::JsxAttributeCompletionStyle(v) => v.0.is_empty(),
            FieldValue::IncludeInlayParameterNameHints(v) => v.0.is_empty(),
            FieldValue::OrganizeImportsSort(v) => v.0 == 0,
            FieldValue::OrganizeImportsCollation(v) => !v.0,
            FieldValue::OrganizeImportsCaseFirst(v) => v.0 == 0,
            FieldValue::OrganizeImportsTypeOrder(v) => v.0 == 0,
            FieldValue::ImportModuleSpecifierPreference(v) => {
                *v == ImportModuleSpecifierPreference::None
            }
            FieldValue::ImportModuleSpecifierEndingPreference(v) => {
                *v == ImportModuleSpecifierEndingPreference::None
            }
            FieldValue::WorkspaceSymbolsScope(v) => v.0.is_empty(),
            FieldValue::Int(v) => *v == 0,
            FieldValue::String(v) => v.is_empty(),
            FieldValue::Bool(v) => !*v,
            FieldValue::StringSlice(v) => v.is_empty(),
        }
    }
}

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
struct StructField {
    name: &'static str,
    raw_tag: &'static str,
    config_tag: &'static str,
    fallback_config_tag: &'static str,
    get: fn(&UserPreferences) -> FieldValue,
    set: fn(&mut UserPreferences, FieldValue),
}

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
#[derive(Clone)]
struct FieldInfo {
    raw_name: &'static str, // raw name for unstable section lookup (e.g., "quotePreference")
    config_path: &'static str, // dotted path for config (e.g., "preferences.quoteStyle")
    fallback_config_paths: Vec<ConfigPathInfo>,
    // PORT: Go `fieldPath []int` is the index path to the field in the
    // struct. The field accessors stand for it.
    get: fn(&UserPreferences) -> FieldValue,
    set: fn(&mut UserPreferences, FieldValue),
    raw_invert: bool,    // whether to invert boolean values for raw name
    config_invert: bool, // whether to invert boolean values for config path
}

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
fn collect_field_infos(fields: &'static [StructField]) -> Vec<FieldInfo> {
    let mut infos = Vec::new();
    for field in fields {
        let raw_tag = field.raw_tag;
        let config_tag = field.config_tag;

        if raw_tag.is_empty() && config_tag.is_empty() {
            crate::core::go_panic(format!(
                "raw or config tag required for field {}",
                field.name
            ));
        }

        let fallback_config_tag = field.fallback_config_tag;

        let mut info = FieldInfo {
            raw_name: "",
            config_path: "",
            fallback_config_paths: Vec::new(),
            get: field.get,
            set: field.set,
            raw_invert: false,
            config_invert: false,
        };

        // Parse raw tag: "name" or "name,invert"
        if !raw_tag.is_empty() {
            let parts: Vec<&'static str> = raw_tag.split(',').collect();
            info.raw_name = parts[0];
            for part in &parts[1..] {
                if *part == "invert" {
                    info.raw_invert = true;
                }
            }
        }

        // Parse config tag: "path.to.setting" or "path.to.setting,invert"
        if !config_tag.is_empty() {
            let config_path = parse_config_path_tag(config_tag);
            info.config_path = config_path.path;
            info.config_invert = config_path.invert;
        }
        if !fallback_config_tag.is_empty() {
            for tag in fallback_config_tag.split(';') {
                info.fallback_config_paths.push(parse_config_path_tag(tag));
            }
        }

        infos.push(info);
    }
    infos
}

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

impl UserPreferences {
    // Go: ls/lsutil/userpreferences.go:674 (UserPreferences).withConfig
    // PORT: Go has a value receiver; this works on a copy of `self`.
}

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

impl UserPreferences {
    // Go: ls/lsutil/userpreferences.go:866 (UserPreferences).ModuleSpecifierPreferences
    pub fn module_specifier_preferences(&self) -> crate::modulespecifiers::UserPreferences {
        crate::modulespecifiers::UserPreferences {
            import_module_specifier_preference: self.import_module_specifier_preference,
            import_module_specifier_ending: self.import_module_specifier_ending,
            auto_import_specifier_exclude_regexes: self
                .auto_import_specifier_exclude_regexes
                .clone(),
        }
    }

    // Go: ls/lsutil/userpreferences.go:758 (UserPreferences).ParsedAutoImportFileExcludePatterns
    // PORT: Go returns a nil `*SpecMatcher` for no patterns; that is `None`.

    // Go: ls/lsutil/userpreferences.go:762 (UserPreferences).IsModuleSpecifierExcluded
}

// Go: ls/lsutil/userpreferences.go:893 ParseUserPreferences

// PORT: Go `strings.ToLower` maps each rune with the simple (one rune)
// `unicode.ToLower`. The first char of Rust `char::to_lowercase` is that
// mapping (U+0130 is the only multi-char case, and it starts with 'i'). Rust
// `str::to_lowercase` also applies the final-sigma rule, which Go does not.
