//! Go: execute/tsc/init.go (`tsc --init`).

use crate::prelude::*;

use super::compile::{System, write_str};
use super::help::{};
use crate::frontend::tsoptions::{
    
};
use crate::frontend::tspath::{};
use crate::locale::Locale;

// Go: execute/tsc/init.go:19 WriteConfigFile
// PORT: Go `options` is the command line `Raw` map
// (`*collections.OrderedMap[string, any]`); here it is the `IndexMap` in
// `CompilerOptionsValue::Map`.

// Go: execute/tsc/init.go:34 tab
const TAB: &str = "  ";

// Go: execute/tsc/init.go:107 commented
// commentedNever': Never comment this out
// commentedAlways': Always comment this out, even if it's on commandline
// commentedOptional': Comment out unless it's on commandline
// PORT: a Go `int` with three constants is a Rust enum, so the Go
// "invalid `commented`" panic in `emitOption` cannot happen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Commented {
    Never,
    Always,
    Optional,
}

// PORT: Go `generateTSConfig` keeps `result` and `allSetOptions` in local
// variables and changes them from closures (`emitHeader`, `newline`,
// `push`, `emitOption`). Here they are fields of this struct, and the
// closures are its methods.


// Go: execute/tsc/init.go:33 generateTSConfig

// Go: execute/tsc/init.go:54 formatSingleValue
// PORT: Go `value == v` compares two `any` values by dynamic type and
// value. `CompilerOptionsValue` equality compares the variant and the value,
// and the command line parser stores the enum map value itself
// (`convert_json_option_of_enum_type`), so the same keys match.

// Go: execute/tsc/init.go:76 formatValueOrArray
// PORT: Go tests `reflect.Kind() == reflect.Slice`. The slice values here
// are `List` and `NilList` (Go `[]any`: a parsed list option or the `types`
// default) and `StringList` (Go `[]string`: the parser's value for a list
// option with no argument).
