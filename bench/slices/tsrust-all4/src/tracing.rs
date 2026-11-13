//! Go: tracing/tracing.go (the `--generateTrace` session: trace.json,
//! types_N.json and legend.json), checker/tracer.go (the checker's tracer
//! and its type adapter) and the execute/tsc.go helpers
//! `startTracingIfNeeded` and `stopTracing`.
//!
//! PORT: Go passes the `*tracing.Tracing` through `ProgramOptions`,
//! `EmitInput` and the checker pool. The port has one program per process
//! (see `program.rs`), so the session is a process global: `get()` returns
//! it, or None when tracing is off (Go `tr == nil`). Each checker keeps its
//! `Tracer` in `Checker::tracer` as in Go.
//!
//! PORT: Go `Push` returns the `func()` that ends the event. Here it returns
//! a `Pop`; the event ends when the `Pop` drops, so Go
//! `defer tr.Push(...)()` is `let _trace = tr.push(...)`, which ends at the
//! end of the enclosing Rust scope.
//!
//! PORT: Go `map[string]any` args are `Args`: a list of (key, value) pairs.
//! Go writes the keys sorted (`json.Deterministic`); the writer here sorts
//! them the same way. No Go call site passes an empty non-nil map, so an
//! empty `Args` is the Go nil map and is left out of the JSON.
//!
//! PORT: Go writes through the program `vfs.FS` (`sys.FS()`). The session
//! is shared by the checker threads, so each thread writes through its own
//! `osvfs_fs()` (see `write_file`).

use crate::prelude::*;

use std::borrow::Cow;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// The tracing session of the process. Set once by `start_tracing`.
static TRACING: OnceLock<Tracing> = OnceLock::new();

/// The tracing session, or None when `--generateTrace` is off.
#[inline]
pub fn get() -> Option<&'static Tracing> {
    TRACING.get()
}

// Go: tracing/tracing.go:72 TraceRecord
#[derive(Clone, Debug, Default)]
pub struct TraceRecord {
    pub config_file_path: String,
    pub trace_path: String,
    pub types_path: String,
    pub checker_id: i64,
}

// Go: tracing/tracing.go:79 traceEvent
struct TraceEvent<'a> {
    pid: i64,
    tid: i64,
    ph: &'static str,
    cat: &'static str,
    ts: f64,
    name: &'static str,
    /// scope, only set for instant events ("g" = global)
    s: &'static str,
    dur: Option<f64>,
    args: &'a [(&'static str, Arg)],
}

// sampleInterval matches TypeScript's 10ms sampling interval.
// Events with separateBeginAndEnd=false are only recorded if their
// duration crosses a 10ms sampling boundary.
// Go: tracing/tracing.go:94 sampleInterval
const SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);

// Go: tracing/tracing.go:96 traceFileName
const TRACE_FILE_NAME: &str = "trace.json";

// Go: tracing/tracing.go:96
const MAIN_THREAD_ID: i64 = 1;
const FIRST_SYNTHETIC_THREAD_ID: i64 = 2;
const FIRST_FILE_THREAD_ID: i64 = 1_000_000;
const FILE_THREAD_ID_HASH_RANGE: u64 = 1_000_000_000;

// Go: tracing/tracing.go:105 traceThreadArgKeys
const TRACE_THREAD_ARG_KEYS: [&str; 5] = [
    "path",
    "fileName",
    "containingFileName",
    "jsFilePath",
    "declarationFilePath",
];

// flushThreshold is the size at which buffered trace content is flushed to disk
// via AppendFile. Keeps peak memory bounded for long-running compilations while
// avoiding a syscall per event.
// Go: tracing/tracing.go:110 flushThreshold
const FLUSH_THRESHOLD: usize = 256 * 1024;

// Go: tracing/tracing.go:113 Tracing
// Tracing manages the overall tracing session including all checkers.
// PORT: the Go fields that `mu` guards are in `TracingState`. Go `fs` is
// `std::fs`, or `osvfs_fs()` in a test process (see the module comment).
// Go `tracers` is not kept: each checker holds its own `TypeTracer` (see
// `stop_tracing`).
pub struct Tracing {
    trace_dir: String,
    trace_path: String,
    config_file_path: String,
    trace_started: AtomicBool,
    metadata_ts: f64,
    /// when true, use monotonic counter instead of real time
    deterministic: bool,
    start_time: std::time::Instant,
    state: Mutex<TracingState>,
}

struct TracingState {
    legend: Vec<TraceRecord>,
    trace_content: String,
    thread_ids: FxHashMap<TraceThreadKey, i64>,
    thread_keys: FxHashMap<i64, TraceThreadKey>,
    /// only used in deterministic mode
    timestamp_counter: u64,
    // flushErr holds the first error encountered while appending the trace buffer
    // to disk. Once set, subsequent flushes become no-ops and the error is
    // surfaced from StopTracing so that transient I/O failures (disk full,
    // permission denied, etc.) don't crash the compiler.
    flush_err: Option<String>,
}

// Go: tracing/tracing.go:137 Phase
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Parse,
    Program,
    Bind,
    Check,
    CheckTypes,
    Emit,
    Session,
}

impl Phase {
    // Go: tracing/tracing.go:136 (the Phase string values)
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Parse => "parse",
            Phase::Program => "program",
            Phase::Bind => "bind",
            Phase::Check => "check",
            Phase::CheckTypes => "checkTypes",
            Phase::Emit => "emit",
            Phase::Session => "session",
        }
    }
}

/// A Go `any` value of a trace event argument.
#[derive(Clone, Debug)]
pub enum Arg {
    Int(i64),
    Str(Cow<'static, str>),
    Bool(bool),
    Strs(Vec<String>),
}

/// Go `map[string]any` trace event arguments (see the module comment).
pub type Args = Vec<(&'static str, Arg)>;

impl From<i32> for Arg {
    fn from(value: i32) -> Self {
        Arg::Int(i64::from(value))
    }
}

impl From<u32> for Arg {
    fn from(value: u32) -> Self {
        Arg::Int(i64::from(value))
    }
}

impl From<i64> for Arg {
    fn from(value: i64) -> Self {
        Arg::Int(value)
    }
}

impl From<usize> for Arg {
    fn from(value: usize) -> Self {
        Arg::Int(value as i64)
    }
}

impl From<bool> for Arg {
    fn from(value: bool) -> Self {
        Arg::Bool(value)
    }
}

impl From<&'static str> for Arg {
    fn from(value: &'static str) -> Self {
        Arg::Str(Cow::Borrowed(value))
    }
}

impl From<String> for Arg {
    fn from(value: String) -> Self {
        Arg::Str(Cow::Owned(value))
    }
}

impl From<Vec<String>> for Arg {
    fn from(value: Vec<String>) -> Self {
        Arg::Strs(value)
    }
}

/// Go `ast.Kind` values are ints.
impl From<SyntaxKind> for Arg {
    fn from(value: SyntaxKind) -> Self {
        Arg::Int(value as i64)
    }
}

/// Go `TypeId` values are ints (the type id number).
impl From<TypeId> for Arg {
    fn from(value: TypeId) -> Self {
        Arg::Int(i64::from(value.0))
    }
}

fn find_arg<'a>(args: &'a [(&'static str, Arg)], key: &str) -> Option<&'a Arg> {
    args.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
}


// Go: tracing/tracing.go:152 StartTracing
// StartTracing creates a new tracing session.
// When deterministic is true, timestamps use a monotonic counter instead of
// real wall-clock time, producing stable output for test baselines.
// PORT: the session is the process global (`get`). A second start fails.

// Go: tracing/tracing.go:187 (*Tracing).timestamp
// timestamp returns the current timestamp in microseconds.
// In deterministic mode it returns a monotonically increasing counter;
// otherwise it returns the real elapsed wall-clock time since tracing started,
// matching TypeScript's 1000 * timestamp() (microseconds).
// PORT: a free function so `start_tracing` can call it before the session
// exists.
fn timestamp(deterministic: bool, start_time: std::time::Instant, state: &mut TracingState) -> f64 {
    if deterministic {
        state.timestamp_counter += 1;
        return state.timestamp_counter as f64;
    }
    micros(start_time.elapsed())
}

/// Go `float64(d.Nanoseconds()) / 1000.0`.
fn micros(d: std::time::Duration) -> f64 {
    d.as_nanos() as f64 / 1000.0
}

// Go: tracing/tracing.go:201 writeEventTo


impl Tracing {

    fn timestamp(&self, state: &mut TracingState) -> f64 {
        timestamp(self.deterministic, self.start_time, state)
    }

    // Go: tracing/tracing.go:205 (*Tracing).writeEvent

    // Go: tracing/tracing.go:213 (*Tracing).maybeFlushLocked
    // maybeFlushLocked appends the buffered trace content to disk if it has grown
    // past the flush threshold. Caller must hold tr.mu. If a previous flush failed,
    // or this flush fails, the error is recorded in tr.flushErr and subsequent
    // writes become no-ops; the error is surfaced from StopTracing.

    // Go: tracing/tracing.go:229 (*Tracing).Instant
    // Instant records an instant event in the trace.

    // Go: tracing/tracing.go:266 (*Tracing).Push
    // Push starts a trace event block on the shared trace buffer.
    // Safe to call from multiple threads.
    //
    // When separateBeginAndEnd is true, a "B" (begin) event is written immediately and
    // the returned value writes a matching "E" (end) event. This is used for events
    // that must always appear in the trace (e.g. checkSourceFile, createProgram, emit).
    //
    // When separateBeginAndEnd is false (the default in TypeScript), the event is only
    // recorded if its duration crosses a 10ms sampling boundary, matching TypeScript's
    // behavior of sampling short-lived events to avoid trace bloat.
    //
    // Returns a `Pop` that ends the event when it drops. Each one is
    // self-contained and does not depend on a shared stack.

    // Go: tracing/tracing.go:324 (*Tracing).threadIDLocked

    // Go: tracing/tracing.go:346 (*Tracing).writeThreadNameEventLocked

    // Go: tracing/tracing.go:411 (*Tracing).NewTypeTracer
    // NewTypeTracer creates a new tracer for a specific checker.
    // The checkerIndex is used to create unique filenames for each checker's output.
    // PORT: `goport` replaces a checker that panicked with a new checker of
    // the same index. Its tracer replaces the old one: the legend keeps one
    // entry per index, and the old checker's types are not dumped (their
    // ids belong to the dropped checker).

    // Go: tracing/tracing.go:434 (*Tracing).StopTracing
    // StopTracing finalizes the tracing session and writes all output files.
    // PORT: Go dumps the types of every `tr.tracers` entry. Each checker
    // lives on its own worker thread here, so each dumps its own types there
    // (`program::for_each_checker_parallel`). Only checkers that exist are
    // dumped: the pool is not made for this.
}

/// A trace event that has begun. It ends when it drops (Go calls the
/// function that `Push` returns).
///
/// PORT: the state is boxed so an `Option<Pop>` is one word on the stack of
/// the recursive checker functions that hold one.
#[must_use = "the trace event ends when this value drops; bind it to a named variable"]
pub struct Pop(Option<Box<PopState>>);

struct PopState {
    tracing: &'static Tracing,
    phase: Phase,
    name: &'static str,
    args: Args,
    kind: PopKind,
}

enum PopKind {
    /// A "B" event was written; the "E" event uses the same thread.
    End { tid: i64 },
    /// A sampled event that started at `start_time`.
    Sampled { start_time: std::time::Instant },
}

impl Pop {
}


// Go: tracing/tracing.go:353 traceThreadKind
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TraceThreadKind {
    Checker,
    File,
}

impl TraceThreadKind {
    fn as_str(self) -> &'static str {
        match self {
            TraceThreadKind::Checker => "checker",
            TraceThreadKind::File => "file",
        }
    }
}

// Go: tracing/tracing.go:360 traceThreadKey
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct TraceThreadKey {
    kind: TraceThreadKind,
    text: String,
    index: i64,
    has_index: bool,
}

// Go: tracing/tracing.go:367 traceThreadKeyFromArgs
fn trace_thread_key_from_args(args: &[(&'static str, Arg)]) -> Option<TraceThreadKey> {
    if args.is_empty() {
        return None;
    }

    if let Some(Arg::Int(checker_id)) = find_arg(args, "checkerId") {
        return Some(TraceThreadKey {
            kind: TraceThreadKind::Checker,
            text: String::new(),
            index: *checker_id,
            has_index: true,
        });
    }

    for key in TRACE_THREAD_ARG_KEYS {
        if let Some(Arg::Str(path)) = find_arg(args, key) {
            if !path.is_empty() {
                return Some(TraceThreadKey {
                    kind: TraceThreadKind::File,
                    text: path.to_string(),
                    index: 0,
                    has_index: false,
                });
            }
        }
    }

    None
}

impl TraceThreadKey {
    // Go: tracing/tracing.go:387 traceThreadKey.defaultThreadID
    fn default_thread_id(&self) -> i64 {
        if self.kind == TraceThreadKind::Checker && self.has_index && self.index >= 0 {
            return FIRST_SYNTHETIC_THREAD_ID + self.index;
        }
        stable_trace_thread_id(self)
    }

    // Go: tracing/tracing.go:394 traceThreadKey.displayName
    fn display_name(&self) -> String {
        if self.has_index {
            return format!("{}:{}", self.kind.as_str(), self.index);
        }
        format!("{}:{}", self.kind.as_str(), self.text)
    }
}

// Go: tracing/tracing.go:401 stableTraceThreadID
// PORT: Go writes the parts into a streaming xxh3 hasher (seed 0); the hash
// of the joined bytes is the same.
fn stable_trace_thread_id(key: &TraceThreadKey) -> i64 {
    let text = if key.has_index {
        format!("{}:{}", key.kind.as_str(), key.index)
    } else {
        format!("{}:{}", key.kind.as_str(), key.text)
    };
    let hash = crate::xxhash_rust::xxh3::xxh3_64(text.as_bytes());
    FIRST_FILE_THREAD_ID + (hash % FILE_THREAD_ID_HASH_RANGE) as i64
}

// Go: tracing/tracing.go:434 json.MarshalIndent(tr.legend, "", "  ")
// PORT: the Go v2 JSON indent output, written by hand.

// ---------------------------------------------------------------------------
// Type tracer
// ---------------------------------------------------------------------------

// Go: tracing/tracing.go:485 typeTracer
// typeTracer is the per-checker tracer implementation
// PORT: Go keeps `*TracedType` values; the port keeps the type ids, which
// index the checker's arena.
pub struct TypeTracer {
    checker_index: usize,
    types_path: String,
    types: Mutex<Vec<TypeId>>,
}

impl TypeTracer {
    // Go: tracing/tracing.go:495 (*typeTracer).RecordType

    // Go: tracing/tracing.go:501 (*typeTracer).DumpTypes
    // PORT: Go reaches the checker through each type. Here the checker that
    // owns the types is passed in.
}


// Go: tracing/tracing.go:534 TypeDescriptor
// TypeDescriptor represents a type in the output JSON
// PORT: Go `omitzero` fields: an empty `Vec`, an empty `String` and `None`
// are left out.
#[derive(Default)]
struct TypeDescriptor {
    id: u32,
    intrinsic_name: String,
    symbol_name: String,
    recursion_id: Option<i64>,
    is_tuple: bool,
    union_types: Vec<u32>,
    intersection_types: Vec<u32>,
    alias_type_arguments: Vec<u32>,
    keyof_type: Option<i64>,
    indexed_access_object_type: Option<i64>,
    indexed_access_index_type: Option<i64>,
    conditional_check_type: Option<i64>,
    conditional_extends_type: Option<i64>,
    // ConditionalTrueType and ConditionalFalseType can be -1: unresolved
    // conditional branches are serialized as -1, matching TypeScript's behavior.
    conditional_true_type: Option<i64>,
    conditional_false_type: Option<i64>,
    substitution_base_type: Option<i64>,
    constraint_type: Option<i64>,
    instantiated_type: Option<i64>,
    type_arguments: Vec<u32>,
    reference_location: Option<Location>,
    reverse_mapped_source_type: Option<i64>,
    reverse_mapped_mapped_type: Option<i64>,
    reverse_mapped_constraint_type: Option<i64>,
    evolving_array_element_type: Option<i64>,
    evolving_array_final_type: Option<i64>,
    destructuring_pattern: Option<Location>,
    first_declaration: Option<Location>,
    flags: Vec<String>,
    display: String,
}

// Go: tracing/tracing.go:569 Location
// Location represents a source code location
struct Location {
    path: String,
    start: Option<LineAndChar>,
    end: Option<LineAndChar>,
}

// Go: tracing/tracing.go:576 LineAndChar
// LineAndChar represents a line and character position (1-indexed)
struct LineAndChar {
    line: i32,
    character: i32,
}

impl TypeDescriptor {
}

fn write_opt_int(out: &mut String, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        let _ = write!(out, ",\"{key}\":{value}");
    }
}

fn write_id_list(out: &mut String, key: &str, ids: &[u32]) {
    if ids.is_empty() {
        return;
    }
    let _ = write!(out, ",\"{key}\":[");
    for (i, id) in ids.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, "{id}");
    }
    out.push(']');
}


// Go: tracing/tracing.go:585 (*typeTracer).buildTypeDescriptor
// PORT: Go reads the type through the `TracedType` adapter
// (checker/tracer.go). The adapter methods are inlined here, each marked
// with its Go name.

// Go: checker/tracer.go:330 Display
// Compute display text for types where it's valuable for trace analysis.
// TypeScript only does this for Anonymous|Literal types, but we extend to
// unions, intersections, and template literals since they often lack
// firstDeclaration and the display text helps identify them.
// Incomplete types during tracing can cause panics, which we intentionally
// suppress (returning ""), matching TypeScript's try/catch around typeToString.
// PORT: Go `recover()` is `catch_unwind`. An unported path reached here
// still counts in the unported report.

// Go: tracing/tracing.go:726 mapTypeIds
fn map_type_ids(types: &[TypeId]) -> Vec<u32> {
    // Go: a nil type maps to 0, which is `TypeId::NIL.0`.
    types.iter().map(|t| t.0).collect()
}


// Go: tracing/tracing.go:739 getLocation

// ---------------------------------------------------------------------------
// Checker tracer
// ---------------------------------------------------------------------------

// Go: checker/tracer.go:13 Tracer
// Tracer records types and trace events during type checking. A None
// `Checker::tracer` is the Go nil `*Tracer`, so call sites use
// `if let Some(tr) = self.tracer` to gate work that only matters under
// --generateTrace.
#[derive(Clone, Copy)]
pub struct Tracer {
    tracing: &'static Tracing,
    recorder: &'static TypeTracer,
    checker_index: usize,
}

// Go: checker/tracer.go:21 NewTracer
// NewTracer creates a Tracer for the given checker index that records both
// type-creation events and trace events through the provided tracing session.


impl Tracer {
    // Go: checker/tracer.go:25 (*Tracer).RecordType

    // Go: checker/tracer.go:29 (*Tracer).Push
    // PORT: Go adds "checkerId" to a copy of the args for a sampled event,
    // and to the caller's map for the "B" and "E" events of a separate one.
    // Both end with the same args plus "checkerId"; the `Pop` owns them.

    // Go: checker/tracer.go:45 (*Tracer).Instant
}

// ---------------------------------------------------------------------------
// execute/tsc.go helpers
// ---------------------------------------------------------------------------

// Go: execute/tsc.go:28 startTracingIfNeeded
// PORT: Go prints the warning to `sys.Writer()` and returns the session.
// The session is the process global here, and the warning text (with its
// newline) is returned for the caller to print. `testing` is Go
// `testing != nil`.

// Go: execute/tsc.go:44 stopTracing
// PORT: the warning text is returned, as for `start_tracing_if_needed`.
// Call it on the thread that loaded the program (it reaches the checkers).

// ---------------------------------------------------------------------------
// JSON and file helpers
// ---------------------------------------------------------------------------

// Go: go-json-experiment jsonwire.AppendQuote with the default flags: only
// `"`, `\` and control characters are escaped (no HTML or JS escaping).
// PORT: Go `internal/json` marshals with `AllowInvalidUTF8`, so each byte
// that is not valid UTF-8 becomes U+FFFD. `s` is the port form of a Go
// string (see `scanner_util::GO_STRING_MARKER`), and
// `json::append_json_quote` does that replacement.

// Go: go-json-experiment jsonwire.AppendFloat(dst, src, 64)
// PORT: Rust `{}` prints the shortest round-trip digits without an
// exponent, which is Go `strconv.AppendFloat(f, 'f', -1, 64)`.

// Go: `tr.fs.WriteFile`, where `tr.fs` is `sys.FS()`: vfs/osvfs/os.go:174
// WriteFile, or the test file system in a test process.
// PORT: `osvfs_fs()` is the file system of the calling thread: the OS one,
// or the test one when a test process installs an OS override
// (`osvfs::install_os_override`). The error text is Go `err.Error()`.

// Go: `tr.fs.AppendFile` (vfs/osvfs/os.go:178 AppendFile); see
// `write_file`.
