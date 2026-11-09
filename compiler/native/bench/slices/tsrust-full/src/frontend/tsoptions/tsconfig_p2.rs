use crate::contentmapper::{Mapper, OptionPathSegment};
use crate::frontend::prelude::*;

// This file ports tsoptions/tsconfigparsing.go lines 901 to 1833.
// PORT: Go `any` values are `CompilerOptionsValue`. Go
// `*collections.OrderedMap[string, any]` is `IndexMap<String,
// CompilerOptionsValue>` (the `Map` variant). Go
// `collections.OrderedMap[string, string]` is `IndexMap<String, String>`
// (`FxIndexMap` in `get_file_names_from_config_specs`).
// Go `*ast.SourceFile` is the source file `Node` (`Node::NIL` is Go nil).
// Go `[][]string` extension groups are `Vec<Vec<String>>`. Go `int` is
// `i32`.

// Go: tsoptions/tsconfigparsing.go:941 getDefaultTypeAcquisition

// Go: tsoptions/tsconfigparsing.go:949 convertCompilerOptionsFromJsonWorker

// Go: tsoptions/tsconfigparsing.go:958 convertTypeAcquisitionFromJsonWorker

// Go: tsoptions/tsconfigparsing.go:964 parseOwnConfigOfJson
// PORT: Go stores a `[]string` (maybe nil) in the `any` field, so the field
// is never Go nil. It is always `Some` here.
// PORT: Go sets the converted `compileOnSave` in the caller's map, which
// becomes `raw`. The port changes its own copy, which becomes `raw`; the
// caller does not read `json` again.

// Go: tsoptions/tsconfigparsing.go:996 readJsonConfigFile
// PORT: the parser keeps source text for the program, so the text and the
// file name are leaked. The empty file gets its own node store, like the
// parsed one.

// Go: tsoptions/tsconfigparsing.go:1015 getExtendedConfig

// Go: tsoptions/tsconfigparsing.go:1052 ParseExtendedConfig
// PORT: Go returns a pointer; this returns the value. Callers wrap it in
// `Rc` where Go shares it.



// parseConfig just extracts options/include/exclude/files out of a config file.
// It does not resolve the included files.
// Go: tsoptions/tsconfigparsing.go:1082 parseConfig
// PORT: Go `json` is a nilable map pointer (`Option`, owned). Go
// `sourceFile` is a nilable pointer that this function changes, so it is
// `Option<&mut TsConfigSourceFile>`. The Go `applyExtendedConfig` closure
// is inlined into the loop over the extended config paths. Go
// `extendedConfigPath` only ever holds a `[]string`, so the Go string case
// is not reachable and not ported.

// Go: tsoptions/tsconfigparsing.go:1219 defaultIncludeSpec
pub const DEFAULT_INCLUDE_SPEC: &str = "**/*";

// Go: tsoptions/tsconfigparsing.go:1221 propOfRaw
// PORT: Go nil `sliceValue` is `None`.

// Go: tsoptions/tsconfigparsing.go:1210 getPropFromRaw (closure in parseJsonConfigFileContentWorker)
// PORT: the Go closure is a private function. `is_json` is Go
// `sourceFile == nil`. A nil element fails both element checks, as in Go.
// A Go `[]string` value makes the `.([]any)` assertion panic; so does this
// port.


// Go: tsoptions/tsconfigparsing.go:1226 isStringValue



// parseJsonConfigFileContentWorker parses the contents of a config file from json or json source file (tsconfig.json).
// json: The contents of the config file to parse
// sourceFile: sourceFile corresponding to the Json
// host: Instance of ParseConfigHost used to enumerate files in folder.
// basePath: A root directory to resolve relative path entries in the config file to. e.g. outDir
// resolutionStack: Only present for backwards-compatibility. Should be empty.
// Go: tsoptions/tsconfigparsing.go:1237 parseJsonConfigFileContentWorker
// PORT: Go `sourceFile` is a pointer that the result keeps, so it moves in
// and into `ParsedCommandLine.config_file`. The Go `getFileNames` and
// `getProjectReferences` closures run inline, in Go order. When `parseConfig`
// hits a cycle, Go options are nil and `mergeCompilerOptions` panics if
// `existingOptions` is set; here the merge is skipped.

// Go: tsoptions/tsconfigparsing.go:1536 canJsonReportNoInputFiles

// Go: tsoptions/tsconfigparsing.go:1542 shouldReportNoInputFiles
fn should_report_no_input_files(
    file_names: &[String],
    can_json_report_no_input_files: bool,
    resolution_stack: &[Path],
) -> bool {
    file_names.is_empty() && can_json_report_no_input_files && resolution_stack.is_empty()
}

// Go: tsoptions/tsconfigparsing.go:1546 validateSpecs
// PORT: Go `specs any` always holds a `[]any`, so it is a slice here.

// Go: tsoptions/tsconfigparsing.go:1572 specToDiagnostic

// Go: tsoptions/tsconfigparsing.go:1582 invalidTrailingRecursion

// Go: tsoptions/tsconfigparsing.go:1589 invalidDotDotAfterRecursiveWildcard
// PORT: Go string indexes are byte offsets, as are Rust `find` results.

// Go: tsoptions/tsconfigparsing.go:1613 GetTsConfigPropArrayElementValue
// PORT: Go returns `*ast.StringLiteral`; that is a `Node` (`NIL` for nil).

// Go: tsoptions/tsconfigparsing.go:1623 ForEachTsConfigPropArray
// PORT: Go `*T` results are `Option<T>`.

// Go: tsoptions/tsconfigparsing.go:1630 CreateDiagnosticAtReferenceSyntax
// PORT: Go returns a nilable `*ast.Diagnostic`; that is `Option`. Go reads
// `config.ConfigFile.SourceFile`, which panics for a nil `ConfigFile`.

// Go: tsoptions/tsconfigparsing.go:1642 createDiagnosticAtProjectReferenceProperty
// PORT: Go `*TsConfigSourceFile` is `Option<&TsConfigSourceFile>`; the Go
// variadic `args` is a `Vec`.

// Go: tsoptions/tsconfigparsing.go:1663 GetCallbackForFindingPropertyAssignmentByValue

// Go: tsoptions/tsconfigparsing.go:1674 GetOptionsSyntaxByArrayElementValue

// Go: tsoptions/tsconfigparsing.go:1682 getContentMapperSyntax (tsgo#4712)
// getContentMapperSyntax returns the tsconfig JSON node to attribute a diagnostic about the content
// mapper at index to: the value of subKey within that mapper's object (when subKey is non-empty),
// falling back to the mapper element, then to the "contentMappers" array. An index outside the array
// (e.g. -1) yields the array itself. Returns nil when there is no source file (JSON API).
// PORT: Go nil `*ast.Node` is `Node::NIL`.

// Go: tsoptions/tsconfigparsing.go:1706 GetContentMapperOptionDiagnosticLocation (tsgo#4712)
// PORT: Go returns `(*ast.SourceFile, core.TextRange)`; the file is a
// `Node` (`Node::NIL` for nil). Go `slices.Index` compares the mapper
// pointers (`Rc::ptr_eq`). Go indexes the array with a negative
// `segment.Index` and panics; here a negative index finds no element.

// Go: tsoptions/tsconfigparsing.go:1743 getContentMappersKeySyntax (tsgo#4712)
// getContentMappersKeySyntax returns the "contentMappers" property key node, used to attribute a
// diagnostic about the setting as a whole rather than a specific mapper.

// Go: tsoptions/tsconfigparsing.go:1754 getContentMapperExtensionSyntax (tsgo#4712)
// getContentMapperExtensionSyntax returns the node for a specific extension string within the content
// mapper at index, falling back to the "extensions" array or the mapper element.

// Go: tsoptions/tsconfigparsing.go:1769 setContentMapperDiagnosticLocation (tsgo#4712)
// setContentMapperDiagnosticLocation attaches a source location to a content mapper diagnostic when a
// tsconfig source file and node are available (the jsonSourceFile API), leaving it as a location-less
// compiler diagnostic otherwise (the JSON API).

// Go: tsoptions/tsconfigparsing.go:1777 ForEachPropertyAssignment
// PORT: Go `*T` results are `Option<T>`. The Go variadic `key2` is a slice.

// Go: tsoptions/tsconfigparsing.go:1793 getTsConfigObjectLiteralExpression

// Go: tsoptions/tsconfigparsing.go:1803 getSubstitutedPathWithConfigDirTemplate

// Go: tsoptions/tsconfigparsing.go:1807 getSubstitutedStringArrayWithConfigDirTemplate
// PORT: Go returns a nil slice for "no change"; that is `None`. Go passes
// the string as `any` to `startsWithConfigDirTemplate`.

// Go: tsoptions/tsconfigparsing.go:1823 handleOptionConfigDirTemplateSubstitution
// PORT: Go clones the shared `Paths` map before the first change (tsgo#4362)
// so a cached extended config keeps its value. Each options value here owns
// its `paths`, so no clone is needed.

/// PORT: not in Go (perf). The extension group of one file and the
/// `ChangeExtension` results of that file, as the two extension priority
/// checks below read them. Go computes the group in each check and makes a
/// new string for each changed extension; here the group is computed once
/// per file and each changed name is built in one reused buffer, with the
/// same text.
struct ExtensionPriority<'e> {
    /// The extension groups of the supported extensions.
    groups: &'e [Vec<&'e str>],
    /// The extensions of the groups that contain the file's extension, in
    /// order (Go `extensionGroup`).
    group: Vec<&'e str>,
    /// `ChangeExtension(file, ext)` of the last `changed` call.
    changed: String,
    /// `ChangeExtension(file, "")`, made at the first `changed` call.
    stem: Option<String>,
}

impl<'e> ExtensionPriority<'e> {
    fn new(groups: &'e [Vec<&'e str>]) -> Self {
        ExtensionPriority {
            groups,
            group: Vec::new(),
            changed: String::new(),
            stem: None,
        }
    }

    /// Starts on `file`: its extension group.
    fn start(&mut self, file: &str) {
        self.group.clear();
        self.stem = None;
        for group in self.groups {
            if file_extension_is_one_of(file, group) {
                self.group.extend_from_slice(group);
            }
        }
    }

}

// hasFileWithHigherPriorityExtension determines whether a literal or wildcard file has already been included that has a higher extension priority.
// file is the path to the file.
// Go: tsoptions/tsconfigparsing.go:1875 hasFileWithHigherPriorityExtension
// PORT: `priority` has the file's extension group (`ExtensionPriority::start`).

// Removes files included via wildcard expansion with a lower extension priority that have already been included.
// file is the path to the file.
// Go: tsoptions/tsconfigparsing.go:1907 removeWildcardFilesWithLowerPriorityExtension
// PORT: `priority` has the file's extension group (`ExtensionPriority::start`).

// getFileNamesFromConfigSpecs gets the file names from the provided config file specs that contain, files, include, exclude and
// other properties needed to resolve the file names
// configFileSpecs is the config file specs extracted with file names to include, wildcards to include/exclude and other details
// basePath is the base path for any relative file specifications.
// options is the Compiler options.
// host is the host used to resolve files and directories.
// extraExtensions are additional file extensions (e.g. from content mappers) to treat as supported.
// Go: tsoptions/tsconfigparsing.go:1934 getFileNamesFromConfigSpecs
// PORT: Go `options` can be nil only after a config cycle; Go
// `GetSupportedExtensions` then panics on the nil pointer, and so does this
// port.


// Go: tsoptions/tsconfigparsing.go:2026 GetSupportedExtensions
// PORT: Go returns the shared tspath tables; this returns owned copies.
// tsgo#4712: the extra extensions are plain extension strings.

// Go: tsoptions/tsconfigparsing.go:2050 GetSupportedExtensionsWithJsonIfResolveJsonModule
// PORT: Go `core.Same` compares slice identity. Content equality gives the
// same result here: a new Go slice equal to a tspath table gets `.json`
// appended, which equals the matching `WITH_JSON` table.

// Reads the config file and reports errors.
// Go: tsoptions/tsconfigparsing.go:2064 GetParsedCommandLineOfConfigFile
// PORT: Go returns a nilable `*ParsedCommandLine`; that is `Option`.

// Go: tsoptions/tsconfigparsing.go:2075 GetParsedCommandLineOfConfigFilePath

#[cfg(test)]
mod extension_priority_tests {
    use super::*;

    // `ExtensionPriority::changed` is `change_extension`, and `start` finds
    // the groups that Go's checks find.
    #[test]
    fn changed_matches_change_extension() {
        let groups: Vec<Vec<&str>> = vec![
            vec![".ts", ".tsx", ".d.ts"],
            vec![".cts", ".d.cts"],
            vec![".mts", ".d.mts"],
            vec![".js", ".jsx"],
            vec![".vue"],
        ];
        let mut priority = ExtensionPriority::new(&groups);
        for file in [
            "/a/b.ts",
            "/a/b.d.ts",
            "/a/b.tsx",
            "/a/b.d.mts",
            "/a/b.js",
            "/a.dir/b.c.jsx",
            "/a/b.json",
            "/a/b.vue",
            "/a/b",
            "/a/.ts",
        ] {
            priority.start(file);
            let expected: Vec<&str> = groups
                .iter()
                .filter(|group| file_extension_is_one_of(file, group))
                .flatten()
                .copied()
                .collect();
            assert_eq!(priority.group, expected, "{file}");
            for ext in [".ts", ".d.ts", ".tsx", ".d.mts", ".js", "vue"] {
                assert_eq!(
                    priority.changed(file, ext),
                    change_extension(file, ext),
                    "{file} {ext}"
                );
            }
        }
    }
}

#[cfg(test)]
mod top_level_array_tests {
    use super::*;
    use crate::frontend::vfs::osvfs_fs;

    struct Host;

    impl ParseConfigHost for Host {
        fn fs(&self) -> Rc<dyn Fs> {
            osvfs_fs()
        }
        fn get_current_directory(&self) -> String {
            "/p".to_string()
        }
    }

    // In Go (tsconfigparsing.go:1300) a top-level array that holds
    // `"files": []` has no `files` node, so CreateDiagnosticForNodeInSourceFile
    // (errors.go:93) dereferences nil. Go N panics with the runtime text and
    // exits 2 (projfuzz1 GO_CRASH_ARR).
    #[test]
    fn empty_files_in_a_top_level_array_panics_like_go() {
        let source_file = new_tsconfig_source_file_from_file_path(
            "/p/tsconfig.json",
            to_path("/p/tsconfig.json", "", true),
            r#"[{"files": []}]"#,
        );
        let payload = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            parse_json_source_file_config_file_content(
                source_file,
                &Host,
                "/p",
                None,
                None,
                "/p/tsconfig.json",
                &[],
                None,
            )
        }))
        .expect_err("Go N panics");
        let panic = payload
            .downcast_ref::<crate::core::GoPanic>()
            .expect("a Go panic, not a port panic");
        assert_eq!(
            panic.message,
            "runtime error: invalid memory address or nil pointer dereference"
        );
    }
}
