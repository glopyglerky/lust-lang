//! Port of Go `ls/lsconv/converters.go`.

use crate::ls::lsconv::prelude::*;

use crate::frontend::bundled::is_bundled;
use crate::frontend::scanner::scanner_p1::{RUNE_ERROR, };
use crate::frontend::tspath;
use crate::gostd;
use crate::locale;
use crate::lsp::lsproto;
use crate::scanner_util::{};
use crate::spanmap::{self, Feature, Fidelity, SpanMap};
use std::ops::Deref;
use std::sync::LazyLock;

// Go: ls/lsconv/converters.go:25 Converters
// PORT: Go `getLineMap func(fileName string) *LSPLineMap`; a nil map is `None`.
pub struct Converters {
    get_line_map: Box<dyn Fn(&str) -> Option<Rc<LSPLineMap>>>,
    position_encoding: lsproto::PositionEncodingKind,
}

// Go: ls/lsconv/converters.go:30 MappedSpan
// PORT: Go embeds `spanmap.MappedSpan`; `Deref` gives its fields (`span`,
// `fidelity`) as Go field promotion does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MappedSpan<T> {
    pub script: T,
    pub mapped_span: spanmap::MappedSpan,
}

impl<T> Deref for MappedSpan<T> {
    type Target = spanmap::MappedSpan;

    fn deref(&self) -> &spanmap::MappedSpan {
        &self.mapped_span
    }
}

// Go: ls/lsconv/converters.go:35 MappedPosition
// PORT: Go embeds `spanmap.MappedPosition`; `Deref` gives its fields
// (`position`, `fidelity`) as Go field promotion does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MappedPosition<T> {
    pub script: T,
    pub mapped_position: spanmap::MappedPosition,
}

impl<T> Deref for MappedPosition<T> {
    type Target = spanmap::MappedPosition;

    fn deref(&self) -> &spanmap::MappedPosition {
        &self.mapped_position
    }
}

// Go: ls/lsconv/converters.go:44 Script
// Script is a source text the converters operate over. For a content-mapped file, Text() is the content
// mapper's virtual output and SpanMap() returns the map from that output back to the original text
// (OriginalText()); virtual ranges are then automatically converted to original coordinates (see
// ToLSPRange). For an ordinary file SpanMap() is nil and OriginalText() equals Text().
// PORT: tsgo#4712 adds `OriginalFileName`, `SpanMap` and `OriginalText`.
// The defaults are Go's answers for a script that is not content-mapped
// (Go `script` in ls/source_map.go, `originalTextScript`, the project
// overlays), so implementors outside the ls lane keep compiling.
pub trait Script {
    fn file_name(&self) -> &str;

    fn original_file_name(&self) -> &str {
        self.file_name()
    }

    fn text(&self) -> ScriptText<'_>;

    fn span_map(&self) -> Option<&SpanMap> {
        None
    }

    fn original_text(&self) -> ScriptText<'_> {
        self.text()
    }
}

/// The text of a `Script`: a borrow of the script, or the text of a source
/// file (`FileText`), which a file node cannot lend. It derefs to `str`.
// PORT: Go returns a string. A source file's text can be a freeable
// version's text (textleak1), so the reader holds it while it reads.
#[derive(Clone)]
pub enum ScriptText<'a> {
    Borrowed(&'a str),
    File(FileText),
}


// PORT: Go passes pointers (`*ast.SourceFile`, `*testScript`, `*Overlay`)
// as a `Script`; the generic `FromLSPRange` and `FromLSPPosition` keep the
// pointer as `T`. A reference to a script is a script, so `T` can be a
// reference here.
impl<S: Script + ?Sized> Script for &S {
    fn file_name(&self) -> &str {
        (**self).file_name()
    }

    fn original_file_name(&self) -> &str {
        (**self).original_file_name()
    }

    fn text(&self) -> ScriptText<'_> {
        (**self).text()
    }

    fn span_map(&self) -> Option<&SpanMap> {
        (**self).span_map()
    }

    fn original_text(&self) -> ScriptText<'_> {
        (**self).original_text()
    }
}

// PORT: Go `*ast.SourceFile` implements Script through its `FileName`,
// `OriginalFileName`, `Text`, `SpanMap` and `OriginalText` methods. Here the
// file is its root `Node`. The inherent `Node::text` (Go `Node.Text`) wins
// in method syntax, so pass a file as `&dyn Script` (or call
// `Script::text(&file)`), as Go passes it as a Script.

// Go: ls/lsconv/converters.go:52 NewConverters
// PORT: Go returns `*Converters`, which the session, snapshots and language
// services share; here `Rc<Converters>`.
pub fn new_converters(
    position_encoding: lsproto::PositionEncodingKind,
    get_line_map: impl Fn(&str) -> Option<Rc<LSPLineMap>> + 'static,
) -> Rc<Converters> {
    Rc::new(Converters {
        get_line_map: Box::new(get_line_map),
        position_encoding,
    })
}

impl Converters {
    /// The position encoding. A cross-project search thread makes its own
    /// converters with it (`ls/search_thread.rs`).
    pub fn position_encoding(&self) -> lsproto::PositionEncodingKind {
        self.position_encoding.clone()
    }

    /// Go `c.getLineMap(fileName)` followed by a dereference.
    // PORT: Go dereferences the returned pointer, which panics when it is nil.
    fn line_map_of(&self, file_name: &str) -> Rc<LSPLineMap> {
        (self.get_line_map)(file_name).unwrap_or_else(|| crate::core::go_nil_dereference())
    }

    // Go: ls/lsconv/converters.go:66 ToLSPRange
    // ToLSPRange converts a range in a SourceFile (or a script read from the file system after declaration
    // mapping) to an lsproto.Range. If the file is a content-mapped virtual SourceFile, the range is mapped
    // through the file's span map and the fidelity of that mapping is returned. For normal files, the second
    // return value is FidelityExact.

    // Go: ls/lsconv/converters.go:78 ToLSPRangeForFeature
    // ToLSPRangeForFeature is [Converters.ToLSPRange] for an LS feature. For a content-mapped file, it returns
    // FidelityNone unless the entire virtual range is covered by contiguous segments that participate in
    // feature; the returned range is still the best-effort mapped range. For normal files, it behaves like
    // [Converters.ToLSPRange].

    // Go: ls/lsconv/converters.go:89 ToLSPPosition
    // ToLSPPosition converts a position in a SourceFile (or a script read from the file system after
    // declaration mapping) to an lsproto.Position. Positions in content-mapped files are mapped through
    // the file's span map; positions in normal files return FidelityExact.

    // Go: ls/lsconv/converters.go:98 ToLSPPositionForFeature
    // ToLSPPositionForFeature is [Converters.ToLSPPosition] for an LS feature. For a content-mapped file, it returns
    // FidelityNone when the virtual position is not in a segment that participates in feature; the
    // returned position is still the best-effort mapped position. For normal files, it behaves like
    // [Converters.ToLSPPosition].

    // Go: ls/lsconv/converters.go:107 ToLSPLocation
    // ToLSPLocation converts a range in a SourceFile or script to an lsproto.Location. If the file is a content-mapped
    // virtual SourceFile, the range is mapped through the file's span map and the fidelity of that mapping is returned.
    // For normal files, the second return value is FidelityExact. If the file is a supplemental output of a content mapper,
    // the file's original file name is used for the URI (e.g. App.astro.0.ts -> App.astro).

    // Go: ls/lsconv/converters.go:119 ToLSPLocationForFeature
    // ToLSPLocationForFeature is [Converters.ToLSPLocation] for an LS feature. For a content-mapped file, it returns
    // FidelityNone when the virtual position is not in a segment that participates in feature; the
    // returned position is still the best-effort mapped position. For normal files, it behaves like
    // [Converters.ToLSPLocation].
}

// Go: ls/lsconv/converters.go:127 (*Converters).FromLSPRange
// PORT: Go 1.27 makes this a (generic) method of `*Converters` (ts#63902).
// The Rust function keeps `c` as its first parameter, so callers in other
// lanes' files do not change.
// FromLSPRange converts an lsproto.Range to offsets in one Script. For a content-mapped script, results
// include each virtual projection covered by segments that participate in feature; it returns no
// results when no projection qualifies. Normal scripts return one exact span.

// Go: ls/lsconv/converters.go:134 (*Converters).FromLSPRangeForSourceFile
// PORT: Go 1.27 makes this a (generic) method of `*Converters` (ts#63902).
// The Rust function keeps `c` as its first parameter, so callers in other
// lanes' files do not change.
// FromLSPRangeForSourceFile converts an lsproto.Range to offsets in a SourceFile. When the file has
// supplemental content-mapper outputs, results include every qualifying virtual projection across the
// canonical and supplemental files. Projections not participating in feature are omitted.

// Go: ls/lsconv/converters.go:149 (*Converters).FromLSPRangeIntersectingForSourceFile
// PORT: Go 1.27 makes this a (generic) method of `*Converters` (ts#63902).
// The Rust function keeps `c` as its first parameter, so callers in other
// lanes' files do not change.
// FromLSPRangeIntersectingForSourceFile projects every feature-enabled intersection with textRange
// across the canonical and supplemental virtual files. Unlike FromLSPRangeForSourceFile, the original
// range endpoints need not be mapped. This is intended for read-only range requests such as semantic
// tokens and inlay hints, where an editor commonly asks for a viewport spanning host markup:
//
//	original: <template>...</template><script>const x = 1</script><style>...</style>
//	          [---------------- requested viewport ---------------------------------)
//	                                          [----------) mapped script
//
// The result contains the script intersection even though both viewport endpoints are outside it.

// Go: ls/lsconv/converters.go:177 (*Converters).lspRangeToVirtualForScripts
// PORT: Go 1.27 makes this a (generic) method of `*Converters` (ts#63902).
// The Rust function keeps `c` as its first parameter, so callers in other
// lanes' files do not change.

impl Converters {
    // Go: ls/lsconv/converters.go:187 (*Converters).lspRangeToVirtual
}

// Go: ls/lsconv/converters.go:211 (*Converters).FromLSPPosition
// PORT: Go 1.27 makes this a (generic) method of `*Converters` (ts#63902).
// The Rust function keeps `c` as its first parameter, so callers in other
// lanes' files do not change.
// FromLSPPosition converts an lsproto.Position to offsets in one Script. For a content-mapped script,
// results include each virtual projection whose segment participates in feature; it returns no results
// when no projection qualifies. Normal scripts return one exact position.

// Go: ls/lsconv/converters.go:218 (*Converters).FromLSPPositionForSourceFile
// PORT: Go 1.27 makes this a (generic) method of `*Converters` (ts#63902).
// The Rust function keeps `c` as its first parameter, so callers in other
// lanes' files do not change.
// FromLSPPositionForSourceFile converts an lsproto.Position to offsets in a SourceFile. When the file has
// supplemental content-mapper outputs, results include every qualifying virtual projection across the
// canonical and supplemental files. Projections not participating in feature are omitted.

// Go: ls/lsconv/converters.go:224 (*Converters).FromLSPRangeToOriginal
// PORT: Go 1.27 makes this a (generic) method of `*Converters` (ts#63902).
// The Rust function keeps `c` as its first parameter, so callers in other
// lanes' files do not change.
// FromLSPRangeToOriginal converts an LSP range in a content-mapped document directly to original-text offsets.
// PORT: Go takes the `Script` interface value; here `&dyn Script`.

// Go: ls/lsconv/converters.go:232 sourceFileProjections

// Go: ls/lsconv/converters.go:239 (*Converters).lspPositionToVirtualForScripts
// PORT: Go 1.27 makes this a (generic) method of `*Converters` (ts#63902).
// The Rust function keeps `c` as its first parameter, so callers in other
// lanes' files do not change.

impl Converters {
    // Go: ls/lsconv/converters.go:249 (*Converters).lspPositionToVirtual
}

/// The Go `Script` interface value that `virtualRangeToOriginal`,
/// `virtualPositionToOriginal` and `diagnosticScriptAndRange` return: the
/// script they were given, or an `originalTextScript` over its original
/// text.
enum ScriptOrOriginal<'a> {
    Script(&'a dyn Script),
    Original(OriginalTextScript<'a>),
}


// Go: ls/lsconv/converters.go:261 virtualRangeToOriginal
// virtualRangeToOriginal maps a content mapper's virtual range back to its original text.
// A nil feature bypasses feature filtering for diagnostics and edits.
// PORT: Go `feature *spanmap.Feature`; nil is `None`.
fn virtual_range_to_original(
    script: &dyn Script,
    text_range: TextRange,
    feature: Option<Feature>,
) -> (ScriptOrOriginal<'_>, TextRange, Fidelity) {
    let Some(span_map) = script.span_map() else {
        return (
            ScriptOrOriginal::Script(script),
            text_range,
            Fidelity::EXACT,
        );
    };
    let (mapped, fidelity) = match feature {
        None => SpanMap::virtual_to_original_span(Some(span_map), text_range),
        Some(feature) => {
            SpanMap::virtual_to_original_span_for_feature(Some(span_map), text_range, feature)
        }
    };
    (
        ScriptOrOriginal::Original(OriginalTextScript {
            file_name: script.original_file_name(),
            text: script.original_text(),
        }),
        mapped,
        fidelity,
    )
}

// Go: ls/lsconv/converters.go:276 virtualPositionToOriginal
// virtualPositionToOriginal is the single-position analog of virtualRangeToOriginal.
fn virtual_position_to_original(
    script: &dyn Script,
    position: i32,
    feature: Option<Feature>,
) -> (ScriptOrOriginal<'_>, i32, Fidelity) {
    let Some(span_map) = script.span_map() else {
        return (ScriptOrOriginal::Script(script), position, Fidelity::EXACT);
    };
    let (mapped, fidelity) = match feature {
        None => SpanMap::virtual_to_original_position(Some(span_map), position),
        Some(feature) => {
            SpanMap::virtual_to_original_position_for_feature(Some(span_map), position, feature)
        }
    };
    (
        ScriptOrOriginal::Original(OriginalTextScript {
            file_name: script.original_file_name(),
            text: script.original_text(),
        }),
        mapped,
        fidelity,
    )
}

// Go: ls/lsconv/converters.go:596 originalTextScript
// originalTextScript presents a content-mapped file's original (untransformed) text as a Script, so that
// ranges already mapped into that text convert to the correct line/character positions.
// PORT: Go copies the two strings; here the script borrows them.
struct OriginalTextScript<'a> {
    file_name: &'a str,
    text: ScriptText<'a>,
}


// Go: ls/lsconv/converters.go:290 LanguageKindToScriptKind
// PORT: Go passes the string value; here by reference.
#[must_use]
pub fn language_kind_to_script_kind(language_id: &lsproto::LanguageKind) -> ScriptKind {
    match &*language_id.0 {
        "typescript" => ScriptKind::TS,
        "typescriptreact" => ScriptKind::TSX,
        "javascript" => ScriptKind::JS,
        "javascriptreact" => ScriptKind::JSX,
        "json" => ScriptKind::JSON,
        _ => ScriptKind::UNKNOWN,
    }
}

// Go: ls/lsconv/converters.go:308 extraEscapeReplacer
// https://github.com/microsoft/vscode-uri/blob/edfdccd976efaf4bb8fdeca87e97c47257721729/src/uri.ts#L455
// PORT: Go `strings.NewReplacer` with one-byte old strings (Go picks its byte
// replacer). The pairs are kept in Go order; `extra_escape_replacer_replace`
// is `Replace`.
const EXTRA_ESCAPE_REPLACER: [(u8, &str); 19] = [
    (b':', "%3A"),
    (b'/', "%2F"),
    (b'?', "%3F"),
    (b'#', "%23"),
    (b'[', "%5B"),
    (b']', "%5D"),
    (b'@', "%40"),
    //
    (b'!', "%21"),
    (b'$', "%24"),
    (b'&', "%26"),
    (b'\'', "%27"),
    (b'(', "%28"),
    (b')', "%29"),
    (b'*', "%2A"),
    (b'+', "%2B"),
    (b',', "%2C"),
    (b';', "%3B"),
    (b'=', "%3D"),
    //
    (b' ', "%20"),
];

/// Go `extraEscapeReplacer.Replace(s)`.
fn extra_escape_replacer_replace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        let replacement = if ch.is_ascii() {
            EXTRA_ESCAPE_REPLACER
                .iter()
                .find(|(old, _)| *old == ch as u8)
                .map(|(_, new)| *new)
        } else {
            None
        };
        match replacement {
            Some(new) => out.push_str(new),
            None => out.push(ch),
        }
    }
    out
}

// Go: ls/lsconv/converters.go:332 FileNameToDocumentURI

/// Go `utf16.RuneLen(r)`.
// Go: unicode/utf16/utf16.go RuneLen
fn utf16_rune_len(r: i32) -> i32 {
    if (0..0xD800).contains(&r) || (0xE000..0x10000).contains(&r) {
        1
    } else if (0x10000..=0x10FFFF).contains(&r) {
        2
    } else {
        -1
    }
}

// PORT (WTF-8): Go text may hold invalid UTF-8. Go decodes each invalid byte
// as a one-byte `RuneError`, which counts as one UTF-16 unit. The port holds
// such text in the port form (see `scanner_util::GO_STRING_MARKER`): each
// invalid byte (also each byte of a WTF-8 lone surrogate in source text) is
// a 7-byte unit, and a real U+FDD0 is a 6-byte unit.
// `utf8_decode_rune_in_string` reads a unit as one rune (`RuneError` for an
// invalid byte) with its size in the port text, so UTF-16 characters match
// Go. Byte offsets after a unit are port offsets, not Go offsets, so the
// UTF-8 position encoding converts a character to and from Go bytes in its
// line (`port_byte_offset`, `go_byte_offset`). Otherwise the decoder follows
// Go `utf8.DecodeRuneInString`, also at a position inside a character.
impl Converters {
    // Go: ls/lsconv/converters.go:366 lineAndCharacterToPosition
    // PORT: Go passes the position by value; here by reference.
    // Private as in Go since tsgo#4712: the LS uses `from_lsp_position`,
    // `from_lsp_range` and their SourceFile forms.

    // Go: ls/lsconv/converters.go:417 positionToLineAndCharacter
    // Private as in Go since tsgo#4712: the LS uses `to_lsp_position`.
}

// Go: ls/lsconv/converters.go:451 diagnosticOptions
struct DiagnosticOptions {
    report_style_checks_as_warnings: bool,
    related_information: bool,
    tag_value_set: Vec<lsproto::DiagnosticTag>,
    visual_studio: bool,
}

// Go: ls/lsconv/converters.go:459 DiagnosticToLSPPull
// DiagnosticToLSPPull converts a diagnostic for pull diagnostics (textDocument/diagnostic)

// Go: ls/lsconv/converters.go:471 DiagnosticToLSPPush
// DiagnosticToLSPPush converts a diagnostic for push diagnostics (textDocument/publishDiagnostics)

// Go: ls/lsconv/converters.go:482 styleCheckDiagnostics
// https://github.com/microsoft/vscode/blob/93e08afe0469712706ca4e268f778cfadf1a43ef/extensions/typescript-language-features/src/typeScriptServiceClientHost.ts#L40C7-L40C29

// Go: ls/lsconv/converters.go:493 diagnosticToLSP

// Go: ls/lsconv/converters.go:577 diagnosticScriptAndRange
// diagnosticScriptAndRange resolves the text basis and range to report a diagnostic against. For a
// content-mapped file it maps the diagnostic's virtual range back to the original text so
// the range lines up with what the editor shows; the original text's line map is already what
// getLineMap returns for the file. A range in synthesized code has no original counterpart, so it is
// surfaced at the top of the file. Non-mapped files are returned unchanged.
// PORT: Go returns the file itself as the `Script`; here a reference to the
// caller's file root `Node`. A nil file is `Node::NIL`, which Go also
// returns as is.

// Go: ls/lsconv/converters.go:608 diagnosticSeverity
// diagnosticSeverity maps a diagnostic category to its LSP severity.

// Go: ls/lsconv/converters.go:621 messageChainToString

// Go: ls/lsconv/converters.go:630 ptrToSliceIfNonEmpty
fn ptr_to_slice_if_non_empty<T>(s: Vec<T>) -> Option<Vec<T>> {
    if s.is_empty() {
        return None;
    }
    Some(s)
}

// Go: diagnosticwriter/diagnosticwriter.go:354 WriteFlattenedASTDiagnosticMessage
// PORT: Go package `diagnosticwriter`. The `String` writer version in
// program.rs is private, and the execute/tsc version writes to its own
// `Writer`, so the three Go functions are ported here for a `String`.
// Only English messages exist; `localize` drops the locale.

// Go: diagnosticwriter/diagnosticwriter.go:358 WriteFlattenedDiagnosticMessage

// Go: diagnosticwriter/diagnosticwriter.go:366 flattenDiagnosticMessageChain

// Go: ls/lsconv/converters_test.go (tsgo#4712)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ContentMapperSourceFileInfo;
    use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
    use crate::spanmap::{Kind, Segment};
    use std::sync::Arc;

    // Go: ls/lsconv/converters_test.go:121 TestConvertersSourceFileProjectionExpansion
    // PORT: Go links the two `*ast.SourceFile` values by pointer; here the
    // parsed files are `Rc`, as `set_content_mapper_info` takes them.
    #[test]
    fn test_converters_source_file_projection_expansion() {
        let original = "x";
        let parse_options = SourceFileParseOptions {
            file_name: "/component.vue".to_string(),
            path: tspath::Path("/component.vue".to_string()),
            ..Default::default()
        };
        let canonical = Rc::new(parse_source_file(&parse_options, " x", ScriptKind::TS));
        let mut supplemental_options = parse_options;
        supplemental_options.path = tspath::Path("/component.vue::supplemental".to_string());
        let supplemental = Rc::new(parse_source_file(
            &supplemental_options,
            "  x",
            ScriptKind::TS,
        ));
        canonical.set_content_mapper_info(ContentMapperSourceFileInfo {
            original_text: original.to_string(),
            content_mapper: "mapper".to_string(),
            span_map: Some(Arc::new(spanmap::new(&[Segment {
                virtual_start: 1,
                virtual_end: 2,
                original_end: 1,
                kind: Kind::VERBATIM,
                features: Feature::ALL,
                ..Default::default()
            }]))),
            supplemental_source_files: vec![Rc::clone(&supplemental)],
            ..Default::default()
        });
        supplemental.set_content_mapper_info(ContentMapperSourceFileInfo {
            original_text: original.to_string(),
            content_mapper: "mapper".to_string(),
            span_map: Some(Arc::new(spanmap::new(&[Segment {
                virtual_start: 2,
                virtual_end: 3,
                original_end: 1,
                kind: Kind::VERBATIM,
                features: Feature::ALL,
                ..Default::default()
            }]))),
            canonical_source_file: Some(Rc::clone(&canonical)),
            ..Default::default()
        });
        let line_map = compute_lsp_line_starts(original);
        let converters = new_converters(lsproto::PositionEncodingKind::UTF16, move |_| {
            Some(Rc::clone(&line_map))
        });

        let positions = from_lsp_position_for_source_file(
            &converters,
            canonical.root,
            lsproto::Position::default(),
            Feature::HOVER,
        );
        assert_eq!(positions.len(), 2);
        let projected_file: Node = positions[0].script;
        assert!(projected_file == canonical.root);
        assert!(positions[0].script == canonical.root);
        assert_eq!(positions[0].position, 1);
        assert!(positions[1].script == supplemental.root);
        assert_eq!(positions[1].position, 2);
    }

    /// A script that is the port form of some Go bytes.
    #[derive(Clone)]
    struct PortScript(String);

    impl Script for PortScript {
        fn file_name(&self) -> &str {
            "/a.ts"
        }
        fn text(&self) -> ScriptText<'_> {
            ScriptText::Borrowed(&self.0)
        }
    }

    // With the UTF-8 position encoding a character is a Go byte column (Go
    // `converters.go:393` `start+char` and `:437` `position - start`). The
    // port form holds an invalid byte as 7 bytes and a real U+FDD0 as 6, so
    // utf8cut `repro_latin1_lsp.py` gave column 25 where Go gives 19. Each
    // Go offset converts to its column and back, and a column past the line
    // end stops at the line end.
    #[test]
    fn utf8_columns_count_go_bytes() {
        let go: &[u8] = b"a\xACb\xEF\xB7\x90c\nd\xED\xA0\x80e \xE2\x82\xACf";
        let script = PortScript(crate::scanner_util::go_string_from_bytes(go.to_vec()));
        let line_map = compute_lsp_line_starts(&script.0);
        let converters = new_converters(lsproto::PositionEncodingKind::UTF8, move |_| {
            Some(Rc::clone(&line_map))
        });
        let go_line_start = |g: usize| if g >= 8 { 8 } else { 0 };
        for g in 0..=go.len() {
            let port = port_byte_offset(&script.0, g as i32);
            let lc = lsproto::Position {
                line: u32::from(g >= 8),
                character: (g - go_line_start(g)) as u32,
            };
            let (got, _) = converters.to_lsp_position(&script, port);
            assert_eq!(got, lc, "Go offset {g} (port {port})");
            let back = from_lsp_position(&converters, script.clone(), lc, Feature::ALL);
            assert_eq!(back[0].position, port, "{lc:?}");
        }
        let past = lsproto::Position {
            line: 0,
            character: 40,
        };
        let back = from_lsp_position(&converters, script.clone(), past, Feature::ALL);
        assert_eq!(back[0].position, port_byte_offset(&script.0, 8));
    }

    // R151 reviewer: UTF-8 columns scanned each line for markers in any
    // non-ASCII text. A text with no marker unit skips the scan
    // (`LSPLineMap::has_marker`) and gives the same columns.
    #[test]
    fn utf8_columns_without_marker_skip_the_scan() {
        let script = PortScript("é = 1;\n€x = \"\u{1F600}\";".to_string());
        let line_map = compute_lsp_line_starts(&script.0);
        assert!(!line_map.ascii_only && !line_map.has_marker);
        let converters = new_converters(lsproto::PositionEncodingKind::UTF8, move |_| {
            Some(Rc::clone(&line_map))
        });
        for pos in 0..=script.0.len() as i32 {
            let line = i32::from(pos >= 8);
            let character = pos - [0, 8][line as usize];
            let lc = lsproto::Position {
                line: line as u32,
                character: character as u32,
            };
            assert_eq!(converters.to_lsp_position(&script, pos).0, lc, "{pos}");
            let back = from_lsp_position(&converters, script.clone(), lc, Feature::ALL);
            assert_eq!(back[0].position, pos, "{lc:?}");
        }

        // The skip itself (followups4): the text of `marked` has a 7-byte
        // unit at the start of each line, and the line map comes from
        // `plain`, which has the same line starts and no marker. A converter
        // that trusts `has_marker` gives the byte columns of `plain`. One
        // that scans the text counts each unit as one Go byte.
        let plain = "éééa = 1;\néééax = 2;";
        let marked = PortScript(crate::scanner_util::go_string_from_bytes(
            b"\xFF = 1;\n\xFFx = 2;".to_vec(),
        ));
        assert_eq!(marked.0.len(), plain.len());
        let line_map = compute_lsp_line_starts(plain);
        assert!(!line_map.ascii_only && !line_map.has_marker);
        assert!(compute_lsp_line_starts(&marked.0).has_marker);
        let converters = new_converters(lsproto::PositionEncodingKind::UTF8, move |_| {
            Some(Rc::clone(&line_map))
        });
        for pos in [0, 7, 12, 13, 20, 26] {
            let line = i32::from(pos >= 13);
            let lc = lsproto::Position {
                line: line as u32,
                character: (pos - [0, 13][line as usize]) as u32,
            };
            assert_eq!(converters.to_lsp_position(&marked, pos).0, lc, "{pos}");
            let back = from_lsp_position(&converters, marked.clone(), lc, Feature::ALL);
            assert_eq!(back[0].position, pos, "{lc:?}");
        }
    }

    // `GoOffsets` gives `go_byte_offset` and `port_byte_offset` of every
    // offset of a text after one scan (code lenses, references).
    #[test]
    fn go_offsets_match_the_scans() {
        use crate::scanner_util::{GoOffsets, go_string_from_bytes};
        let lone = format!(
            "a{}\u{10F83D}b\u{FDD0}\u{FDD0}c",
            crate::scanner_util::GO_STRING_MARKER
        );
        let texts = [
            go_string_from_bytes(b"a\xACb\xEF\xB7\x90c\nd\xED\xA0\x80e \xE2\x82\xACf".to_vec()),
            go_string_from_bytes(b"\xFF\xFE\xEF\xB7\x90".to_vec()),
            "plain é text".to_string(),
            lone,
        ];
        for text in &texts {
            let offsets = GoOffsets::new(text);
            assert_eq!(offsets.has_units(), text.contains('\u{FDD0}'), "{text:?}");
            for pos in -2..=text.len() as i32 + 3 {
                assert_eq!(
                    offsets.go_offset(pos),
                    go_byte_offset(text, pos),
                    "{text:?} {pos}"
                );
                assert_eq!(
                    offsets.port_offset(pos),
                    port_byte_offset(text, pos),
                    "{text:?} Go {pos}"
                );
            }
        }
    }
}
