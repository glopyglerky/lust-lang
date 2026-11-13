use crate::api::prelude::*;

// Port of Go `internal/api/session.go`, lines 1-1271: the snapshot
// registries, `Session`, `NewSession`, the checker and language service
// setup, the `HandleRequest` dispatch and the snapshot, project, symbol and
// type handlers up to `handleGetTargetOfSignature`, the transpile handlers
// (tsgo#4849), then `handleGetImportAdderEdits` and `toAPITextEdits`
// (tsgo#3881) and `originalTextOffset` (tsgo#4712). Lines 1272-2484 are
// `session_p2.rs`.
//
// PORT notes for both files:
// - Go `*ast.Symbol`, `*checker.Type` and `*checker.Signature` carry their
//   data. A Rust `SymbolId`, `TypeId` or `SignatureId` is an index into the
//   arenas of one checker, so a registry entry keeps the checker that made
//   the handle: `(Rc<RefCell<Checker>>, handle)`. Reads borrow that checker.
// - Handlers call a checker method in its own statement
//   (`setup.checker.borrow_mut().x(..)`), so no borrow is held when a
//   `new_*_response` borrows the checker again to read the result.
// - A handle from the registry is used with the setup checker only through
//   `checker_symbol`, `checker_type` and `checker_signature`. Go can hand a
//   pointer of one checker to another checker. For a symbol, the setup
//   checker first catches up to the binder lineage, so it has every binder
//   symbol of a live file version under the same index, then uses the same
//   symbol, or a shadow of a symbol that the other checker made: a copy in
//   the setup checker's arena with the same id and no links
//   (`import_symbol`). A type or signature of another checker calls
//   `unported!`.
// - Go `defer setup.done()`: `CheckerSetup::done` is a `Release` guard. It
//   releases the checker when `setup` drops at the end of the handler.
// - Current program: node handles (`node_handle_from`, `resolve_node_handle`)
//   and the source file encoder read lazy JSDoc through `prog()`. So every
//   handler that reads a program keeps it current for its whole body.
//   `setup_checker` does it through `done`. The handlers with no checker
//   call `ls_program::enter` after `get_program`. The `resolve*PropertyOf*`
//   helpers have no project, so they enter the program of the checker that
//   owns the handle.
// - Go mutexes (`snapshotsMu`, the registry mutexes) are dropped: the
//   session runs on the dispatch thread (PORTING "Threads").
// - The profile handlers run `crate::pprof`, which writes profiles with no
//   samples (see its module comment).

use crate::api::encoder;
use crate::api::proto;
use crate::api::requestfilesystem;
use crate::astnav;
use crate::emitter::emitter::EmitOnly;
use crate::frontend::compiler;
use crate::frontend::core_context::{self, CheckerLifetime};
use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::frontend::tsoptions;
use crate::frontend::tspath;
use crate::frontend::vfs;
use crate::gostd::{self, GoError, errors};
use crate::ls;
use crate::ls::autoimport;
use crate::program::ls_program;
use crate::project;
use crate::transpile;
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

// Go: api/session.go:47 sessionIDCounter
pub static SESSION_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

// Go: api/session.go:53 snapshotData
// snapshotData holds the per-snapshot state including the snapshot itself
// and symbol/type registries scoped to this snapshot.
// Multiple clients may hold references to the same snapshot via ref counting;
// the registries are cleaned up when refCount reaches zero.
// PORT: registry values keep the checker that owns the handle (file header).

// Go: api/session.go:91 projectRegistryData
// projectRegistryData holds per-project type and signature registries.
// Types and signatures use per-checker sequential IDs, so the same local ID
// can appear in multiple projects. Separate maps per project prevent collisions
// and allow clean teardown when a project is removed.








// Go: api/session.go:407 Session
// Session represents an API session that provides programmatic access
// to TypeScript language services through the LSP server.
// It implements the Handler interface to process incoming API requests.
// The session supports multiple active snapshots, each with their own
// symbol and type registries for maintaining object identity.
// PORT: ts#64163 gives the session a snapshot host, and a project session
// only in LSP mode (Go nil is `None`).

// Go: api/session.go batchResponsePage (ts#64061)
#[derive(Debug, Default)]
pub struct BatchResponsePage {
    pub encoded_responses: Vec<JsonValue>,
}

// Go: api/session.go:284 `var _ Handler = (*Session)(nil)`
// Ensure Session implements Handler
// PORT: the `impl Handler for Session` below.

// Go: api/session.go:467 SessionOptions
// SessionOptions configures an API session.
#[derive(Clone, Debug, Default)]
pub struct SessionOptions {
    // UseBinaryResponses enables binary responses for msgpack protocol.
    pub use_binary_responses: bool,
}

// Go: api/session.go DefaultMaxResponseBytesPerPage (ts#64061)
// DefaultMaxResponseBytesPerPage leaves room for base64 expansion beneath V8's
// maximum string length while rounding down to an even decimal value.
pub const DEFAULT_MAX_RESPONSE_BYTES_PER_PAGE: i32 = 300_000_000;

// Go: api/session.go NewLSPSession (ts#64163)
// NewLSPSession creates a new API session with the given project session.

// Go: api/session.go NewStandaloneSession (ts#64163)
// NewStandaloneSession creates an API session with an independently owned snapshot host.

// Go: api/session.go newSession (ts#64163)
// PORT: returns the session by value so the two constructors can set their
// fields before it is shared.

// PORT: Go `project.SnapshotHost` satisfies `tsoptions.ParseConfigHost`
// through its `FS` and `GetCurrentDirectory` methods; the config handlers
// pass it (ts#64163). Rust needs the impl; it forwards to the inherent
// methods.

// Go: api/session.go languageServerSnapshotUpdate (ts#64204)


// Go: api/session.go snapshotOpenState (ts#64204)
#[derive(Clone, Debug, Default)]
pub struct SnapshotOpenState {
    pub open_projects: FxHashSet<tspath::Path>,
    pub open_files: FxHashSet<tspath::Path>,
}

// Go: api/session.go:544 snapshotHandle
// snapshotHandle creates a snapshot handle from a snapshot's ID.

// Go: api/session.go:592 checkerSetup
// checkerSetup holds the common context needed by handlers that require a type checker.
// PORT: Go `done func()` is the `Release` guard; it runs when the setup drops.







// Go: api/session.go isSourceFileResponseMethod (ts#64216)

// Go: api/session.go newBatchResponsePage (ts#64061)


// Go: api/session.go isValidCreateSourceFileScriptKind (ts#64216)

// Go: api/session.go apiBuildSystem (ts#64158)
// Wrapper for the API session for build orchestrator
// PORT: Go keeps the session; the system needs its snapshot host and project
// session (Go `session.snapshotHost.FS()` and `session.DefaultLibraryPath()`).


// PORT: Go passes the `tsc.System` as the `tsoptions.ParseConfigHost` of
// `ParseBuildCommandLine`; Rust needs the impl.

// Go: api/session.go:1943 transpileOutput (tsgo#4849)


// Go: api/session.go newResolvedModuleResponse (ts#64247)
// PORT: Go `resolution.IsResolved()` is false for a nil resolution.

// Go: api/session.go newResolvedTypeReferenceDirectiveResponse (ts#64247)



// Go: api/session.go:2935 toAPITextEdits (tsgo#3881, tsgo#4712)
// PORT: Go returns nil when an edit position is outside the original text;
// nil is `None`.

// Go: api/session.go:2958 originalTextOffset (tsgo#4712)
// PORT: Go takes `len(originalText)`; the port takes the text. The API
// session uses the UTF-8 encoding, so `position.character` counts Go bytes
// after the line start. Port offsets differ from Go offsets after a marker
// unit (see `scanner_util::GO_STRING_MARKER`), so the Go bytes after the
// line start give the port offset (`port_byte_offset`), as
// `Converters::line_and_character_to_position` does. Go does not stop at the
// line end, and `port_byte_offset` counts the bytes past the end, so the
// offset is past the port end exactly when Go's is past the Go end. A
// unit has at least as many port bytes as Go bytes, so a `character` past
// the port bytes after the line start is past the end without a scan (and
// fits no `i32`). Go `int` arithmetic is `i64` here; the offset is at most
// the text length, so it fits the `i32` that `PositionMap` takes.

// Go: api/session_textedit_test.go (tsgo#4712)
#[cfg(test)]
mod textedit_tests {
    use super::*;
    use crate::frontend::parser::{self, SourceFileParseOptions};

    // Go: api/session_textedit_test.go:14 TestToAPITextEditsUsesOriginalCoordinates
    // PORT: Go sets the info on the parsed `*ast.SourceFile`. Here the parse
    // is recorded (`program::note_parsed_source_file`) and the info is set on
    // the `ParsedSourceFile`, as the content mapper transform does.
    #[test]
    fn test_to_api_text_edits_uses_original_coordinates() {
        let source_file = Rc::new(parser::parse_source_file(
            &SourceFileParseOptions {
                file_name: "/app.vue".to_string(),
                path: tspath::Path("/app.vue".to_string()),
                ..Default::default()
            },
            "const transformed = true;",
            ScriptKind::TS,
        ));
        crate::program::note_parsed_source_file(&source_file);
        source_file.set_content_mapper_info(ContentMapperSourceFileInfo {
            original_text: "😀\nabc".to_string(),
            content_mapper: "mapper".to_string(),
            ..Default::default()
        });

        let edits = to_api_text_edits(
            source_file.root,
            &[lsproto::TextEdit {
                range: lsproto::Range {
                    start: lsproto::Position {
                        line: 1,
                        character: 1,
                    },
                    end: lsproto::Position {
                        line: 1,
                        character: 2,
                    },
                },
                new_text: "x".to_string(),
            }],
        );

        assert_eq!(
            edits,
            Some(vec![TextEdit {
                pos: 4,
                end: 5,
                new_text: "x".to_string(),
            }])
        );
    }

    // PORT: no Go test. The characters count Go bytes; marker units (an
    // invalid byte, a WTF-8 lone surrogate, a real U+FDD0, see
    // `scanner_util::GO_STRING_MARKER`) have more port bytes than Go bytes.
    // Go reads `ab\xe9cd` on line 1 at Go offset 2: character 3 is `c` (Go
    // offset 5, UTF-16 offset 5).
    #[test]
    fn to_api_text_edits_counts_go_bytes() {
        let edit = |line: u32, start: u32, end: u32| lsproto::TextEdit {
            range: lsproto::Range {
                start: lsproto::Position {
                    line,
                    character: start,
                },
                end: lsproto::Position {
                    line,
                    character: end,
                },
            },
            new_text: "x".to_string(),
        };
        // (Go bytes, line, start, end, UTF-16 pos and end; None past the end)
        let cases: [(&[u8], u32, u32, u32, Option<(i32, i32)>); 6] = [
            (b"\xe9\nab\xe9cd", 1, 3, 4, Some((5, 6))),
            (b"\xe9\nab\xe9cd", 1, 5, 5, Some((7, 7))),
            (b"\xe9\nab\xe9cd", 1, 6, 6, None),
            // A WTF-8 lone surrogate is 3 Go bytes and one UTF-16 unit (Go
            // `DecodeJSStringRune` in `ComputePositionMap`).
            (b"a\xed\xa0\x80b\r\nc", 0, 4, 5, Some((2, 3))),
            // A real U+FDD0 is 3 Go bytes and one UTF-16 unit.
            ("\u{FDD0}\u{FDD0}x\ny".as_bytes(), 0, 6, 7, Some((2, 3))),
            ("\u{FDD0}\u{FDD0}x\ny".as_bytes(), 1, 0, 1, Some((4, 5))),
        ];
        for (bytes, line, start, end, want) in cases {
            let original_text = crate::scanner_util::go_string_from_bytes(bytes.to_vec());
            let source_file = Rc::new(parser::parse_source_file(
                &SourceFileParseOptions {
                    file_name: "/app.vue".to_string(),
                    path: tspath::Path("/app.vue".to_string()),
                    ..Default::default()
                },
                "const transformed = true;",
                ScriptKind::TS,
            ));
            crate::program::note_parsed_source_file(&source_file);
            source_file.set_content_mapper_info(ContentMapperSourceFileInfo {
                original_text,
                content_mapper: "mapper".to_string(),
                ..Default::default()
            });
            let edits = to_api_text_edits(source_file.root, &[edit(line, start, end)]);
            let got = edits.map(|edits| (edits[0].pos, edits[0].end));
            assert_eq!(got, want, "{bytes:?} line {line} {start}..{end}");
        }
    }
}
