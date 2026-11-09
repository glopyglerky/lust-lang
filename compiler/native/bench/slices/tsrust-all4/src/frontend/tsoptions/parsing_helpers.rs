use crate::contentmapper::Mapper;
use crate::frontend::json_ext::JsonValue;
use crate::frontend::prelude::*;

// Port of tsoptions/parsinghelpers.go.
//
// PORT: Go `any` values are `CompilerOptionsValue` (U13). Go
// `*collections.OrderedMap[string, any]` is `CompilerOptionsValue::Map` or
// `IndexMap<String, CompilerOptionsValue>`. `IndexMap::insert` keeps the
// position of an existing key, like Go `OrderedMap.Set`.

// Go: tsoptions/parsinghelpers.go:16 ParseTristate

// Go: tsoptions/parsinghelpers.go:30 ParseStringArray
// PORT: Go returns a nil slice (`None`) when the value is not `[]any` and
// for a nil `[]any` (`NilList`). A `List` gives a non-nil slice.

// Go: tsoptions/parsinghelpers.go:46 parseStringMap
// PORT: the Go map values are `[]string`; a nil slice (a non-array value)
// is `None`, so `verifyCompilerOptions` can tell nil from empty.

// Go: tsoptions/parsinghelpers.go:57 ParseString

// Go: tsoptions/parsinghelpers.go:64 parseNumber
// PORT: Go `*int` (64-bit) is `Option<i64>`. Go `int(float64)` truncates
// toward zero, as `as i64` does in range. Out of range and NaN, Go leaves the
// result to the platform: amd64 gives the min int64, and Rust `as` would
// saturate, so the amd64 value is kept here.

// Go: tsoptions/parsinghelpers.go:75 projectReferenceParseResult
#[derive(Clone, Debug, Default)]
pub struct ProjectReferenceParseResult {
    pub reference: ProjectReference,
    pub has_path: bool,
    pub path_valid: bool,
    pub has_circular: bool,
    pub circular_valid: bool,
}

// Go: tsoptions/parsinghelpers.go:83 parseProjectReference
// PORT: Go returns a nilable pointer; that is `Option`.

// Go: tsoptions/parsinghelpers.go:105 parseContentMapper (tsgo#4712)
// PORT: Go returns a nilable `*contentmapper.Mapper`; that is an owned
// `Option<Mapper>`, which the caller changes and then puts in an `Rc`.

// Go: tsoptions/parsinghelpers.go:145 parseStringArrayStrict (tsgo#4712)
// parseStringArrayStrict returns the string slice and true only if value is an array whose
// elements are all strings. A missing element or wrong element type yields false.
// PORT: Go `([]string, bool)` is `Option<Vec<String>>`.

// Go: tsoptions/parsinghelpers.go:161 parseJsonToStringKey
// PORT: Go returns a map pointer that is never nil. It is `Option` so it
// can go straight to the nilable `json` argument of
// `parseJsonConfigFileContentWorker`; it is always `Some`.

// Go: tsoptions/parsinghelpers.go:198 optionParser
// PORT: Go `*diagnostics.Message` results are never nil for the ported
// parsers, so they are `&'static Message`.

// Go: tsoptions/parsinghelpers.go:204 compilerOptionsParser
// PORT: the embedded Go `*core.CompilerOptions` is a mutable borrow.


// Go: tsoptions/parsinghelpers.go:220 watchOptionsParser
// PORT: not ported. Watch mode is out of scope for this port, and the crate
// has no `core.WatchOptions` type. The Go parser has no caller in the
// tsconfig path (`watchOptions` parsing is commented out in Go).

// Go: tsoptions/parsinghelpers.go:236 typeAcquisitionParser
// PORT: the embedded Go `*core.TypeAcquisition` is a mutable borrow.
pub struct TypeAcquisitionParser<'a> {
    pub type_acquisition: &'a mut TypeAcquisition,
}


// Go: tsoptions/parsinghelpers.go:252 buildOptionsParser
// PORT: ported next to `core.BuildOptions` in execute/build/command_line.rs.

// Go: tsoptions/parsinghelpers.go:268 ParseCompilerOptions
// PORT: Go `allOptions` can be nil; every Rust caller has options, so the
// nil check is dropped.

// Go: tsoptions/parsinghelpers.go:279 parseCompilerOptions
// PORT: renamed, because the snake_case form of the Go name is the same as
// `ParseCompilerOptions`. Go `[]string` results go to `Option<Vec<String>>`
// fields, so an explicit empty list stays different from nil.

// PORT: Go `value.(T)` for the enum types of `floatOrInt32ToFlag`. One
// accessor per Go type argument.

// Go: tsoptions/parsinghelpers.go:570 floatOrInt32ToFlag
// PORT: the Go type parameter `T ~int32` is the pair (`typed`, `from_i32`):
// `typed` is Go `value.(T)` and `from_i32` is the Go conversion `T(...)`.
// Go `value.(float64)` panics on another type; so does this port. A JSON
// value of another type comes from a bad `.tsbuildinfo` (`"module": "x"`),
// and the panic has Go's runtime text.

// Go: tsoptions/parsinghelpers.go:577 ParseWatchOptions
// PORT: not ported. Watch mode is out of scope, and the crate has no
// `core.WatchOptions` type.

// Go: tsoptions/parsinghelpers.go:606 ParseTypeAcquisition
// PORT: Go `allOptions` can be nil; every Rust caller has one, so the nil
// check is dropped. Go `[]string` fields are `Vec<String>`.

// Go: tsoptions/parsinghelpers.go:626 ParseBuildOptions
// PORT: ported in execute/build/command_line.rs.


// Go: tsoptions/parsinghelpers.go:658 mergeCompilerOptions
// mergeCompilerOptions merges the source compiler options into the target compiler options
// with optional awareness of explicitly set null values in the raw JSON.
// Fields in the source options will overwrite the corresponding fields in the target options,
// including when they are explicitly set to null in the raw configuration (if rawSource is provided).
// PORT: Go loops over the struct fields by reflection. The Rust port lists
// each field with its Go json name, in Go field order. Go `IsZero` is
// `== Default::default()`. Go `[]string` fields are `Option<Vec<String>>`,
// so `None` (nil) is zero and `Some(vec![])` (empty non-nil) is not, as in
// Go. Go `rawSource any` is the map it must
// hold to have an effect, so it is `Option<&IndexMap>`. The Go merge
// copies the `Paths` pointer, so a later in-place change of `paths` also
// changes the source options; the Rust clone does not share it.

// Go: tsoptions/parsinghelpers.go:706 convertToOptionsWithAbsolutePaths
// PORT: Go changes the map through its pointer and returns it. The Rust
// port takes the map by value and returns it.

// Go: tsoptions/parsinghelpers.go:721 ConvertOptionToAbsolutePath

#[cfg(test)]
mod tests {
    use super::*;

    // Go keeps an explicit `[]` as a non-nil slice, so it overrides the
    // extended config in `mergeCompilerOptions`.
    #[test]
    fn empty_string_array_overrides_extended_config() {
        let mut target = CompilerOptions {
            custom_conditions: Some(vec!["@tanstack/custom-condition".to_string()]),
            types: Some(vec!["node".to_string()]),
            ..Default::default()
        };
        let mut source = CompilerOptions::default();
        let empty = CompilerOptionsValue::List(Vec::new());
        parse_compiler_options_worker("customConditions", &empty, &mut source);
        parse_compiler_options_worker("types", &empty, &mut source);
        merge_compiler_options(&mut target, Some(&source), None);
        assert_eq!(target.custom_conditions, Some(Vec::new()));
        assert_eq!(target.types, Some(Vec::new()));
        assert_eq!(target.root_dirs, None);
    }

    // Go keeps a nil `[]any` nil. An array of nulls and a null list option
    // leave the option unset, so the extended config keeps its value.
    #[test]
    fn nil_list_leaves_list_options_unset() {
        use crate::frontend::tsoptions::{
            COMPILER_NAME_MAP, convert_json_option_of_list_type, parse_config_file_text_to_json,
        };
        let (json, errors) = parse_config_file_text_to_json(
            "/tsconfig.json",
            Path("/tsconfig.json".to_string()),
            r#"{"types": [null, null], "lib": []}"#,
        );
        assert!(errors.is_empty());
        let CompilerOptionsValue::Map(json) = json else {
            panic!("tsconfig JSON is an object");
        };
        assert_eq!(json["types"], CompilerOptionsValue::NilList);
        assert_eq!(json["lib"], CompilerOptionsValue::List(Vec::new()));

        let plugins = COMPILER_NAME_MAP.get("plugins").expect("plugins option");
        let (null_plugins, errors) = convert_json_option_of_list_type(
            plugins,
            CompilerOptionsValue::Nil,
            "/",
            Node::NIL,
            Node::NIL,
            Node::NIL,
        );
        assert!(errors.is_empty());
        assert_eq!(null_plugins, CompilerOptionsValue::NilList);

        let mut source = CompilerOptions::default();
        parse_compiler_options_worker("types", &json["types"], &mut source);
        parse_compiler_options_worker("lib", &json["lib"], &mut source);
        parse_compiler_options_worker("plugins", &null_plugins, &mut source);
        assert_eq!(source.types, None);
        assert_eq!(source.lib, Some(Vec::new()));
        assert_eq!(source.plugins, None);

        let mut target = CompilerOptions {
            types: Some(vec!["node".to_string()]),
            ..Default::default()
        };
        merge_compiler_options(&mut target, Some(&source), None);
        assert_eq!(target.types, Some(vec!["node".to_string()]));
    }
}
