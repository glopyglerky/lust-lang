//! Port of tsoptions/showconfig.go (`tsc --showConfig`).

use crate::frontend::prelude::*;


// Go: tsoptions/showconfig.go:15 computeFn
// computeFn wraps a typed getter method so it can be stored in an
// impliedOption's compute field (which has type func(*core.CompilerOptions) any).
// PORT: Go `any` is `CompilerOptionsValue`. The table below uses
// non-capturing closures that wrap each getter result in its variant.

// Go: tsoptions/showconfig.go:23 impliedOption
// impliedOption describes a compiler option whose effective value can be derived from
// other options. This mirrors TypeScript's computedOptions concept used in convertToTSConfig.

// Go: tsoptions/showconfig.go:35 impliedOptions
// impliedOptions lists the compiler options that may be implied by other options,
// mirroring TypeScript's computedOptions used in convertToTSConfig.
// Each compute function delegates directly to an existing core.CompilerOptions getter.


// Go: tsoptions/showconfig.go:53 TSConfig (JSON v2 struct marshaler)
// PORT: Go marshals the struct by reflection in field order with the tags
// `json:"compilerOptions"`, `json:"references,omitzero"`,
// `json:"files,omitzero"`, `json:"include,omitzero"`,
// `json:"exclude,omitzero"` and `json:"compileOnSave,omitzero"`. The
// `any` values use the v2 marshaler for their dynamic type (`marshal_any`),
// and the options map uses `collections.OrderedMap.MarshalJSONTo` (members
// in insertion order).

// Go: tsoptions/showconfig.go:64 ConvertToTSConfig
// ConvertToTSConfig generates a complete tsconfig representation for --showConfig output,
// matching the behavior of TypeScript's convertToTSConfig function.

// Go: tsoptions/showconfig.go:141 filterSameAsDefaultInclude
// filterSameAsDefaultInclude returns nil if specs is the default include spec ["**/*"]
// PORT: an empty `Vec` is the Go nil slice.
fn filter_same_as_default_include(specs: &[String]) -> Vec<String> {
    if specs.is_empty() {
        return Vec::new();
    }
    if specs.len() == 1 && specs[0] == DEFAULT_INCLUDE_SPEC {
        return Vec::new();
    }
    specs.to_vec()
}

// Go: tsoptions/showconfig.go:153 getNameOfCompilerOptionValue
// getNameOfCompilerOptionValue returns the string key for a given enum value by
// searching the option's enum map.
// PORT: Go `v == value` on two `any` values is `PartialEq` on
// `CompilerOptionsValue` (same dynamic type and value). Go returns "" when
// nothing matches.


// Go: tsoptions/showconfig.go:165 serializeCompilerOptions
// serializeCompilerOptions converts CompilerOptions to an ordered map with
// string names as keys and serialized values (enums as strings, paths as
// relative paths, etc.) matching the output of tsc --showConfig.


// Go: tsoptions/showconfig.go:280 serializeEnumValue
// serializeEnumValue converts an enum field value to its corresponding string key
// using the option's enum map. It handles int32-based enum types.

// Go: tsoptions/showconfig.go:300 addImpliedOptions
// addImpliedOptions adds compiler options that are implied by other explicitly-set options,
// mirroring TypeScript's convertToTSConfig behavior for computedOptions.
// For example, when module: nodenext is set, moduleResolution: nodenext is implied.

// Go: tsoptions/showconfig.go:352 anyDependencyProvided
// anyDependencyProvided returns true if any of the given dependency names
// (using Go field names like "Target") corresponds to an option in the provided set.
// PORT: Go `map[string]bool` with only true values is a set.

// Go: tsoptions/showconfig.go:365 serializeImpliedOptionValue
// serializeImpliedOptionValue converts a computed implied option value to its serializable form.
// For enum options, it converts numeric values to their string names.
// For boolean options, it returns the bool directly.
// PORT: Go returns nil `any` as `CompilerOptionsValue::Nil`.

#[cfg(test)]
mod tests {
    use super::*;

    // Expected bytes are tsgo `--showConfig` output (execute/tsc.go:405,
    // prefix "", indent four spaces) for a solution config with two references.
    #[test]
    fn marshal_matches_tsgo_show_config() {
        let mut circular = IndexMap::new();
        circular.insert(
            "path".to_string(),
            CompilerOptionsValue::String("./packages/b/tsconfig.json".to_string()),
        );
        circular.insert("circular".to_string(), CompilerOptionsValue::Bool(true));
        let mut plain = IndexMap::new();
        plain.insert(
            "path".to_string(),
            CompilerOptionsValue::String("./packages/a".to_string()),
        );
        let config = TsConfig {
            references: vec![
                CompilerOptionsValue::Map(plain),
                CompilerOptionsValue::Map(circular),
            ],
            ..TsConfig::default()
        };
        let mut out: Vec<u8> = Vec::new();
        crate::frontend::json::json_marshal_indent_write(&mut out, &config, "", "    ").unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "{\n    \"compilerOptions\": {},\n    \"references\": [\n        {\n            \"path\": \"./packages/a\"\n        },\n        {\n            \"path\": \"./packages/b/tsconfig.json\",\n            \"circular\": true\n        }\n    ]\n}"
        );
    }

    // ts#64397: Go `reflect.Value.IsZero` of `Plugins` is false for a
    // non-nil empty slice, so tsgo `--showConfig` prints `"plugins": []`
    // for a config that sets it, and nothing for a config that does not.
    #[test]
    fn show_config_keeps_an_empty_plugins_list() {
        let compare = ComparePathsOptions::default();
        let shown = |plugins: Option<Vec<crate::options::PluginImport>>| {
            let options = CompilerOptions {
                plugins,
                ..CompilerOptions::default()
            };
            serialize_compiler_options(&options, "/project/tsconfig.json", &compare)
        };
        assert_eq!(
            shown(Some(Vec::new())).get("plugins"),
            Some(&CompilerOptionsValue::List(Vec::new()))
        );
        assert_eq!(shown(None).get("plugins"), None);
    }
}
