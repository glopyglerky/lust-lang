//! Port of Go `internal/contentmapper/transform.go` (tsgo#4712).
//!
//! PORT: Go `*ast.SourceFile` is `Rc<ParsedSourceFile>` here, as in the
//! compiler host (`frontend/compiler/host.rs`). Go `SetContentMapperInfo` is
//! `ParsedSourceFile::set_content_mapper_info`. The canonical file holds its
//! supplemental files as `Rc`, and each supplemental file points back with a
//! `Weak`, so the two links make no `Rc` cycle (see that method).

use crate::contentmapper::prelude::*;

use crate::frontend::parser::{SourceFileParseOptions, };
use crate::frontend::tspath::Path;
use std::sync::Arc;

// Go: contentmapper/transform.go:14 SourceFiles
// SourceFiles is the canonical output and its unnamed supplemental compiler inputs.
// PORT: Go `Canonical` is nil in the zero value; that is `None`.

// Go: contentmapper/transform.go:26 TransformAndParse
// TransformAndParse runs the given content mapper's transform for a content-mapped source file and
// parses the resulting TypeScript, preserving the original file name and retaining the untransformed text
// on the source file. The mapper is supplied by the caller (which also owns the failure accounting) so it
// is neither re-resolved nor substituted here. It returns an error if the transform fails or the mapper
// produces invalid position mappings (a *spanmap.MappingError); the caller decides how to report the failure
// and what placeholder file to substitute. It is the shared implementation behind
// CompilerHost.GetContentMappedSourceFile.



// Go: contentmapper/transform.go:47 ParseResult
// ParseResult validates and parses one mapper result and all its supplemental outputs.



// Go: contentmapper/transform.go:123 isModuleVirtualExtension
pub(crate) fn is_module_virtual_extension(extension: &str) -> bool {
    [
        tspath::EXTENSION_MTS,
        tspath::EXTENSION_CTS,
        tspath::EXTENSION_MJS,
        tspath::EXTENSION_CJS,
    ]
    .contains(&extension)
}

// Go: contentmapper/transform.go:128 CheckSupplementalFileNameCollisions
// CheckSupplementalFileNameCollisions rejects compiler-assigned virtual filenames that name physical files.

#[cfg(test)]
mod tests {
    // Go: contentmapper/transform_test.go
    use super::*;

    fn astro_parse_options() -> SourceFileParseOptions {
        SourceFileParseOptions {
            file_name: "/component.astro".to_string(),
            path: Path("/component.astro".to_string()),
            ..Default::default()
        }
    }

    // PORT: no Go counterpart (cmpar1). A parse worker parses the virtual
    // text only for a result that `parse_result` parses: Go checks the
    // mappings, then the virtual extension, before its parse
    // (transform.go:48-58).
    #[test]
    fn a_worker_parses_only_what_parse_result_parses() {
        let text = "export const a = 1;";
        let verbatim = |end: usize| {
            Some(spanmap::new(&[spanmap::Segment {
                virtual_end: end as i32,
                original_end: text.len() as i32,
                kind: spanmap::Kind::VERBATIM,
                ..Default::default()
            }]))
        };
        let result = |mappings: Option<spanmap::SpanMap>, extension: &str| Result {
            text: text.to_string(),
            virtual_extension: extension.to_string(),
            mappings: mappings.map(Arc::new),
            ..Default::default()
        };
        let cases = [
            (result(verbatim(text.len()), ".ts"), true),
            (result(Some(spanmap::new(&[])), ".mts"), true),
            // The mapping ends past the virtual text.
            (result(verbatim(text.len() + 1), ".ts"), false),
            (result(None, ".ts"), false),
            (result(verbatim(text.len()), ".vue"), false),
        ];
        for (i, (result, parses)) in cases.into_iter().enumerate() {
            assert_eq!(parses_canonical_output(&result, text), parses, "case {i}");
            let parsed = parse_result(&astro_parse_options(), text, &Mapper::default(), "", result);
            assert_eq!(parsed.is_ok(), parses, "case {i}");
        }
    }

    // Go: transform_test.go:14 TestParseResultSupplementalFileExtensions
    #[test]
    fn test_parse_result_supplemental_file_extensions() {
        let mappings = Some(Arc::new(spanmap::new(&[])));
        let mapped = |extension: &str| MappedResult {
            virtual_extension: extension.to_string(),
            mappings: mappings.clone(),
            ..Default::default()
        };
        let result = Result {
            virtual_extension: ".ts".to_string(),
            mappings: mappings.clone(),
            supplemental: vec![
                mapped(".js"),
                mapped(".jsx"),
                mapped(".ts"),
                mapped(".tsx"),
                mapped(".mts"),
                mapped(".cts"),
                mapped(".json"),
            ],
            ..Default::default()
        };
        let mapper = Mapper {
            definition: Definition {
                extensions: vec![".astro".to_string()],
                ..Default::default()
            },
            manifest: Manifest {
                name: "mapper".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let files = parse_result(
            &astro_parse_options(),
            "",
            &mapper,
            "transform-identity",
            result,
        )
        .expect("parse result");
        let canonical = files.canonical.clone().expect("canonical file");
        assert_eq!(
            canonical.content_mapper_transform_identity(),
            "transform-identity"
        );
        let canonical_supplementals = canonical.supplemental_source_files();
        assert_eq!(canonical_supplementals.len(), files.supplemental.len());

        let expected: [(&str, ScriptKind); 7] = [
            ("/component.astro.0.js", ScriptKind::JS),
            ("/component.astro.1.jsx", ScriptKind::JSX),
            ("/component.astro.2.ts", ScriptKind::TS),
            ("/component.astro.3.tsx", ScriptKind::TSX),
            ("/component.astro.4.mts", ScriptKind::TS),
            ("/component.astro.5.cts", ScriptKind::TS),
            ("/component.astro.6.json", ScriptKind::JSON),
        ];
        assert_eq!(files.supplemental.len(), expected.len());
        for (i, (file_name, script_kind)) in expected.iter().enumerate() {
            let file = &files.supplemental[i];
            assert_eq!(file.file_name(), *file_name);
            assert_eq!(*file.path(), Path((*file_name).to_string()));
            assert_eq!(file.script_kind, *script_kind);
            assert_eq!(
                file.content_mapper_transform_identity(),
                "transform-identity"
            );
            assert!(Rc::ptr_eq(&canonical_supplementals[i], file));
            let file_canonical = file.canonical_source_file().expect("canonical source file");
            assert!(Rc::ptr_eq(&file_canonical, &canonical));
        }
    }

    // Go: transform_test.go:65 TestParseResultAllowsSupplementalModules
    #[test]
    fn test_parse_result_allows_supplemental_modules() {
        let mappings = Some(Arc::new(spanmap::new(&[])));
        let mapper = Mapper {
            definition: Definition {
                extensions: vec![".astro".to_string()],
                ..Default::default()
            },
            manifest: Manifest {
                name: "mapper".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let files = parse_result(
            &astro_parse_options(),
            "",
            &mapper,
            "",
            Result {
                text: "export {};".to_string(),
                virtual_extension: ".ts".to_string(),
                mappings: mappings.clone(),
                supplemental: vec![MappedResult {
                    text: "export const value = 1;".to_string(),
                    virtual_extension: ".mts".to_string(),
                    mappings: mappings,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("parse result");
        // PORT: Go `ast.IsExternalModule(file)` reads the parsed file's
        // indicator; `ast::is_external_module` reads the published store, and
        // this file is not published.
        assert!(files.supplemental[0].external_module_indicator.is_some());
    }

    // Go: transform_test.go:88 TestParseResultDoesNotLeakCanonicalModuleForcingToSupplementals
    #[test]
    fn test_parse_result_does_not_leak_canonical_module_forcing_to_supplementals() {
        let mappings = Some(Arc::new(spanmap::new(&[])));
        let mapper = Mapper {
            manifest: Manifest {
                name: "mapper".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let files = parse_result(
            &astro_parse_options(),
            "",
            &mapper,
            "",
            Result {
                text: "const canonical = 1;".to_string(),
                virtual_extension: ".mts".to_string(),
                mappings: mappings.clone(),
                supplemental: vec![MappedResult {
                    text: "const supplemental = 1;".to_string(),
                    virtual_extension: ".ts".to_string(),
                    mappings: mappings,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .expect("parse result");
        let canonical = files.canonical.expect("canonical file");
        assert!(
            canonical
                .parse_options()
                .external_module_indicator_options
                .force
        );
        assert!(
            !files.supplemental[0]
                .parse_options()
                .external_module_indicator_options
                .force
        );
    }
}
