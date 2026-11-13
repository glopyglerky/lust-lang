use crate::frontend::prelude::*;
use std::sync::LazyLock;

// This file ports tsoptions/tsconfigparsing.go lines 1 to 900.
// PORT: Go `any` values are `CompilerOptionsValue`. Go
// `*collections.OrderedMap[string, any]` is `IndexMap<String,
// CompilerOptionsValue>` (the `Map` variant). Go `[]*ast.Diagnostic` is
// `Vec<Diagnostic>`. Go `*ast.SourceFile` is the source file `Node`
// (`Node::NIL` is Go nil).


// Go: tsoptions/tsconfigparsing.go:26 extendsResult
// PORT: Go `collections.Set[string]` is `FxHashSet<String>`. Readers sort
// the keys (`core.InsertSorted`), so order does not matter.

// Go: tsoptions/tsconfigparsing.go:36 compilerOptionsDeclaration
// PORT: Go package-level vars are `LazyLock` statics of leaked
// declarations, so Go pointer identity is `std::ptr::eq`.

// Go: tsoptions/tsconfigparsing.go:42 compileOnSaveCommandLineOption

// Go: tsoptions/tsconfigparsing.go:48 extendsOptionDeclaration

// Go: tsoptions/tsconfigparsing.go:57 tsconfigRootOptionsMap

// Go: tsoptions/tsconfigparsing.go:94 configFileSpecs


// Go: tsoptions/tsconfigparsing.go:148 FileExtensionInfo
// tsgo#4712 removes it: extra extensions are plain strings now.

// Go: tsoptions/tsconfigparsing.go:153 ExtendedConfigCache
// PORT: Go returns a shared `*ExtendedConfigCacheEntry`, so this returns
// an `Rc`.

// Go: tsoptions/tsconfigparsing.go:157 ExtendedConfigCacheEntry
// PORT: Go pointers are `Option<Rc<..>>`. `ParseExtendedConfig` finishes
// the `TsConfigSourceFile` before the entry is shared, so no `RefCell`.


// Go: tsoptions/tsconfigparsing.go:170 parsedTsconfig
// PORT: Go `extendedConfigPath any` only ever holds a `[]string` (typed,
// so never Go nil once set) or Go nil. `Some` is "set", `None` is Go nil.

// Go: tsoptions/tsconfigparsing.go:178 parseOwnConfigOfJsonSourceFile

// Go: tsoptions/tsconfigparsing.go:283 TsConfigSourceFile
// PORT: Go `*configFileSpecs` is `Option<ConfigFileSpecs>`; readers copy
// or borrow it and never share it.

// Go: tsoptions/tsconfigparsing.go:289 tsconfigToSourceFile

// Go: tsoptions/tsconfigparsing.go:296 NewTsconfigSourceFileFromFilePath
// PORT: Go returns a pointer to a new value; this returns the value. The
// parser keeps source text for the program, so the text is leaked.
// PORT: Go keeps the parser fields (`ParseOptions()`) on the returned
// `*ast.SourceFile`. The port keeps them in the parse, so the parse is
// recorded (`program::note_parsed_source_file`), as for other parses
// outside a program. Then the API encoder finds them for the config file
// (`getConfigSourceFile`), and a program version that publishes the file's
// store gives it its parser fields.


// Go: tsoptions/tsconfigparsing.go:306 jsonConversionNotifier
// PORT: Go func field is a boxed `FnMut`. Callers pass
// `Option<&mut JsonConversionNotifier>` where Go passes a nilable pointer.
// `on_property_set` borrows the value (Go passes the map pointer that the
// result also holds) and returns only the diagnostics, because Go's only
// caller drops the returned value.

// Go: tsoptions/tsconfigparsing.go:311 convertConfigFileToObject


// Go: tsoptions/tsconfigparsing.go:341 isCompilerOptionsValue
// PORT: Go `reflect` kind tests are variant matches (Go `orderedMapType`, line 333, is the `Map` match). Slice kinds are `List`
// and `StringList`; `orderedMapType` is only the `Map` variant.


// Go: tsoptions/tsconfigparsing.go:375 validateJsonOptionValue

// Go: tsoptions/tsconfigparsing.go:404 convertJsonOptionOfListType
// PORT: Go returns a `[]any` that callers store in an `any`, where a nil
// slice is not Go nil. A non-nil input stays a non-nil `List` through
// `core.MapIndex` and `core.Filter`. A `NilList` input (an array of nulls)
// and a value that is not an array give a `NilList`, so the option stays
// unset. Go filters with `v != 0`, which never matches a JSON `float64`, so
// `Number(0.0)` is kept.

// Go: tsoptions/tsconfigparsing.go:434 configDirTemplate
pub const CONFIG_DIR_TEMPLATE: &str = "${configDir}";

// Go: tsoptions/tsconfigparsing.go:436 startsWithConfigDirTemplate

// Go: tsoptions/tsconfigparsing.go:444 normalizeNonListOptionValue

// Go: tsoptions/tsconfigparsing.go:457 convertJsonOption

// Go: tsoptions/tsconfigparsing.go:504 getExtendsConfigPathOrArray
// PORT: Go returns a `[]string` that may be nil. Every caller stores it in
// an `any`, where nil and empty are both non-nil, so this returns a `Vec`.

// Go: tsoptions/tsconfigparsing.go:553 getExtendsConfigPath

// Go: tsoptions/tsconfigparsing.go:590 tsConfigOptions
// PORT: Go declares this struct and never uses it. It is not ported, so no
// `core.ProjectReference` type is needed here.

// Go: tsoptions/tsconfigparsing.go:596 CommandLineOptionNameMap
// PORT: Go map type is a newtype. A Go nil map is an empty map; every
// non-nil map built here has entries, so `is_nil` is `is_empty`.


// Go: tsoptions/tsconfigparsing.go:615 commandLineOptionsToMap

// Go: tsoptions/tsconfigparsing.go:624 CommandLineCompilerOptionsMap

// Go: tsoptions/tsconfigparsing.go:626 convertMapToOptions

// Go: tsoptions/tsconfigparsing.go:634 convertOptionsFromJson

// Go: tsoptions/tsconfigparsing.go:664 convertArrayLiteralExpressionToJson
// PORT: A Go nil `[]any` (every element converted to nil) is still a
// non-nil `any`. It is a `NilList`, so readers can tell it from `[]`.

// Go: tsoptions/tsconfigparsing.go:694 directoryOfCombinedPath

// ParseConfigFileTextToJson parses the text of the tsconfig.json file
// fileName is the path to the config file
// jsonText is the text of the config file
// Go: tsoptions/tsconfigparsing.go:703 ParseConfigFileTextToJson

// Go: tsoptions/tsconfigparsing.go:715 ParseConfigHost
// PORT: Go `FS()` returns the `vfs.FS` interface. This returns a shared
// `Rc<dyn Fs>`, so the resolver host can keep it.

// Go: tsoptions/tsconfigparsing.go:720 resolverHost
// PORT: Go embeds the ParseConfigHost interface. This keeps its `FS()` and
// `GetCurrentDirectory()` values, because `ResolutionHost` returns borrows.
pub struct ResolverHost {
    pub fs: Rc<dyn Fs>,
    pub current_directory: String,
}

impl ResolverHost {
    // Go: tsoptions/tsconfigparsing.go:724 (*resolverHost).Trace
    pub fn trace(&self, _msg: &str) {}
}

impl ResolutionHost for ResolverHost {
    fn fs(&self) -> &dyn Fs {
        &*self.fs
    }

    fn get_current_directory(&self) -> &str {
        &self.current_directory
    }
}

// Go: tsoptions/tsconfigparsing.go:726 ParseJsonSourceFileConfigFileContent
// PORT: Go passes a pointer that no caller uses again, so the source file
// moves into the result.

// Go: tsoptions/tsconfigparsing.go:742 convertObjectLiteralExpressionToJson
// PORT: Go returns a nil map when `returnValue` is false. Every caller
// passes true; `Nil` stands for the nil map.

// convertToJson converts the json syntax tree into the json value and report errors
// This returns the json value (apart from checking errors) only if returnValue provided is true.
// Otherwise it just checks the errors and returns undefined
// Go: tsoptions/tsconfigparsing.go:794 convertToJson
// PORT: Go returns `struct{}{}` for a missing root. That is a non-nil
// value of no other type, so it is `CompilerOptionsValue::EmptyStruct`.

// Go: tsoptions/tsconfigparsing.go:814 isDoubleQuotedString

// Go: tsoptions/tsconfigparsing.go:818 convertPropertyValueToJson

// ParseJsonConfigFileContent parses the contents of a config file (tsconfig.json).
// jsonNode: The contents of the config file to parse
// host: Instance of ParseConfigHost used to enumerate files in folder.
// basePath: A root directory to resolve relative path entries in the config file to. e.g. outDir
// Go: tsoptions/tsconfigparsing.go:872 ParseJsonConfigFileContent

// Go: tsoptions/tsconfigparsing.go:882 normalizeJsonValue
// PORT: `CompilerOptionsValue` has no form for a Go `map[string]any` (Go
// sorts its keys into an ordered map), so only the ordered map case is
// ported. A Go typed slice (`reflect` case) is `StringList`; it cannot be
// nil.

// convertToObject converts the json syntax tree into the json value
// Go: tsoptions/tsconfigparsing.go:919 convertToObject

// Go: tsoptions/tsconfigparsing.go:927 getDefaultCompilerOptions
