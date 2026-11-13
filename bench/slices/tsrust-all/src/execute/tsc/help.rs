//! Go: execute/tsc/help.go (`--version`, `--help`, `--help --all` and
//! `--build --help` output).
//!
//! PORT: Go builds the output as a `[]string` of chunks and writes each one
//! with `fmt.Fprint`; the port does the same with `Vec<String>` and
//! `write_str`. Each Go `Message.Localize(locale, ...)` is
//! `message_localize`.

use crate::prelude::*;

use super::compile::{};
use super::diagnostics::{Colors, go_repeat};
use crate::frontend::tsoptions::{
    CommandLineOptionKind, 
    
};
use crate::locale::Locale;

// Go: execute/tsc/help.go:15 PrintVersion

// Go: execute/tsc/help.go:19 PrintHelp

// Go: execute/tsc/help.go:27 getOptionsForHelp
// PORT: Go `slices.SortFunc` is pdqsort, which is not stable. The input is
// the fixed declaration list, and its only names that are equal in lower
// case are the two `help` options (`-h`, then `-?`). Go keeps them in that
// order (checked against the oracle `--help --all` output).
// `gostd::slices::sort_func` is the same pdqsort.

// Go: execute/tsc/help.go:44 getHeader
// PORT: Go `getHeader` is unexported; init.rs uses it too.

// Go: execute/tsc/help.go:67 printEasyHelp

// Go: execute/tsc/help.go:110 printAllHelp

// Go: execute/tsc/help.go:135 PrintBuildHelp

// Go: execute/tsc/help.go:149 generateSectionOptionsOutput

// Go: execute/tsc/help.go:194 generateGroupOptionOutput

// Go: execute/tsc/help.go:222 generateOptionOutput

// Go: execute/tsc/help.go:295 formatDefaultValue
// PORT: Go `option` is a pointer that can be nil (`Elements()` of a list
// without elements); Go dereferences it after the nil value check.



// Go: execute/tsc/help.go:313 valueCandidate
struct ValueCandidate {
    // "one or more" or "any of"
    value_type: String,
    possible_values: String,
}

// Go: execute/tsc/help.go:319 showAdditionalInfoOutput

// Go: execute/tsc/help.go:332 getValueCandidate
// PORT: Go takes `sys` and does not use it.

// Go: execute/tsc/help.go:366 getPossibleValues

// Go: execute/tsc/help.go:397 getPrettyOutput
// PORT: Go cuts `right` at a byte index, so a cut inside a UTF-8 sequence
// writes invalid UTF-8. The loop cuts the Go bytes of `right` and keeps
// each piece in port form (see `scanner_util::GO_STRING_MARKER`). The
// process output writes the Go bytes back (`compile::GoOutput`).

// Go: execute/tsc/help.go:424 getDisplayNameTextOfOption
