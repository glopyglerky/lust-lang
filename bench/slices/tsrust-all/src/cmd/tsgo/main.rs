//! Go `cmd/tsc/main.go`, and the parts of the Go standard library that
//! package main needs and the crate does not have yet: `flag` (bool, int
//! and string flags, `Parse`, the default usage text) and
//! `signal.NotifyContext`.

use crate::cmd::tsgo::prelude::*;

use crate::gostd::context::{self, };
use crate::gostd::errors;
use std::cell::Cell;
use std::sync::{Arc, LazyLock};

// Go: cmd/tsc/main.go:14 main
// PORT: `bin/goport.rs` `main` calls `run_main` before its own compile
// path and exits with the status it returns.

// Go: cmd/tsc/main.go:18 runMain
// PORT: `args` is Go `osutil.Args()[1:]`. `None` means Go continues with
// `execute.CommandLine`; goport continues with its own compile path, which
// replaces it. Go `core.ApplyDebugStackLimit()` (the TS_GO_DEBUG_STACK_LIMIT
// override) becomes the stack of the thread that runs the command
// (`gostd::stack::max_stack_size`).
// PORT: Go `signal.NotifyContext` before `execute.CommandLine` is in
// `bin/tsgo.rs`, which runs that path; the goport compile path has none.

// PORT: runs `f` on a new thread with the Go maximum stack
// (`gostd::stack::max_stack_size`) and returns its status. Go runs it on
// the main goroutine. A thread that cannot start ends the process as the
// Go runtime does (`GoThread`).
// A panic that reaches the top of that thread ends Go with a crash. A Go
// panic (`core::go_panic`) ends it as the Go runtime does, as `bin/tsgo.rs`
// does: `panic: <message>` on stderr and `EXIT_GO_PANIC` (2). Any other
// panic is a port gap: goport returns `EXIT_UNPORTED` (70), as
// `bin/goport.rs` does for a failed worker.

// Go: os/signal/signal.go:293 NotifyContext (go1.27.1)
// NotifyContext returns a copy of the parent context that is marked done
// (its Done channel is closed) when one of the listed signals arrives,
// when the returned stop function is called, or when the parent context's
// Done channel is closed, whichever happens first.
// PORT: the signals are fixed. Every tsgo caller passes SIGINT
// (`os.Interrupt`) and SIGTERM. Go `Notify(c.ch, ...)` is a signal-hook
// `Signals`, and the goroutine is a thread that waits on it. Closing its
// handle ends the wait: that is the `<-c.Done()` case of the `select`.
// PORT: after Go `Stop(c.ch)`, a later SIGINT or SIGTERM kills the process
// (the Go runtime default). The signal-hook handler stays installed, so
// the port ignores such a signal. Every caller returns right after `stop`.

// PORT: off unix, signal-hook has no `Signals` iterator, so no signal
// cancels the context: Ctrl+C ends the process (the OS default) where Go
// cancels it. `stop` cancels it, as in Go. Not run on such a target.
#[cfg(not(unix))]
pub fn notify_context(parent: &Context) -> (Context, CancelFunc) {
    let (ctx, cancel) = context::with_cancel_cause(parent);
    let stop: CancelFunc = Arc::new(move || cancel(None));
    (ctx, stop)
}

// Go: os/signal/signal.go:352 signalError
// PORT: Go `Is(target error) bool` (true for `context.Canceled`) has no
// port form. No port code reads the cause.
#[cfg(not(target_family = "wasm"))]
#[derive(Debug, PartialEq)]
struct SignalError(String);

#[cfg(not(target_family = "wasm"))]
impl std::fmt::Display for SignalError {
    // Go: signal.go:354 signalError.Error
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// Go: syscall/syscall_unix.go Signal.String, with the Linux `signalList`
// names of the two signals that `notify_context` registers.
#[cfg(not(target_family = "wasm"))]
fn signal_string(s: i32) -> String {
    match s {
        SIGINT => "interrupt".to_string(),
        SIGTERM => "terminated".to_string(),
        _ => format!("signal {s}"),
    }
}

// Go: core.Must(os.Getwd()), for the LSP and the API.
// PORT: Go `os.Getwd` is `frontend::vfs::os_current_dir`, the same one the
// compile path reads (a symlinked cwd keeps the `$PWD` link path; the value
// is in the port form of the Go bytes). Go `core.Must` panics with the
// error value, so the run ends with `panic: <err.Error()>` and exit 2
// (`core::go_panic`, `getwd_error_text`).

// Go: flag/flag.go (go1.27.1), the part that package main uses.

// Go: flag.go:101 ErrHelp
// ErrHelp is the error returned if the -help or -h flag is invoked
// but no such flag is defined.

// Go: flag.go:105 errParse
// errParse is returned by Set if a flag's value fails to parse, such as with an invalid integer for Int.
// It then gets wrapped through failf to provide more information.

// Go: flag.go:109 errRange
// errRange is returned by Set if a flag's value is out of range.
// It then gets wrapped through failf to provide more information.

// Go: flag.go ErrorHandling
// ErrorHandling defines how [FlagSet.Parse] behaves if the parse fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorHandling {
    ContinueOnError, // Return a descriptive error.
    ExitOnError,     // Call os.Exit(2) or for -h/-help Exit(0).
    PanicOnError,    // Call panic with a descriptive error.
}

// Go: flag.go boolValue, intValue and stringValue, the `Value` kinds
// package main defines. The value lives behind the pointer that `Bool`,
// `Int` and `String` return.
// PORT: Go `int` is 64 bits on the release targets (`strconv.IntSize`), so
// the int flag is `i64`.
#[derive(Clone)]
pub enum FlagValue {
    Bool(Rc<Cell<bool>>),
    Int(Rc<Cell<i64>>),
    String(Rc<RefCell<String>>),
}

impl FlagValue {
    // Go: flag.go:133 boolValue.Set, intValue.Set and stringValue.Set

    // Go: boolValue.String (strconv.FormatBool), intValue.String
    // (strconv.Itoa) and stringValue.String
    pub fn string(&self) -> String {
        match self {
            FlagValue::Bool(b) => b.get().to_string(),
            FlagValue::Int(i) => i.get().to_string(),
            FlagValue::String(v) => v.borrow().clone(),
        }
    }

    // Go: boolValue.IsBoolFlag
    pub fn is_bool_flag(&self) -> bool {
        matches!(self, FlagValue::Bool(_))
    }
}

/// Go `strconv.ErrSyntax` and `strconv.ErrRange`, the `*NumError` causes
/// that `numError` maps to `errParse` and `errRange`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumError {
    Syntax,
    Range,
}

// Go: internal/strconv/atoi.go:11 lower (go1.27.1)
fn lower(c: u8) -> u8 {
    c | (b'x' - b'X')
}

// Go: internal/strconv/atoi.go:172 ParseInt (go1.27.1), with base 0 and bitSize 64
// PORT: only the cause of a Go `*NumError` is kept (see `NumError`).

// Go: internal/strconv/atoi.go:47 ParseUint (go1.27.1), with base 0 and bitSize 64

// Go: internal/strconv/atoi.go:252 underscoreOK (go1.27.1)
// underscoreOK reports whether the underscores in s are allowed.
// Checking them in this one function lets all the parsers skip over them simply.
// Underscore must appear only between digits or between a base prefix and a digit.

// Go: strconv/atob.go ParseBool
// PORT: Go returns `(bool, error)`; the error is a `*NumError` that
// boolValue.Set replaces with errParse, so only `ok` is kept.
fn parse_bool(s: &str) -> (bool, bool) {
    match s {
        "1" | "t" | "T" | "TRUE" | "true" | "True" => (true, true),
        "0" | "f" | "F" | "FALSE" | "false" | "False" => (false, true),
        _ => (false, false),
    }
}

// Go: flag.go Flag
// A Flag represents the state of a flag.
pub struct Flag {
    pub name: String,      // name as it appears on command line
    pub usage: String,     // help message
    pub value: FlagValue,  // value as set
    pub def_value: String, // default value (as text); for usage message
}

// Go: flag.go FlagSet
// A FlagSet represents a set of defined flags.
// PORT: Go `Usage` is always `defaultUsage` here and `output` is always
// stderr; `undef` (from `Set` before a definition) has no caller.
pub struct FlagSet {
    pub name: String,
    pub parsed: bool,
    // PORT: Go `actual map[string]*Flag`; only the names are kept.
    pub actual: FxHashSet<String>,
    pub formal: FxHashMap<String, Flag>,
    pub args: Vec<String>, // arguments after flags
    pub error_handling: ErrorHandling,
}

// Go: flag.go:431 FlagSet.Output (go1.27.1)

// Go: flag.go:1223 NewFlagSet
// NewFlagSet returns a new, empty flag set with the specified name and
// error handling property. If the name is not empty, it will be printed
// in the default usage message and in error messages.
pub fn new_flag_set(name: &str, error_handling: ErrorHandling) -> FlagSet {
    FlagSet {
        name: name.to_string(),
        parsed: false,
        actual: FxHashSet::default(),
        formal: FxHashMap::default(),
        args: Vec::new(),
        error_handling,
    }
}

impl FlagSet {
    // Go: flag.go FlagSet.Bool
    // Bool defines a bool flag with specified name, default value, and usage string.
    // The return value is the address of a bool variable that stores the value of the flag.

    // Go: flag.go FlagSet.Int
    // Int defines an int flag with specified name, default value, and usage string.
    // The return value is the address of an int variable that stores the value of the flag.

    // Go: flag.go FlagSet.String
    // String defines a string flag with specified name, default value, and usage string.
    // The return value is the address of a string variable that stores the value of the flag.

    // Go: flag.go:1010 FlagSet.Var

    // Go: flag.go:1050 FlagSet.sprintf
    // sprintf formats the message, prints it to output, and returns it.

    // Go: flag.go:1058 FlagSet.failf
    // failf prints to standard error a formatted error and usage message and
    // returns the error.

    // Go: flag.go:1066 FlagSet.usage
    // usage calls the Usage method for the flag set if one is specified,
    // or the appropriate default usage function otherwise.

    // Go: flag.go:684 FlagSet.defaultUsage
    // defaultUsage is the default function to print a usage message.

    // Go: flag.go:607 FlagSet.PrintDefaults
    // PrintDefaults prints, to standard error unless configured otherwise, the
    // default values of all defined command-line flags in the set.
    // PORT: `isZeroValue` cannot fail for the three value kinds here.

    // Go: flag.go:1075 FlagSet.parseOne
    // parseOne parses one flag. It reports whether a flag was seen.
    // PORT: Go `(bool, error)`; an error always comes with `false`.

    // Go: flag.go:1153 FlagSet.Parse
    // Parse parses flag definitions from the argument list, which should not
    // include the command name. Must be called after all flags in the [FlagSet]
    // are defined and before flags are accessed by the program.
    // The return value will be [ErrHelp] if -help or -h were set but not defined.
}

// Go: flag.go:568 UnquoteUsage
// UnquoteUsage extracts a back-quoted name from the usage
// string for a flag and returns it and the un-quoted usage.
// Given "a `name` to show" it returns ("name", "a name to show").
// If there are no back quotes, the name is an educated guess of the
// type of the flag's value, or the empty string if the flag is boolean.
pub fn unquote_usage(flag: &Flag) -> (String, String) {
    // Look for a back-quoted name, but avoid the strings package.
    let usage = flag.usage.as_bytes();
    for i in 0..usage.len() {
        if usage[i] == b'`' {
            for j in i + 1..usage.len() {
                if usage[j] == b'`' {
                    let name = flag.usage[i + 1..j].to_string();
                    let usage = format!("{}{}{}", &flag.usage[..i], name, &flag.usage[j + 1..]);
                    return (name, usage);
                }
            }
            break; // Only one back quote; use type name.
        }
    }
    // No explicit name, so use type if we can find one.
    let name = match flag.value {
        FlagValue::Bool(_) => "",
        FlagValue::Int(_) => "int",
        FlagValue::String(_) => "string",
    };
    (name.to_string(), flag.usage.clone())
}

// Go: flag.go:538 isZeroValue
// isZeroValue determines whether the string represents the zero
// value for a flag.
// PORT: the zero values print as "false" (bool), "0" (int) and "" (string).
pub fn is_zero_value(flag: &Flag, value: &str) -> bool {
    let zero = match flag.value {
        FlagValue::Bool(_) => "false",
        FlagValue::Int(_) => "0",
        FlagValue::String(_) => "",
    };
    value == zero
}
