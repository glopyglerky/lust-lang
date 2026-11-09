//! Go: execute/tsc/diagnostics.go (diagnostic, error summary and status
//! reporters), with the `diagnosticwriter` pieces they call. The help and
//! version printers (execute/tsc/help.go) are in help.rs.
//!
//! PORT: Go `diagnosticwriter.Diagnostic` is an interface. Every Go caller
//! of these writers passes an `*ast.Diagnostic` wrapped in
//! `diagnosticwriter.ASTDiagnostic` (`WrapASTDiagnostic`,
//! `FromASTDiagnostics`). The writers here take the `Diagnostic` and wrap it
//! themselves (`AstDiagnostic`), so a content-mapped file (tsgo#4712) is
//! shown as Go shows it.

use crate::prelude::*;

use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use std::time::SystemTime;

use super::compile::{};
// PORT: testing (the status reporters)
use crate::diagnostics::Category;
use crate::frontend::tspath::{ComparePathsOptions, };
use crate::locale::Locale;

// Go: diagnosticwriter/diagnosticwriter.go:196 FormattingOptions
#[derive(Clone, Debug, Default)]
pub struct FormattingOptions {
    pub new_line: String,
    pub compare_paths_options: ComparePathsOptions,
    pub locale: Locale,
}

// Go: execute/tsc/diagnostics.go:14 getFormatOptsOfSys

// Go: execute/tsc/diagnostics.go:23 DiagnosticReporter

// Go: execute/tsc/diagnostics.go:25 QuietDiagnosticReporter

// Go: execute/tsc/diagnostics.go:27 CreateDiagnosticReporter

// Go: execute/tsc/diagnostics.go:43 defaultIsPretty (ts#63941)

// Go: execute/tsc/diagnostics.go:61 shouldBePretty

// Go: execute/tsc/diagnostics.go:68 colors
#[derive(Clone, Debug, Default)]
pub struct Colors {
    show_colors: bool,

    is_windows: bool,
    is_windows_terminal: bool,
    is_vs_code: bool,
    supports_richer_colors: bool,
}

// Go: execute/tsc/diagnostics.go:77 createColors (ts#63941)

impl Colors {
    // Go: execute/tsc/diagnostics.go:93 (*colors).bold
    pub fn bold(&self, str: &str) -> String {
        if !self.show_colors {
            return str.to_string();
        }
        format!("\x1b[1m{str}\x1b[22m")
    }

    // Go: execute/tsc/diagnostics.go:100 (*colors).blue
    pub fn blue(&self, str: &str) -> String {
        if !self.show_colors {
            return str.to_string();
        }

        // Effectively Powershell and Command prompt users use cyan instead
        // of blue because the default theme doesn't show blue with enough contrast.
        if self.is_windows && !self.is_windows_terminal && !self.is_vs_code {
            return self.bright_white(str);
        }
        format!("\x1b[94m{str}\x1b[39m")
    }

    // Go: execute/tsc/diagnostics.go:113 (*colors).blueBackground
    pub fn blue_background(&self, str: &str) -> String {
        if !self.show_colors {
            return str.to_string();
        }
        if self.supports_richer_colors {
            format!("\x1B[48;5;68m{str}\x1B[39;49m")
        } else {
            format!("\x1b[44m{str}\x1B[39;49m")
        }
    }

    // Go: execute/tsc/diagnostics.go:124 (*colors).brightWhite
    pub fn bright_white(&self, str: &str) -> String {
        if !self.show_colors {
            return str.to_string();
        }
        format!("\x1b[97m{str}\x1b[39m")
    }
}

// Go: execute/tsc/diagnostics.go:136 DiagnosticsReporter

// Go: execute/tsc/diagnostics.go:138 QuietDiagnosticsReporter

// Go: execute/tsc/diagnostics.go:140 CreateReportErrorSummary
// PORT: Go reads `sys.Writer()` on each report. The reporter cannot keep
// `sys`, so it reads the writer when it is made. A system's writer does
// not change after the system is made.

// Go: execute/tsc/diagnostics.go:150 CreateBuilderStatusReporter
// PORT: Go `options` can be nil only through `shouldBePretty`; the quiet
// check reads it, so it is required here.

// Go: execute/tsc/diagnostics.go:168 CreateWatchStatusReporter


// ---------------------------------------------------------------------------
// Go time.Local (go1.27.1 time/zoneinfo_unix.go and zoneinfo_windows.go),
// for the status clock
// ---------------------------------------------------------------------------
// PORT: Go `time.Location` is a `jiff::tz::TimeZone`. On Unix, Go parses
// zone files with `LoadLocationFromTZData`; jiff parses the same TZif data
// (the transitions and the POSIX TZ footer) with `TimeZone::tzif`. Windows
// has its own `init_local` (below).

// Go: time/zoneinfo.go:88 localLoc, :89 localOnce, :91 (*Location).get


// Go: time/zoneinfo_unix.go:21 platformZoneSources
// Many systems use /usr/share/zoneinfo, Solaris 2 has
// /usr/share/lib/zoneinfo, IRIX 6 has /usr/lib/locale/TZ,
// NixOS has /etc/zoneinfo.
const PLATFORM_ZONE_SOURCES: &[&str] = &[
    "/usr/share/zoneinfo/",
    "/usr/share/lib/zoneinfo/",
    "/usr/lib/locale/TZ/",
    "/etc/zoneinfo",
];

// Go: time/zoneinfo_unix.go:28 initLocal
// PORT: Go `syscall.Getenv` gives the raw bytes of the value; so does
// `as_encoded_bytes` on Unix.

// Go: time/zoneinfo_windows.go:230 initLocal
// PORT: Go calls `GetTimeZoneInformation` and makes the zone from the
// Windows standard and daylight rules. jiff asks Windows for the zone key
// name (`GetDynamicTimeZoneInformation`), maps it to an IANA name (CLDR
// windowsZones) and takes that zone from the zone data in the binary. For a
// mapped zone, both give the same civil time now. On failure both fall back
// to UTC.
// DIVERGES: jiff reads `TZ` first; Go on Windows does not read it. The
// logger test `log_time_is_local_time` sets `TZ`, so on Windows it needs
// this. A Windows zone with no IANA name in jiff's table gives UTC, and jiff
// does not read the Windows setting that turns off daylight saving time.
// Go's exact path needs `unsafe` FFI, which the workspace forbids.
#[cfg(windows)]
fn init_local() -> jiff::tz::TimeZone {
    jiff::tz::TimeZone::try_system().unwrap_or(jiff::tz::TimeZone::UTC)
}

// Go: time/zoneinfo_read.go:531 loadLocation
// PORT: `initLocal` reads only whether it failed, so the Go first-error
// bookkeeping is dropped. After the sources Go tries the embedded
// `time/tzdata`, which tsgo does not import, and
// `runtime.GOROOT()/lib/time/zoneinfo.zip`. The port has no Go root, so
// that zip is not ported. It changes the result only for a zone name that
// no system source has.

// Go: time/zoneinfo_read.go:520 loadTzinfo
// Go: time/zoneinfo_read.go:367 loadTzinfoFromDirOrZip
// PORT: Go reads a source that ends in "tzdata" (Android) or ".zip" as an
// archive. No source in `initLocal` does, so only the directory case is
// ported.

// Go: time/zoneinfo_read.go:37 maxFileSize
const MAX_FILE_SIZE: usize = 10 << 20;

// Go: time/zoneinfo_read.go:575 readFile
// PORT: the path is raw bytes, as in Go. Go stops reading past
// `maxFileSize` and fails; this reads the file and then checks the size.
// Off unix a path must be UTF-8; another path fails, as a missing file.

/// The wasm build reads no zone file: its hosts (npm/wasm/core.js) give the
/// module no preopened directory (`fd_prestat_get`), so `std::fs` cannot
/// open one, and `time.Local` is UTC, as in Go when no zone file is found.
/// This leaves the zone file parser out of the wasm module.
// PORT: not in Go.
#[cfg(target_family = "wasm")]
fn read_file(_name: &[u8]) -> Option<Vec<u8>> {
    None
}

// ---------------------------------------------------------------------------
// Go diagnosticwriter/diagnosticwriter.go (the pieces the reporters call)
// ---------------------------------------------------------------------------

// Go: diagnosticwriter/diagnosticwriter.go:21 FileLike








// Go: diagnosticwriter/diagnosticwriter.go:44 ASTDiagnostic
// ASTDiagnostic wraps ast.Diagnostic to implement the Diagnostic interface
// PORT: Go reads `Code`, `Category` and `Localize` through the embedded
// `*ast.Diagnostic`; the port reads them from `.0`. The api lane calls it
// (ts#63935 `NewDiagnosticResponse`).


// Go: diagnosticwriter/diagnosticwriter.go:164 WrapASTDiagnostic (ts#63935)

// Go: diagnosticwriter/diagnosticwriter.go:87 resolvedLocation (tsgo#4712)
// resolvedLocation describes how a diagnostic on a content-mapped file should be reported.
#[derive(Clone, Copy)]
struct ResolvedLocation {
    loc: TextRange,
    use_original: bool, // render against the file's original, untransformed text
    synthesized: bool,  // the range is in virtual code with no corresponding original location
}

// Go: diagnosticwriter/diagnosticwriter.go:118 originalTextFile (tsgo#4712)

// Go: diagnosticwriter/diagnosticwriter.go:124 newOriginalTextFile (tsgo#4712, ts#63936)

// Go: diagnosticwriter/diagnosticwriter.go:137 renamedFile (ts#63936)
/// Go `renamedFile`: a source file shown under another name (the name of
/// its canonical source file); its text and line map are the file's own.
pub struct RenamedFile {
    file: Node,
    file_name: &'static str,
}

// Go: scanner/scanner.go:2676 GetECMALineOfPosition
// PORT: Go takes an `ast.SourceFileLike`. The Rust scanner function takes a
// file node, so this is its code on a `FileLike`.

// Go: scanner/scanner.go:2684 GetECMALineAndUTF16CharacterOfPosition
// PORT: Go takes an `ast.SourceFileLike`. The code is
// `scanner_util::ecma_line_and_utf16_character_of_text_position` (a `pos`
// inside a char, and the Go panics for a `pos` out of the text) on a
// `FileLike`.

// Go: diagnosticwriter/diagnosticwriter.go:203 foregroundColorEscapeGrey
const FOREGROUND_COLOR_ESCAPE_GREY: &str = "\u{1b}[90m";
// Go: diagnosticwriter/diagnosticwriter.go:204 foregroundColorEscapeRed
const FOREGROUND_COLOR_ESCAPE_RED: &str = "\u{1b}[91m";
// Go: diagnosticwriter/diagnosticwriter.go:205 foregroundColorEscapeYellow
const FOREGROUND_COLOR_ESCAPE_YELLOW: &str = "\u{1b}[93m";
// Go: diagnosticwriter/diagnosticwriter.go:206 foregroundColorEscapeBlue
const FOREGROUND_COLOR_ESCAPE_BLUE: &str = "\u{1b}[94m";
// Go: diagnosticwriter/diagnosticwriter.go:207 foregroundColorEscapeCyan
const FOREGROUND_COLOR_ESCAPE_CYAN: &str = "\u{1b}[96m";

// Go: diagnosticwriter/diagnosticwriter.go:211 gutterStyleSequence
const GUTTER_STYLE_SEQUENCE: &str = "\u{1b}[7m";
// Go: diagnosticwriter/diagnosticwriter.go:212 gutterSeparator
const GUTTER_SEPARATOR: &str = " ";
// Go: diagnosticwriter/diagnosticwriter.go:213 resetEscapeSequence
const RESET_ESCAPE_SEQUENCE: &str = "\u{1b}[0m";
// Go: diagnosticwriter/diagnosticwriter.go:214 ellipsis
const ELLIPSIS: &str = "...";

// Go: diagnosticwriter/diagnosticwriter.go:229 FormatDiagnosticWithColorAndContext

// Go: diagnosticwriter/diagnosticwriter.go:264 writeCodeSnippet

// Go: diagnosticwriter/diagnosticwriter.go:358 WriteFlattenedDiagnosticMessage
// PORT: also Go `WriteFlattenedASTDiagnosticMessage` (:338): the port takes
// the ast diagnostic and wraps it.

// Go: diagnosticwriter/diagnosticwriter.go:366 flattenDiagnosticMessageChain

// Go: diagnosticwriter/diagnosticwriter.go:380 diagnosticPrefix (tsgo#4712)
// diagnosticPrefix returns the prefix shown before a diagnostic's code, e.g. "TS" for compiler
// diagnostics or a content mapper's custom source for its diagnostics.

// Go: diagnosticwriter/diagnosticwriter.go:387 getCategoryFormat

// Go: diagnosticwriter/diagnosticwriter.go:401 FormattedWriter

// Go: diagnosticwriter/diagnosticwriter.go:403 writeWithStyleAndReset

// Go: diagnosticwriter/diagnosticwriter.go:409 WriteLocation
// PORT: `file` is a `FileLike` or a source file node.

// Some of these lived in watch.ts, but they're not specific to the watch API.

// Go: diagnosticwriter/diagnosticwriter.go:427 ErrorSummary
// PORT: the lists borrow the diagnostics.

// Go: diagnosticwriter/diagnosticwriter.go:434 WriteErrorSummaryText

// Go: diagnosticwriter/diagnosticwriter.go:481 getErrorSummary
// PORT: Go calls `File()` twice for each diagnostic. For a content-mapped
// file each call makes a new `originalTextFile`, so each such diagnostic has
// its own entry (tsgo#4712). One call per diagnostic gives the same entries.

// Go: diagnosticwriter/diagnosticwriter.go:516 writeTabularErrorsDisplay

// Go: diagnosticwriter/diagnosticwriter.go:547 prettyPathForFileError

// Go: diagnosticwriter/diagnosticwriter.go:571 WriteFormatDiagnostic

// Go: diagnosticwriter/diagnosticwriter.go:565 WriteFormatDiagnostics

// Go: diagnosticwriter/diagnosticwriter.go:584 FormatDiagnosticsStatusWithColorAndTime

// Go: diagnosticwriter/diagnosticwriter.go:591 FormatDiagnosticsStatusAndTime

// Go: diagnosticwriter/diagnosticwriter.go:601 TryClearScreen

// ---------------------------------------------------------------------------
// Go standard library helpers used above and in help.rs
// ---------------------------------------------------------------------------


// Go: strings/strings.go:597 Repeat (go1.27.1)
/// Go `strings.Repeat`, which panics on a negative count.
// PORT: the Go panic ends the run with Go's exit code 2 (`core::go_panic`),
// not the port crash code 70. `--pretty` reaches it in Go too
// (diagnosticwriter.go:335), when a squiggle ends before it starts.
pub(super) fn go_repeat(s: &str, count: i32) -> String {
    match usize::try_from(count) {
        Ok(count) => s.repeat(count),
        Err(_) => crate::core::go_panic("strings: negative Repeat count".to_string()),
    }
}

#[cfg(test)]
mod tests {
    /// Go `strings.Repeat` panics on a negative count. `--pretty` reaches it
    /// when a squiggle ends before it starts (diagnosticwriter.go:335), and
    /// Go ends the run with exit code 2. It is a Go panic, not a port gap
    /// (exit 70).
    #[test]
    fn go_repeat_with_a_negative_count_is_a_go_panic() {
        assert_eq!(super::go_repeat("~", 2), "~~");
        let payload = std::panic::catch_unwind(|| super::go_repeat("~", -1))
            .expect_err("a negative count panics");
        let panic = payload
            .downcast_ref::<crate::core::GoPanic>()
            .expect("a Go panic");
        assert_eq!(panic.message, "strings: negative Repeat count");
    }
}
