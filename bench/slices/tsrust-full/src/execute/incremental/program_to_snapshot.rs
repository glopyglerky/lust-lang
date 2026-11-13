//! Port of execute/incremental/programtosnapshot.go.
//!
//! PORT: Go `*compiler.Program` is the current program (`prog()`), read
//! through the `program.rs` free functions. Go runs the per-file work of
//! `computeProgramFileChanges` on a work group; here the checker part of
//! every file is sent to its checker thread first
//! (`start_referenced_files_job`), and the rest runs in file order.

use super::checker_access::*;
use super::hash::FileInfo;
use super::hash::*;
use super::snapshot::*;
use crate::frontend::prelude::*;
use std::sync::Arc;

// Go: incremental/programtosnapshot.go:16 programToSnapshot

// Go: incremental/programtosnapshot.go:42 toProgramSnapshot
// PORT: Go `program` is the current program and is not a field.




// Go: incremental/programtosnapshot.go:235 fileAffectsGlobalScope

// Go: incremental/programtosnapshot.go:260 addReferencedFilesFromSymbol
// PORT: the symbol belongs to the checker, so its arena is a parameter.

// Go: incremental/programtosnapshot.go:276 addReferencedFilesFromImportLiteral
// Get the module source file and all augmenting files from the import name node from file

/// The Go `referencedFiles` set of `getReferencedFiles`, in Go order.
// PORT: perf. Go inserts a path for every declaration of every ambient
// module, and the set keeps the first. `add_file` skips a file that it
// added before, without the path hash (in Hono, about 110 ambient module
// declarations for each file). The paths are kept as a list of
// candidates in Go order, so `finish` can give back the old snapshot's set
// when it is the same, without a copy of each path. The set and its order
// are the same.
#[derive(Default)]
struct ReferencedFileSet {
    /// What Go adds to the set, in order. A path can repeat.
    candidates: Vec<ReferencedCandidate>,
    /// The files that `add_file` added.
    files: FxHashSet<Node>,
}

/// One path that `getReferencedFiles` adds: the path of a source file, or
/// a path from a file name.
enum ReferencedCandidate {
    File(Node),
    Path(Path),
}

impl ReferencedFileSet {
    /// Adds the path of `file`.
    fn add_file(&mut self, file: Node) {
        if self.files.insert(file) {
            self.candidates.push(ReferencedCandidate::File(file));
        }
    }

    /// Adds `paths`, in order.
    fn add_paths(&mut self, paths: Vec<Path>) {
        self.candidates
            .extend(paths.into_iter().map(ReferencedCandidate::Path));
    }


}

// Go: incremental/programtosnapshot.go:282 addReferencedFileFromFileName
// Gets the path to reference file from file name, it could be resolvedPath if present otherwise path
// PORT: the paths are pushed in Go order; the checker job adds them to the
// set (see `start_referenced_files_job`).

// Go: incremental/programtosnapshot.go:291 getReferencedFiles
// Gets the referenced files for a file from the program with values for the keys as referenced file's path to be true







// Go: incremental/programtosnapshot.go:337 repopulateDiagnosticsOfFile
// repopulateDiagnosticsOfFile repopulates diagnostic chains that depend on program state.
// When diagnostics are copied from a previous build, their message chains may reference
// stale program state (e.g., resolved module alternate results, package.json scope).
// This function recomputes those chains using the current program's state.
// PORT: testing. `diags.clone()` keeps the entry id (Go returns the same
// pointer), and the new entry gets a new id (see
// `DiagnosticsOrBuildInfoDiagnosticsWithFileName`).

// Go: incremental/programtosnapshot.go:351 repopulateDiagnosticsList
// repopulateDiagnosticsList repopulates diagnostic chains in a list of diagnostics.
// Returns nil if no diagnostics needed repopulation (i.e., no changes were made).

// Go: incremental/programtosnapshot.go:373 repopulateDiagnosticMessageChain
// repopulateDiagnosticMessageChain repopulates chains that have repopulate info.
// Returns nil if no changes were made.

// Go: incremental/programtosnapshot.go:418 astDiagToBuildInfoDiag

#[cfg(test)]
mod tests {
    use super::*;

    // Go: incremental/external_diagnostic_test.go:14 TestExternalDiagnosticBuildInfoRoundTrip (tsgo#4712)
    // PORT: Go `toDiagnostic(nil, file)` has no program argument here.
    #[test]
    fn test_external_diagnostic_build_info_round_trip() {
        let file = parse_source_file(
            &SourceFileParseOptions {
                file_name: "/app.vue".to_string(),
                path: Path("/app.vue".to_string()),
                ..Default::default()
            },
            "",
            ScriptKind::TS,
        )
        .root;
        let diagnostic = crate::ast::new_external_diagnostic(
            file,
            TextRange::new(1, 2),
            "vue",
            crate::diagnostics::Category::Warning,
            1001,
            "mapper warning",
        );

        let serialized = ast_diag_to_build_info_diag(&diagnostic);
        assert_eq!(serialized.source, "vue");
        assert_eq!(serialized.message_text, "mapper warning");

        let restored = serialized.to_diagnostic(file);
        assert_eq!(restored.source(), "vue");
        assert_eq!(restored.localize(&crate::locale::DEFAULT), "mapper warning");
    }
}
