//! Port of Go `internal/contentmapper/hostimpl.go` (tsgo#4712).
//!
//! PORT: threads. The host, its projects and its connections are
//! dispatch-thread values (see the module comment in `mod.rs`): Go's
//! `lifecycleMu` and `mu` are dropped, and the maps are `RefCell`s. Go holds
//! `mu` across some protocol calls; here each call runs with no map
//! borrowed. Other threads share the spawned process (`ProcessExitState`, an
//! `ipc::ReadWriteCloser`), its connection (`MuxConn`, whose read loop runs
//! on its own thread), the timing collector and the stderr logger, so these
//! use `Arc`, `Mutex` and atomics. A parse worker sends the transform
//! requests of an open project through a `ConcurrentTransform`.
//!
//! PORT: Go map iteration order is random. The host maps are `IndexMap`s in
//! insertion order, so the order of the close calls is fixed.

use crate::contentmapper::prelude::*;

use crate::contentmapper::muxconn::{};

use crate::flags_macros::go_enum;
use crate::frontend::json_ext::{
    AnyValue, 
    write_object_end, write_object_start,
};
use crate::gostd::context::{self, };
use crate::ipc::{ReadWriteCloser as _};
use crate::locale::Locale;
use crate::options_json::{};
use std::borrow::Cow;
use std::cell::Cell;
use std::io::Write;
use std::rc::Weak;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

/// wasm cannot start a process (execute/tsc/compile.rs `spawn_process`), so
/// no mapper has a connection there: its dial always fails. The wasm build
/// calls this where a connection would be used, so that the compiler leaves
/// the mapper protocol (ipc and the requests) out of the wasm module.
// PORT: not in Go.
#[cfg(target_family = "wasm")]
fn no_mapper_connection() {
    unreachable!("wasm cannot start a content mapper process");
}

// PORT: Go mutexes do not poison.

// Go: contentmapper/hostimpl.go:30 initializeTimeoutSeconds
const INITIALIZE_TIMEOUT_SECONDS: i32 = 5;

// Go: contentmapper/hostimpl.go:32 initializeTimeout
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(INITIALIZE_TIMEOUT_SECONDS as u64);

// Go: contentmapper/hostimpl.go:35
// Content mapper protocol method names.
pub const METHOD_INITIALIZE: &str = "initialize";
pub const METHOD_OPEN_PROJECT: &str = "openProject";
pub const METHOD_CLOSE_PROJECT: &str = "closeProject";
pub const METHOD_TRANSFORM: &str = "transform";

// Go: contentmapper/hostimpl.go:43 InitializeParams
// InitializeParams is the parameter object for the initialize request.



// Go: contentmapper/hostimpl.go:51 InitializeResult
// InitializeResult is the mapper's response to the initialize request.



// Go: contentmapper/hostimpl.go:59 OpenProjectParams
// OpenProjectParams is the parameter object for the openProject request.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OpenProjectParams {
    // ConfigFileName is the absolute project configuration file name, or empty when there is none.
    pub config_file_name: String,
    // ProjectHandle is an opaque, process-local handle assigned by the host.
    pub project_handle: String,
    // Options is the mapper entry's options from the project's contentMappers configuration.
    pub options: JsonValue,
    // CompilerOptions contains the project's effective compiler options.
    pub compiler_options: JsonValue,
}



// Go: contentmapper/hostimpl.go:72 OpenProjectResult
// OpenProjectResult is the mapper's response to an openProject request. ConfigIdentity and WatchedFiles
// may only be returned by mappers that declare dynamicConfig.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OpenProjectResult {
    // ConfigIdentity is a stable fingerprint of all dynamic configuration that can affect transforms.
    pub config_identity: String,
    // WatchedFiles are absolute files whose changes may alter ConfigIdentity or transform output.
    pub watched_files: Vec<String>,
    // OptionDiagnostics report invalid mapper options. Paths are relative to the mapper entry's options object.
    pub option_diagnostics: Vec<OptionDiagnosticResult>,
}



// Go: contentmapper/hostimpl.go:81 OptionDiagnosticResult
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OptionDiagnosticResult {
    pub path: Vec<JsonValue>,
    pub message_text: String,
    pub code: i32,
}



// Go: contentmapper/hostimpl.go:88 CloseProjectParams
// CloseProjectParams is the parameter object for the closeProject request.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CloseProjectParams {
    pub project_handle: String,
}



// Go: contentmapper/hostimpl.go:93 PositionEncoding
// PositionEncoding is the coordinate space a mapper uses for mappings and diagnostics.
// PORT: a Go string type, like the lsproto string enums.

// Go: contentmapper/hostimpl.go:95



// Go: contentmapper/hostimpl.go:101 TransformParams
// TransformParams is the parameter object for the transform request.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransformParams {
    // FileName is the absolute name of the content-mapped source file being transformed.
    pub file_name: String,
    // Content is the content-mapped source file's text.
    pub content: String,
    // ProjectHandle identifies the mapper project configuration opened for this transform.
    pub project_handle: String,
}



// Go: contentmapper/hostimpl.go:111 MappedOutput
// MappedOutput is virtual source text and its mapping to an original input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MappedOutput {
    // Text is the virtual JavaScript or TypeScript source text.
    pub text: String,
    // Extension determines the virtual source file's syntax.
    pub extension: String,
    // Mappings is the span map's tuple-array JSON (see spanmap.Marshal), expressed in the selected
    // position encoding. Absent or empty means the output is fully synthesized.
    pub mappings: JsonValue,
    // DiagnosticDirectives describe framework directives that suppress TypeScript diagnostics in
    // virtual ranges and optionally report an error when no diagnostic is produced.
    // PORT: Go `*DiagnosticDirectives`; nil is `None`.
    pub diagnostic_directives: Option<DiagnosticDirectives>,
}

impl MappedOutput {

}



// Go: contentmapper/hostimpl.go:125 DiagnosticDirectivePolicy
// DiagnosticDirectivePolicy is the numeric policy stored in a mapped diagnostic directive tuple.
go_enum!(DiagnosticDirectivePolicy, u8 {
    IGNORE = 0;
    EXPECT = 1;
});

impl MarshalerTo for DiagnosticDirectivePolicy {
    fn marshal_json_to(&self, enc: &mut String) -> std::result::Result<(), JsonError> {
        u32::from(self.0).marshal_json_to(enc)
    }
}


// Go: contentmapper/hostimpl.go:132 UnusedExpectDirectiveDiagnostic
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UnusedExpectDirectiveDiagnostic {
    pub code: i32,
    pub message_text: String,
}



// Go: contentmapper/hostimpl.go:138 DiagnosticDirectives
// DiagnosticDirectives shares unused-expect diagnostics across compact directive tuples.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiagnosticDirectives {
    pub unused_expect_directive_diagnostics: Vec<UnusedExpectDirectiveDiagnostic>,
    pub directives: Vec<MappedDiagnosticDirective>,
}



// Go: contentmapper/hostimpl.go:146 MappedDiagnosticDirective
// MappedDiagnosticDirective is encoded as
// [originalStart, originalLength, virtualStart, virtualEnd, policy, unusedExpectDirectiveIndex?].
// An omitted index selects the only unused-expect diagnostic and is invalid when there is not exactly one.
// PORT: Go `*int` is `Option<i32>` (nil is `None`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MappedDiagnosticDirective {
    pub original_start: i32,
    pub original_length: i32,
    pub virtual_start: i32,
    pub virtual_end: i32,
    pub policy: DiagnosticDirectivePolicy,
    pub unused_expect_directive_index: Option<i32>,
}

// Go: contentmapper/hostimpl.go:160 MappedDiagnosticDirective.MarshalJSONTo
impl MarshalerTo for MappedDiagnosticDirective {
    fn marshal_json_to(&self, enc: &mut String) -> std::result::Result<(), JsonError> {
        let mut tuple: Vec<i32> = vec![
            self.original_start,
            self.original_length,
            self.virtual_start,
            self.virtual_end,
            i32::from(self.policy.0),
        ];
        if let Some(index) = self.unused_expect_directive_index {
            tuple.push(index);
        }
        tuple.marshal_json_to(enc)
    }
}

// Go: contentmapper/hostimpl.go:168 MappedDiagnosticDirective.UnmarshalJSONFrom
// PORT: Go returns plain `fmt.Errorf` errors from the method; here they are
// `JsonError`s with the same text (the JSON layer wraps method errors).

// Go: contentmapper/hostimpl.go:193 SupplementalOutput
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SupplementalOutput {
    pub mapped_output: MappedOutput,
}



// Go: contentmapper/hostimpl.go:198 TransformResult
// TransformResult is the canonical output for one input file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TransformResult {
    pub mapped_output: MappedOutput,
    // Diagnostics are mapper-authored errors expressed in original-source coordinates.
    pub diagnostics: Vec<Diagnostic>,
    // Supplemental contains additional unnamed compiler inputs associated with this source file.
    pub supplemental: Vec<SupplementalOutput>,
}



// Go: contentmapper/hostimpl.go:207 Diagnostic
// Diagnostic is an error reported by a mapper in original-source coordinates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Diagnostic {
    // MessageText is the diagnostic message.
    pub message_text: String,
    // Start and Length locate the diagnostic in the original content using the selected position encoding.
    pub start: i32,
    pub length: i32,
    pub code: i32,
}



// Go: contentmapper/hostimpl.go:218 dialFunc
// dialFunc establishes a running connection to a mapper. In production it spawns the mapper's process;
// tests substitute an in-memory connection. It returns the connection and a closer that tears it down.
// PORT: Go's `io.Closer` is the spawned process connection.

// Go: contentmapper/hostimpl.go:221 host
// host manages one child process per mapper identity. It is the production implementation of Host.
// PORT: Go unexported `host`; other packages see it through the `Host`
// interface, so the Rust name is `HostImpl` (as `CompilerHostImpl`). Go nil
// maps after Close are `None`. `this` gives the project leases and the
// acquire releases their `*host` pointer.

// Go: contentmapper/hostimpl.go:238 projectEntry

// Go: contentmapper/hostimpl.go:248 mapperConn

// Go: contentmapper/hostimpl.go:260 operationTiming
// PORT: Go `operationTiming` and `OperationTiming` (host.go) are both
// `OperationTiming` in Rust; the unexported one is `OperationTimingImpl`.
// The parse workers record transforms too (`ConcurrentTransform`), so the
// collectors keep Go's atomics and mutex.
#[derive(Default)]
struct OperationTimingImpl {
    count: AtomicU64,
    /// Nanoseconds (Go `atomic.Int64` of a `time.Duration`).
    duration: AtomicU64,
}

impl OperationTimingImpl {
    // Go: contentmapper/hostimpl.go:265 operationTiming.record
    fn record(&self, start: Instant) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.duration.fetch_add(
            u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
    }

    // Go: contentmapper/hostimpl.go:270 operationTiming.snapshot
    fn snapshot(&self) -> OperationTiming {
        OperationTiming {
            count: self.count.load(Ordering::Relaxed),
            duration: Duration::from_nanos(self.duration.load(Ordering::Relaxed)),
        }
    }
}

// Go: contentmapper/hostimpl.go:274 timingCollector
struct TimingCollector {
    mu: Mutex<TimingState>,
}

/// The fields of Go `timingCollector` that its `mu` guards.
struct TimingState {
    mappers: IndexMap<String, Arc<MapperTimingCollector>>,
    active_requests: u64,
    request_wait_start: Instant,
    request_wait_elapsed: Duration,
}

// Go: contentmapper/hostimpl.go:282 mapperTimingCollector
// PORT: Go's `owner` pointer is a `Weak`, so the collector and its mapper
// entries do not keep each other alive.
#[derive(Default)]
struct MapperTimingCollector {
    spawn: OperationTimingImpl,
    initialize: OperationTimingImpl,
    open_project: OperationTimingImpl,
    close_project: OperationTimingImpl,
    transform: OperationTimingImpl,
    owner: std::sync::Weak<TimingCollector>,
}

impl TimingCollector {
    /// Go `&timingCollector{mappers: make(map[string]*mapperTimingCollector)}`.
    fn new() -> TimingCollector {
        TimingCollector {
            mu: Mutex::new(TimingState {
                mappers: IndexMap::new(),
                active_requests: 0,
                request_wait_start: Instant::now(),
                request_wait_elapsed: Duration::ZERO,
            }),
        }
    }

    // Go: contentmapper/hostimpl.go:291 timingCollector.mapper

    // Go: contentmapper/hostimpl.go:302 timingCollector.snapshot
}

impl MapperTimingCollector {
    fn owner(&self) -> Arc<TimingCollector> {
        self.owner
            .upgrade()
            .expect("content mapper timing collector outlives its host")
    }

    // Go: contentmapper/hostimpl.go:324 mapperTimingCollector.startRequest

    // Go: contentmapper/hostimpl.go:334 mapperTimingCollector.finishRequest
}

// Go: contentmapper/hostimpl.go:349 Spawner
// Spawner starts a child process, returning its stdio as an io.ReadWriteCloser (Read is the
// process's stdout, Write is its stdin) whose Close tears the process down. This seam keeps os/exec out
// of this package: production hosts spawn a real process, tests supply an in-process pipe.
// PORT: Go `io.Discard` as `stderr` is `None`. The result is a
// `ProcessExitState` (see there).

// Go: contentmapper/hostimpl.go:353 processExitState
// PORT: Go type-asserts the spawned `io.ReadWriteCloser` for
// `ExitCode() (int, bool)`. Rust has no dynamic interface check, so a
// spawned process is this trait: an `ipc::ReadWriteCloser` whose
// `exit_code` defaults to `(0, false)`, the result for a Go value that has no
// `ExitCode` method.
pub trait ProcessExitState: ipc::ReadWriteCloser {
    fn exit_code(&self) -> (i32, bool) {
        (0, false)
    }
}


// Go: contentmapper/hostimpl.go:358 SpawnerFunc
// SpawnerFunc adapts a spawn function to the Spawner interface.


// Go: contentmapper/hostimpl.go:365 Logger
// Logger receives content mapper protocol and process output as complete log lines.
// PORT: the stderr lines come from the spawner's thread, so it is `Send + Sync`.

// Go: contentmapper/hostimpl.go:368 HostOptions
// HostOptions configures optional content mapper process logging.
// PORT: Go nil `Logger` is `None`.

// Go: contentmapper/hostimpl.go:372 loggingProtocol
// PORT: Go embeds `ipc.Protocol`; the methods below forward to `protocol`.



// Go: contentmapper/hostimpl.go:419 stderrLogger
// PORT: Go keeps `pending` as a string of raw bytes. Here it is bytes too,
// so a character split between two writes stays whole; each complete line
// goes to the logger as text (invalid UTF-8 becomes U+FFFD).




// Go: contentmapper/hostimpl.go:454 loggedProcess
// PORT: Go embeds `io.ReadWriteCloser`, which does not promote the spawned
// value's `ExitCode` method, so a logged process reports no exit state
// (`exit_code` keeps the default).



// Go: contentmapper/hostimpl.go:465 closeOnceReadWriteCloser



/// The error text prefix of Go `ipc.AsyncConn.Call` for an error response.
const IPC_REMOTE_ERROR_PREFIX: &str = "ipc: remote error [";

// PORT: Go runs the connection's read loop on a goroutine and closes the
// process when it ends: `go func() { _ = conn.Run(ctx); _ = rwc.Close() }()`.
// `MuxConn` reads on its own thread, as `Run` does, so that the parse
// workers can call the mapper too (`ConcurrentTransform`). This connection
// closes the process when a call fails because the connection ended (a read
// or a write failed), which is where Go's read loop ends. A call that ends
// with `ctx` or with the mapper's error response leaves the process open,
// as in Go.



// PORT: not in Go, where the parse goroutines call `projectLease.Transform`
// (hostimpl.go:1000) on an open project. The project is dispatch-thread
// state; this is what a parse worker needs to send the transform request of
// a file of an open project, as `transform_locked` does: the mapper's
// connection, which threads can share (`MuxConn`), the project handle, the
// host context, the timing and what decoding needs. The loader disables it
// when the mapper fails (`disable`), so that the workers send no more.


// Go: contentmapper/hostimpl.go:487 NewHost
// NewHost creates a Host that spawns each mapper's process via the given spawner and drives it over a
// JSON-RPC connection. The host's lifetime is bound to ctx: cancelling it (e.g. the CLI's signal context
// on SIGINT, or a build/watch session ending) tears every mapper process down, so owners of a session
// context need not close the host explicitly. Close does the same synchronously.

// Go: contentmapper/hostimpl.go:492 NewHostWithOptions
// NewHostWithOptions creates a Host with optional protocol and process logging.

// Go: contentmapper/hostimpl.go:555 newWithDial
// PORT: Go's `AfterFunc(ctx, func() { _ = h.Close() })` would close the
// host from another goroutine. The host is dispatch-thread state, so the
// callback closes the spawned processes (the part of Close that other
// threads can reach, through `processes`); the host maps stay until the
// owner closes or drops the host.
// Go has no close at drop, and neither has the host. A mapper process
// closes at `close`, when `ctx` ends (the callback above), or when the last
// project lease or `acquire` that uses its identity is released
// (`release`). The leases and the acquire releases hold the host (`rc`), as
// Go's `*host` pointers do, and the `MuxConn` reader thread holds each
// process, as Go's `Run` goroutine holds `rwc`. So, as in Go, an owner that
// drops the host while a project or an acquire is not released leaves
// those processes running until `ctx` ends. The owners follow Go's: `tsc`
// closes its project, `tsc -b` closes the host, and the watcher and the LSP
// session end with their context.



// Go: contentmapper/hostimpl.go:627 projectSpecKey
// PORT: Go `%p` of a nil pointer prints `0x0`, as Rust `{:p}` of a null pointer.

// Go: contentmapper/hostimpl.go:636 combinedIdentity
// PORT: Go `hex.EncodeToString` of the big-endian `Bytes()` of an
// `xxh3.Uint128` is `{:032x}` of the `u128`.


// Go: jsontext `Value.Kind`: the kind of the value's first token, with
// every number as '0'; 0 for an empty value.

// Go: contentmapper/hostimpl.go:853 projectLease
// PORT: Go `map[*Mapper]string` is an `IndexMap` keyed by the mapper's `Rc`
// pointer, in insertion order. `refs` and `once` are cells (dispatch thread).

// Go: contentmapper/hostimpl.go:862 retainedProject
// PORT: Go embeds `*projectLease`; every method but Close forwards to it.




// Go: contentmapper/hostimpl.go:1091 handshake

// Go: contentmapper/hostimpl.go:1121 decodeTransformResult

// Go: contentmapper/hostimpl.go:1170 decodeMappedOutput

// Go: contentmapper/hostimpl.go:1208 normalizeDiagnosticDirectives

// Go: contentmapper/hostimpl.go:1291 normalizeMappings

// Go: contentmapper/hostimpl.go:1316 positionNormalizer

// Go: contentmapper/hostimpl.go:1323 newPositionNormalizer


// Go: contentmapper/hostimpl.go:1364 rejectHandler
// rejectHandler rejects any request initiated by the mapper. The content mapper protocol is currently
// parent-driven only; a request from the child is a protocol violation.
struct RejectHandler;


#[cfg(all(test, unix))]
mod tests {
    // Go: contentmapper/host_test.go
    //
    // PORT: Go serves each fake mapper with `ipc.NewAsyncConn(server, handler).Run`
    // on a goroutine over `net.Pipe`. Here the pipe is a Unix socket pair,
    // and the mapper side runs on its own thread: the handler is `Send`
    // (`MapperHandler`), and the thread makes its own connection around it.
    use super::*;
    use crate::gostd::context::background;
    use crate::ipc::{Protocol as _, ReadWriteCloser as _};
    use std::io::Read;
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream;
    use std::sync::atomic::AtomicI32;
    use std::sync::{Once, mpsc};

    /// One end of Go `net.Pipe`.
    struct PipeEnd {
        stream: UnixStream,
    }

    impl ipc::ReadWriteCloser for PipeEnd {
        fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            (&self.stream).read(buf)
        }

        fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
            (&self.stream).write(buf)
        }

        fn flush(&self) -> std::io::Result<()> {
            (&self.stream).flush()
        }

        // Go `net.Pipe` Close returns nil.
        fn close(&self) -> std::result::Result<(), GoError> {
            let _ = self.stream.shutdown(Shutdown::Both);
            Ok(())
        }
    }

    impl ProcessExitState for PipeEnd {}

    /// Go `net.Pipe()`.
    fn net_pipe() -> (Arc<PipeEnd>, Arc<PipeEnd>) {
        let (client, server) = UnixStream::pair().expect("socket pair");
        (
            Arc::new(PipeEnd { stream: client }),
            Arc::new(PipeEnd { stream: server }),
        )
    }

    /// A fake mapper process: Go `ipc.Handler` values that the tests pass to
    /// `fakeSpawner`.
    trait MapperHandler: Send + Sync {
        fn handle_request(
            &self,
            method: &str,
            params: &JsonValue,
        ) -> std::result::Result<Option<Box<dyn AnyValue>>, GoError>;

        /// Go `interface{ handlesProjects() }`.
        fn handles_projects(&self) -> bool {
            false
        }
    }

    // Go: host_test.go:198 noOpProjectMapper, applied by `fakeSpawner.Spawn`.
    struct HandlerAdapter(Arc<dyn MapperHandler>);

    impl ipc::Handler for HandlerAdapter {
        fn handle_request(
            &self,
            _ctx: &Context,
            method: &str,
            params: JsonValue,
        ) -> std::result::Result<Option<Box<dyn AnyValue>>, GoError> {
            if !self.0.handles_projects() {
                match method {
                    METHOD_OPEN_PROJECT => return Ok(Some(Box::new(OpenProjectResult::default()))),
                    METHOD_CLOSE_PROJECT => return Ok(None),
                    _ => {}
                }
            }
            self.0.handle_request(method, &params)
        }

        fn handle_notification(
            &self,
            _ctx: &Context,
            _method: &str,
            _params: JsonValue,
        ) -> std::result::Result<(), GoError> {
            Ok(())
        }
    }

    fn unmarshal_params<T: UnmarshalerFrom + Default>(
        params: &JsonValue,
    ) -> std::result::Result<T, GoError> {
        let mut p = T::default();
        json_unmarshal(&params.0, &mut p, &[]).map_err(errors::from_value)?;
        Ok(p)
    }

    fn unexpected_method(method: &str) -> GoError {
        errors::new(format!("unexpected method {method}"))
    }

    fn initialize_result(source: &str) -> Box<dyn AnyValue> {
        Box::new(InitializeResult {
            position_encoding: PositionEncoding::UTF8,
            diagnostic_source: source.to_string(),
        })
    }

    // Go: host_test.go:28 fakeMapper
    // fakeMapper is an in-process mapper that transforms content verbatim and reports one diagnostic.
    struct FakeMapper;

    impl MapperHandler for FakeMapper {
        // Go: host_test.go:53 fakeMapper.HandleRequest
        fn handle_request(
            &self,
            method: &str,
            params: &JsonValue,
        ) -> std::result::Result<Option<Box<dyn AnyValue>>, GoError> {
            match method {
                METHOD_INITIALIZE => Ok(Some(initialize_result("vue"))),
                METHOD_TRANSFORM => {
                    let p: TransformParams = unmarshal_params(params)?;
                    let length = p.content.len() as i32;
                    let mappings = spanmap::new(&[spanmap::Segment {
                        virtual_end: length,
                        original_end: length,
                        kind: spanmap::Kind::VERBATIM,
                        ..Default::default()
                    }])
                    .marshal()?;
                    Ok(Some(Box::new(TransformResult {
                        mapped_output: MappedOutput {
                            text: p.content,
                            extension: ".ts".to_string(),
                            mappings: JsonValue(mappings),
                            ..Default::default()
                        },
                        diagnostics: vec![Diagnostic {
                            message_text: "boom".to_string(),
                            start: 0,
                            length: 3.min(length),
                            code: 9999,
                        }],
                        ..Default::default()
                    })))
                }
                _ => Err(unexpected_method(method)),
            }
        }
    }

    type ResponseFn = dyn Fn(TransformParams) -> Box<dyn AnyValue> + Send + Sync;

    // Go: host_test.go:30 responseMapper
    struct ResponseMapper {
        response: Box<ResponseFn>,
    }

    impl MapperHandler for ResponseMapper {
        // Go: host_test.go:34 responseMapper.HandleRequest
        fn handle_request(
            &self,
            method: &str,
            params: &JsonValue,
        ) -> std::result::Result<Option<Box<dyn AnyValue>>, GoError> {
            match method {
                METHOD_INITIALIZE => Ok(Some(initialize_result("mapper"))),
                METHOD_TRANSFORM => {
                    let p: TransformParams = unmarshal_params(params)?;
                    Ok(Some((self.response)(p)))
                }
                _ => Err(unexpected_method(method)),
            }
        }
    }

    fn response_mapper(
        response: impl Fn(TransformParams) -> Box<dyn AnyValue> + Send + Sync + 'static,
    ) -> Arc<dyn MapperHandler> {
        Arc::new(ResponseMapper {
            response: Box::new(response),
        })
    }

    // Go: host_test.go:84 unicodeMapper
    struct UnicodeMapper {
        encoding: PositionEncoding,
        source: Option<String>,
    }

    // Go: host_test.go:89 protocolDiagnosticDirectives
    fn protocol_diagnostic_directives(
        directives: Vec<MappedDiagnosticDirective>,
        unused: Vec<UnusedExpectDirectiveDiagnostic>,
    ) -> Option<DiagnosticDirectives> {
        Some(DiagnosticDirectives {
            unused_expect_directive_diagnostics: unused,
            directives,
        })
    }

    impl MapperHandler for UnicodeMapper {
        // Go: host_test.go:96 unicodeMapper.HandleRequest
        fn handle_request(
            &self,
            method: &str,
            params: &JsonValue,
        ) -> std::result::Result<Option<Box<dyn AnyValue>>, GoError> {
            match method {
                METHOD_INITIALIZE => {
                    let p: InitializeParams = unmarshal_params(params)?;
                    let offered = p.position_encodings.contains(&self.encoding);
                    if !offered
                        && (self.encoding == PositionEncoding::UTF8
                            || self.encoding == PositionEncoding::UTF16)
                    {
                        return Err(errors::new(format!(
                            "position encoding {} was not offered",
                            gostd::strconv::quote(&self.encoding.0)
                        )));
                    }
                    let source = self.source.clone().unwrap_or_else(|| "mapper".to_string());
                    Ok(Some(Box::new(InitializeResult {
                        position_encoding: self.encoding.clone(),
                        diagnostic_source: source,
                    })))
                }
                METHOD_TRANSFORM => {
                    let p: TransformParams = unmarshal_params(params)?;
                    let (emoji_length, text_length) = if self.encoding == PositionEncoding::UTF8 {
                        (2, 3)
                    } else if self.encoding == PositionEncoding::UTF16 {
                        (1, 2)
                    } else {
                        return Ok(Some(Box::new(TransformResult {
                            mapped_output: MappedOutput {
                                text: p.content,
                                extension: ".ts".to_string(),
                                ..Default::default()
                            },
                            ..Default::default()
                        })));
                    };
                    let verbatim = spanmap::Kind::VERBATIM.0;
                    let mappings = json_marshal(
                        &vec![
                            vec![0, emoji_length, 0, emoji_length, verbatim],
                            vec![
                                emoji_length,
                                text_length - emoji_length,
                                emoji_length,
                                text_length - emoji_length,
                                verbatim,
                            ],
                        ],
                        &[],
                    )
                    .map_err(errors::from_value)?;
                    Ok(Some(Box::new(TransformResult {
                        mapped_output: MappedOutput {
                            text: p.content,
                            extension: ".ts".to_string(),
                            mappings: JsonValue(mappings.into_bytes()),
                            diagnostic_directives: protocol_diagnostic_directives(
                                vec![MappedDiagnosticDirective {
                                    original_start: emoji_length,
                                    original_length: text_length - emoji_length,
                                    virtual_start: emoji_length,
                                    virtual_end: text_length,
                                    policy: DiagnosticDirectivePolicy::IGNORE,
                                    unused_expect_directive_index: None,
                                }],
                                Vec::new(),
                            ),
                        },
                        diagnostics: vec![Diagnostic {
                            message_text: "after non-ASCII character".to_string(),
                            start: emoji_length,
                            length: text_length - emoji_length,
                            code: 1001,
                        }],
                        ..Default::default()
                    })))
                }
                _ => Err(unexpected_method(method)),
            }
        }
    }

    // Go: host_test.go:160 invalidDiagnosticMapper
    struct InvalidDiagnosticMapper {
        encoding: PositionEncoding,
    }

    impl MapperHandler for InvalidDiagnosticMapper {
        // Go: host_test.go:164 invalidDiagnosticMapper.HandleRequest
        fn handle_request(
            &self,
            method: &str,
            _params: &JsonValue,
        ) -> std::result::Result<Option<Box<dyn AnyValue>>, GoError> {
            match method {
                METHOD_INITIALIZE => Ok(Some(Box::new(InitializeResult {
                    position_encoding: self.encoding.clone(),
                    diagnostic_source: "mapper".to_string(),
                }))),
                METHOD_TRANSFORM => Ok(Some(Box::new(TransformResult {
                    mapped_output: MappedOutput {
                        extension: ".ts".to_string(),
                        ..Default::default()
                    },
                    diagnostics: vec![Diagnostic {
                        message_text: "invalid boundary".to_string(),
                        start: 1,
                        code: 1002,
                        ..Default::default()
                    }],
                    ..Default::default()
                }))),
                _ => Err(unexpected_method(method)),
            }
        }
    }

    // Go: host_test.go:192 fakeSpawner
    // fakeSpawner serves each spawn request with an in-process mapper over a net.Pipe, counting spawns so
    // tests can assert process consolidation. When handler is nil it serves a fakeMapper.
    #[derive(Default)]
    struct FakeSpawner {
        spawns: AtomicI32,
        closes: Arc<AtomicI32>,
        handler: Option<Arc<dyn MapperHandler>>,
    }

    fn fake_spawner(handler: Arc<dyn MapperHandler>) -> Rc<FakeSpawner> {
        Rc::new(FakeSpawner {
            handler: Some(handler),
            ..Default::default()
        })
    }

    impl Spawner for FakeSpawner {
        // Go: host_test.go:211 fakeSpawner.Spawn
        fn spawn(
            &self,
            _command: &[String],
            _dir: &str,
            _stderr: Option<Box<dyn Write + Send>>,
        ) -> std::result::Result<Arc<dyn ProcessExitState>, GoError> {
            self.spawns.fetch_add(1, Ordering::SeqCst);
            let handler: Arc<dyn MapperHandler> =
                self.handler.clone().unwrap_or_else(|| Arc::new(FakeMapper));
            let (client, server) = net_pipe();
            std::thread::spawn(move || {
                let server: Arc<dyn ipc::ReadWriteCloser> = server;
                let conn = ipc::new_async_conn(server, Rc::new(HandlerAdapter(handler)));
                let _ = conn.run(&background());
            });
            Ok(Arc::new(CountingReadWriteCloser {
                inner: client,
                closes: self.closes.clone(),
                once: Once::new(),
            }))
        }
    }

    // Go: host_test.go:225 countingReadWriteCloser
    struct CountingReadWriteCloser {
        inner: Arc<PipeEnd>,
        closes: Arc<AtomicI32>,
        once: Once,
    }

    impl ipc::ReadWriteCloser for CountingReadWriteCloser {
        fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(buf)
        }

        fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&self) -> std::io::Result<()> {
            self.inner.flush()
        }

        // Go: host_test.go:231 countingReadWriteCloser.Close
        fn close(&self) -> std::result::Result<(), GoError> {
            self.once.call_once(|| {
                self.closes.fetch_add(1, Ordering::SeqCst);
            });
            self.inner.close()
        }
    }

    impl ProcessExitState for CountingReadWriteCloser {}

    fn test_ctx() -> Context {
        background()
    }

    fn mapper(package: &str, name: &str, version: &str, exec: &str) -> Rc<Mapper> {
        Rc::new(Mapper {
            definition: Definition {
                package: package.to_string(),
                ..Default::default()
            },
            manifest: Manifest {
                name: name.to_string(),
                version: version.to_string(),
                exec: vec![exec.to_string()],
                ..Default::default()
            },
            ..Default::default()
        })
    }

    fn vue_mapper_with_extension() -> Rc<Mapper> {
        Rc::new(Mapper {
            definition: Definition {
                extensions: vec![".vue".to_string()],
                ..Default::default()
            },
            manifest: Manifest {
                name: "mapper".to_string(),
                exec: vec!["mapper".to_string()],
                ..Default::default()
            },
            ..Default::default()
        })
    }

    fn request(file_name: &str, content: &str) -> Request {
        Request {
            file_name: file_name.to_string(),
            content: content.to_string(),
        }
    }

    fn default_options() -> Option<Rc<CompilerOptions>> {
        Some(Rc::new(CompilerOptions::default()))
    }

    fn assert_error_contains<T>(result: std::result::Result<T, GoError>, text: &str) {
        match result {
            Ok(_) => panic!("expected an error containing {text:?}"),
            Err(err) => assert!(
                err.error().contains(text),
                "error {:?} does not contain {text:?}",
                err.error()
            ),
        }
    }

    // Go: host_test.go:236 TestRunnerTransform
    #[test]
    fn test_runner_transform() {
        let r = new_host(
            &test_ctx(),
            Rc::new(FakeSpawner::default()),
            locale::DEFAULT,
        );

        let mapper = Rc::new(Mapper {
            definition: Definition {
                extensions: vec![".vue".to_string()],
                ..Default::default()
            },
            manifest: Manifest {
                name: "vue".to_string(),
                version: "1.0.0".to_string(),
                exec: vec!["vue-mapper".to_string()],
                ..Default::default()
            },
            ..Default::default()
        });
        let result = r
            .transform(&mapper, request("/a.vue", "export const x = 1;"))
            .expect("transform");
        assert_eq!(result.text, "export const x = 1;");
        assert_eq!(result.virtual_extension, ".ts");
        assert!(result.mappings.is_some());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].code, 9999);
        assert_eq!(result.diagnostics[0].source(), "vue");
        let _ = r.close();
    }

    // Go: host_test.go:252 TestHostLogging
    #[test]
    fn test_host_logging() {
        let logs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let logger: Logger = {
            let logs = logs.clone();
            Arc::new(move |message: &str| lock(&logs).push(message.to_string()))
        };
        let spawner = SpawnerFunc(Box::new(
            |command: &[String], dir: &str, mut stderr: Option<Box<dyn Write + Send>>| {
                if let Some(stderr) = stderr.as_mut() {
                    let _ = stderr.write_all(b"mapper diagnostic\n");
                }
                FakeSpawner::default().spawn(command, dir, stderr)
            },
        ));
        let host = new_host_with_options(
            &test_ctx(),
            Rc::new(spawner),
            locale::DEFAULT,
            HostOptions {
                logger: Some(logger),
            },
        );
        let mapper = mapper("configured", "resolved", "1.0.0", "mapper");
        host.transform(&mapper, request("/a.vue", "export const x = 1;"))
            .expect("transform");

        let joined = lock(&logs).join("\n");
        assert!(joined.contains(
            r#"[content mapper: resolved] send: {"jsonrpc":"2.0","id":"api1","method":"initialize""#
        ));
        assert!(joined.contains(
            r#"[content mapper: resolved] receive: {"jsonrpc":"2.0","id":"api1","result":"#
        ));
        assert!(joined.contains("[content mapper: resolved] stderr: mapper diagnostic"));
        let _ = host.close();
    }

    // Go: host_test.go:282 TestHostDiscardsStderrWithoutLogging
    // PORT: Go compares `stderr` with `io.Discard`; the port passes `None`.
    #[test]
    fn test_host_discards_stderr_without_logging() {
        let spawner = SpawnerFunc(Box::new(
            |command: &[String], dir: &str, stderr: Option<Box<dyn Write + Send>>| {
                assert!(stderr.is_none());
                FakeSpawner::default().spawn(command, dir, stderr)
            },
        ));
        let host = new_host(&test_ctx(), Rc::new(spawner), locale::DEFAULT);
        let mapper = mapper("configured", "resolved", "1.0.0", "mapper");
        host.transform(&mapper, request("/a.vue", "export const x = 1;"))
            .expect("transform");
        let _ = host.close();
    }

    // Go: host_test.go:298 TestMapperDiagnosticName
    #[test]
    fn test_mapper_diagnostic_name() {
        let tests: [(Mapper, &str); 3] = [
            (
                Mapper {
                    definition: Definition {
                        package: "configured".to_string(),
                        ..Default::default()
                    },
                    manifest: Manifest {
                        name: "resolved".to_string(),
                        ..Default::default()
                    },
                    contribution_id: "contributed".to_string(),
                    ..Default::default()
                },
                "resolved",
            ),
            (
                Mapper {
                    definition: Definition {
                        package: "configured".to_string(),
                        ..Default::default()
                    },
                    contribution_id: "contributed".to_string(),
                    ..Default::default()
                },
                "configured",
            ),
            (
                Mapper {
                    contribution_id: "contributed".to_string(),
                    ..Default::default()
                },
                "contributed",
            ),
        ];
        for (mapper, want) in tests {
            assert_eq!(mapper.diagnostic_name(), want);
        }
    }

    // Go: host_test.go:313 TestRunnerTransformResponseValidation
    #[test]
    fn test_runner_transform_response_validation() {
        let mapper = vue_mapper_with_extension();

        // malformed result fails the request
        let host = new_host(
            &test_ctx(),
            fake_spawner(response_mapper(|_p| {
                Box::new(JsonValue(br#"{"text":1}"#.to_vec()))
            })),
            locale::DEFAULT,
        );
        let result = host.transform(&mapper, request("/a.vue", "a"));
        assert!(result.is_err());
        let _ = host.close();
    }

    // Go: host_test.go:401 exitedReadWriteCloser
    struct ExitedReadWriteCloser {
        inner: Arc<PipeEnd>,
        exit_code: i32,
    }

    impl ipc::ReadWriteCloser for ExitedReadWriteCloser {
        fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(buf)
        }

        fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&self) -> std::io::Result<()> {
            self.inner.flush()
        }

        fn close(&self) -> std::result::Result<(), GoError> {
            self.inner.close()
        }
    }

    impl ProcessExitState for ExitedReadWriteCloser {
        // Go: host_test.go:406 exitedReadWriteCloser.ExitCode
        fn exit_code(&self) -> (i32, bool) {
            (self.exit_code, true)
        }
    }

    // Go: host_test.go:410 exitOnCloseReadWriteCloser
    struct ExitOnCloseReadWriteCloser {
        inner: Arc<PipeEnd>,
        exited: AtomicBool,
    }

    impl ipc::ReadWriteCloser for ExitOnCloseReadWriteCloser {
        fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(buf)
        }

        fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&self) -> std::io::Result<()> {
            self.inner.flush()
        }

        // Go: host_test.go:415 exitOnCloseReadWriteCloser.Close
        fn close(&self) -> std::result::Result<(), GoError> {
            self.exited.store(true, Ordering::SeqCst);
            self.inner.close()
        }
    }

    impl ProcessExitState for ExitOnCloseReadWriteCloser {
        // Go: host_test.go:420 exitOnCloseReadWriteCloser.ExitCode
        fn exit_code(&self) -> (i32, bool) {
            (1, self.exited.load(Ordering::SeqCst))
        }
    }

    // Go: host_test.go:424 closeSignalReadWriteCloser
    struct CloseSignalReadWriteCloser {
        inner: Arc<PipeEnd>,
        closed: mpsc::SyncSender<()>,
        once: Once,
    }

    impl ipc::ReadWriteCloser for CloseSignalReadWriteCloser {
        fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.read(buf)
        }

        fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.write(buf)
        }

        fn flush(&self) -> std::io::Result<()> {
            self.inner.flush()
        }

        // Go: host_test.go:430 closeSignalReadWriteCloser.Close
        fn close(&self) -> std::result::Result<(), GoError> {
            let err = self.inner.close();
            self.once.call_once(|| {
                let _ = self.closed.try_send(());
            });
            err
        }
    }

    impl ProcessExitState for CloseSignalReadWriteCloser {}

    // Go: host_test.go:329 TestHostClosesProcessWhenReadLoopFails
    #[test]
    fn test_host_closes_process_when_read_loop_fails() {
        let (closed_tx, closed_rx) = mpsc::sync_channel::<()>(1);
        let spawner = SpawnerFunc(Box::new(
            move |_command: &[String], _dir: &str, _stderr: Option<Box<dyn Write + Send>>| {
                let (client, server) = net_pipe();
                std::thread::spawn(move || {
                    let transport: Arc<dyn ipc::ReadWriteCloser> = server.clone();
                    let mut protocol = ipc::new_jsonrpc_protocol(transport);
                    let message = protocol.read_message().expect("read initialize");
                    assert_eq!(message.method, METHOD_INITIALIZE);
                    protocol
                        .write_response(message.id.as_ref(), Some(initialize_result("mapper")))
                        .expect("write initialize response");
                    server.write(b"oops\n").expect("write garbage");
                    let _ = server.close();
                });
                let process: Arc<dyn ProcessExitState> = Arc::new(CloseSignalReadWriteCloser {
                    inner: client,
                    closed: closed_tx.clone(),
                    once: Once::new(),
                });
                Ok(process)
            },
        ));
        let host = new_host(&test_ctx(), Rc::new(spawner), locale::DEFAULT);
        let mapper = mapper("", "mapper", "", "mapper");
        let result = host.transform(&mapper, request("/a.vue", ""));
        assert!(result.is_err());
        let process_closed = closed_rx.recv_timeout(Duration::from_secs(1)).is_ok();
        assert!(
            process_closed,
            "mapper process was not closed after its read loop failed"
        );
        let _ = host.close();
    }

    // Go: host_test.go:364 TestHostReportsInitializationTimeoutBeforeClosingProcess
    // PORT: Go's mapper waits for the test context. This one reads until the
    // host closes its end, which it does at the initialize deadline.
    #[test]
    fn test_host_reports_initialization_timeout_before_closing_process() {
        let spawner = SpawnerFunc(Box::new(
            |_command: &[String], _dir: &str, _stderr: Option<Box<dyn Write + Send>>| {
                let (client, server) = net_pipe();
                std::thread::spawn(move || {
                    let transport: Arc<dyn ipc::ReadWriteCloser> = server.clone();
                    let _ = ipc::new_jsonrpc_protocol(transport).read_message();
                    let mut buf = [0u8; 256];
                    while matches!(server.read(&mut buf), Ok(n) if n > 0) {}
                    let _ = server.close();
                });
                let process: Arc<dyn ProcessExitState> = Arc::new(ExitOnCloseReadWriteCloser {
                    inner: client,
                    exited: AtomicBool::new(false),
                });
                Ok(process)
            },
        ));
        let host = new_host(&test_ctx(), Rc::new(spawner), locale::DEFAULT);
        let mapper = mapper("", "mapper", "", "mapper");
        let err = host
            .transform(&mapper, request("/a.vue", ""))
            .err()
            .expect("transform error");
        let initialize_error = errors::as_type::<InitializeError>(&err);
        assert!(
            initialize_error.is_some(),
            "expected InitializeError, got {err:?}"
        );
        assert_eq!(
            initialize_error.unwrap().kind,
            InitializeErrorKind::NO_RESPONSE
        );
        let _ = host.close();
    }

    // Go: host_test.go:384 TestHostReportsProcessExitBeforeInitialization
    #[test]
    fn test_host_reports_process_exit_before_initialization() {
        let spawner = SpawnerFunc(Box::new(
            |_command: &[String], _dir: &str, _stderr: Option<Box<dyn Write + Send>>| {
                let (client, server) = net_pipe();
                server.close().expect("close server");
                let process: Arc<dyn ProcessExitState> = Arc::new(ExitedReadWriteCloser {
                    inner: client,
                    exit_code: 42,
                });
                Ok(process)
            },
        ));
        let host = new_host(&test_ctx(), Rc::new(spawner), locale::DEFAULT);
        let mapper = mapper("", "mapper", "", "mapper");
        let err = host
            .transform(&mapper, request("/a.vue", ""))
            .err()
            .expect("transform error");
        let initialize_error = errors::as_type::<InitializeError>(&err)
            .unwrap_or_else(|| panic!("expected InitializeError, got {err:?}"));
        assert_eq!(initialize_error.kind, InitializeErrorKind::PROCESS_EXIT);
        assert_eq!(initialize_error.exit_code, 42);
        let _ = host.close();
    }

    // Go: host_test.go:436 TestRunnerTransformDiagnosticDirectives
    #[test]
    fn test_runner_transform_diagnostic_directives() {
        let mapper = vue_mapper_with_extension();
        let transform = |output: MappedOutput| -> std::result::Result<Result, GoError> {
            let host = new_host(
                &test_ctx(),
                fake_spawner(response_mapper(move |_p| {
                    Box::new(TransformResult {
                        mapped_output: output.clone(),
                        ..Default::default()
                    })
                })),
                locale::DEFAULT,
            );
            let result = host.transform(&mapper, request("/a.vue", "directive\nsource"));
            let _ = host.close();
            result
        };

        let result = transform(MappedOutput {
            text: "virtual source".to_string(),
            extension: ".ts".to_string(),
            diagnostic_directives: protocol_diagnostic_directives(
                vec![MappedDiagnosticDirective {
                    original_start: 0,
                    original_length: 9,
                    virtual_start: 8,
                    virtual_end: 14,
                    policy: DiagnosticDirectivePolicy::EXPECT,
                    unused_expect_directive_index: None,
                }],
                vec![UnusedExpectDirectiveDiagnostic {
                    code: 2578,
                    message_text: "Unused framework directive.".to_string(),
                }],
            ),
            ..Default::default()
        })
        .expect("transform");
        assert_eq!(result.diagnostic_directives.len(), 1);
        let directive = &result.diagnostic_directives[0];
        assert_eq!(directive.original_range.pos(), 0);
        assert_eq!(directive.original_range.end(), 9);
        assert_eq!(directive.virtual_range.pos(), 8);
        assert_eq!(directive.virtual_range.end(), 14);
        assert_eq!(
            directive.policy,
            ast::MappedDiagnosticDirectivePolicy::EXPECT
        );
        assert_eq!(directive.unused_code, 2578);
        assert_eq!(directive.unused_message_text, "Unused framework directive.");
        assert_eq!(directive.source, "mapper");
        let result = transform(MappedOutput {
            text: "virtual source".to_string(),
            extension: ".ts".to_string(),
            diagnostic_directives: protocol_diagnostic_directives(
                vec![MappedDiagnosticDirective {
                    original_length: 9,
                    virtual_start: 8,
                    virtual_end: 14,
                    policy: DiagnosticDirectivePolicy::EXPECT,
                    unused_expect_directive_index: Some(1),
                    ..Default::default()
                }],
                vec![
                    UnusedExpectDirectiveDiagnostic {
                        code: 1,
                        message_text: "first".to_string(),
                    },
                    UnusedExpectDirectiveDiagnostic {
                        code: 2,
                        message_text: "second".to_string(),
                    },
                ],
            ),
            ..Default::default()
        })
        .expect("transform");
        assert_eq!(result.diagnostic_directives[0].unused_code, 2);
        assert_eq!(
            result.diagnostic_directives[0].unused_message_text,
            "second"
        );
        transform(MappedOutput {
            text: "x".to_string(),
            extension: ".ts".to_string(),
            diagnostic_directives: protocol_diagnostic_directives(
                vec![MappedDiagnosticDirective {
                    original_start: -1,
                    policy: DiagnosticDirectivePolicy::IGNORE,
                    ..Default::default()
                }],
                vec![UnusedExpectDirectiveDiagnostic::default()],
            ),
            ..Default::default()
        })
        .expect("transform");

        let invalid: Vec<(
            &str,
            &str,
            Vec<MappedDiagnosticDirective>,
            DiagnosticDirectiveErrorKind,
        )> = vec![
            (
                "invalid range",
                "x",
                vec![MappedDiagnosticDirective {
                    virtual_start: -1,
                    policy: DiagnosticDirectivePolicy::IGNORE,
                    ..Default::default()
                }],
                DiagnosticDirectiveErrorKind::INVALID_RANGE,
            ),
            (
                "unknown policy",
                "x",
                vec![MappedDiagnosticDirective {
                    policy: DiagnosticDirectivePolicy(2),
                    ..Default::default()
                }],
                DiagnosticDirectiveErrorKind::INVALID_POLICY,
            ),
            (
                "expect requires unused diagnostic",
                "x",
                vec![MappedDiagnosticDirective {
                    policy: DiagnosticDirectivePolicy::EXPECT,
                    ..Default::default()
                }],
                DiagnosticDirectiveErrorKind::EXPECT_MISSING_UNUSED_DIAGNOSTIC,
            ),
            (
                "multiple unused diagnostics require index",
                "x",
                vec![MappedDiagnosticDirective {
                    policy: DiagnosticDirectivePolicy::EXPECT,
                    ..Default::default()
                }],
                DiagnosticDirectiveErrorKind::EXPECT_MISSING_UNUSED_DIAGNOSTIC,
            ),
            (
                "original range out of bounds",
                "x",
                vec![MappedDiagnosticDirective {
                    original_start: 99,
                    policy: DiagnosticDirectivePolicy::EXPECT,
                    ..Default::default()
                }],
                DiagnosticDirectiveErrorKind::INVALID_RANGE,
            ),
            (
                "virtual range out of bounds",
                "x",
                vec![MappedDiagnosticDirective {
                    virtual_start: 99,
                    policy: DiagnosticDirectivePolicy::IGNORE,
                    ..Default::default()
                }],
                DiagnosticDirectiveErrorKind::INVALID_RANGE,
            ),
            (
                "overlap",
                "abc",
                vec![
                    MappedDiagnosticDirective {
                        virtual_end: 2,
                        policy: DiagnosticDirectivePolicy::IGNORE,
                        ..Default::default()
                    },
                    MappedDiagnosticDirective {
                        virtual_start: 1,
                        virtual_end: 3,
                        policy: DiagnosticDirectivePolicy::IGNORE,
                        ..Default::default()
                    },
                ],
                DiagnosticDirectiveErrorKind::OVERLAP,
            ),
        ];
        for (name, text, directives, kind) in invalid {
            let mut diagnostic_directives =
                protocol_diagnostic_directives(directives, Vec::new()).expect("directives");
            if name == "original range out of bounds" {
                diagnostic_directives.unused_expect_directive_diagnostics =
                    vec![UnusedExpectDirectiveDiagnostic::default()];
            } else if name == "multiple unused diagnostics require index" {
                diagnostic_directives.unused_expect_directive_diagnostics = vec![
                    UnusedExpectDirectiveDiagnostic::default(),
                    UnusedExpectDirectiveDiagnostic::default(),
                ];
            }
            let err = transform(MappedOutput {
                text: text.to_string(),
                extension: ".ts".to_string(),
                diagnostic_directives: Some(diagnostic_directives),
                ..Default::default()
            })
            .err()
            .unwrap_or_else(|| panic!("{name}: expected an error"));
            let directive_error = errors::as_type::<DiagnosticDirectiveError>(&err)
                .unwrap_or_else(|| panic!("{name}: expected DiagnosticDirectiveError"));
            assert_eq!(directive_error.kind, kind, "{name}");
        }
    }

    // Go: host_test.go:579 TestMappedDiagnosticDirectiveJSON
    #[test]
    fn test_mapped_diagnostic_directive_json() {
        let tests: [(&str, MappedDiagnosticDirective, &str); 2] = [
            (
                "ignore",
                MappedDiagnosticDirective {
                    virtual_start: 8,
                    virtual_end: 14,
                    original_start: 0,
                    original_length: 9,
                    policy: DiagnosticDirectivePolicy::IGNORE,
                    unused_expect_directive_index: None,
                },
                "[0,9,8,14,0]",
            ),
            (
                "expect",
                MappedDiagnosticDirective {
                    virtual_start: 8,
                    virtual_end: 14,
                    original_start: 0,
                    original_length: 9,
                    policy: DiagnosticDirectivePolicy::EXPECT,
                    unused_expect_directive_index: None,
                },
                "[0,9,8,14,1]",
            ),
        ];
        for (name, directive, want) in tests {
            let data = json_marshal(&directive, &[]).expect("marshal");
            assert_eq!(data, want, "{name}");
            let mut decoded = MappedDiagnosticDirective::default();
            json_unmarshal(data.as_bytes(), &mut decoded, &[]).expect("unmarshal");
            assert_eq!(decoded, directive, "{name}");
        }

        for data in ["[0,0,0,0]", "[0,0,0,0,0,1,2]"] {
            let mut directive = MappedDiagnosticDirective::default();
            let err =
                json_unmarshal(data.as_bytes(), &mut directive, &[]).expect_err("want an error");
            assert!(
                err.to_string().contains("diagnostic directive tuple"),
                "{err}"
            );
        }

        let diagnostic_directives = DiagnosticDirectives {
            unused_expect_directive_diagnostics: vec![
                UnusedExpectDirectiveDiagnostic {
                    code: 1,
                    message_text: "first".to_string(),
                },
                UnusedExpectDirectiveDiagnostic {
                    code: 2,
                    message_text: "second".to_string(),
                },
            ],
            directives: vec![MappedDiagnosticDirective {
                original_start: 2,
                original_length: 3,
                virtual_start: 5,
                virtual_end: 9,
                policy: DiagnosticDirectivePolicy::EXPECT,
                unused_expect_directive_index: Some(1),
            }],
        };
        let data = json_marshal(&diagnostic_directives, &[]).expect("marshal");
        assert_eq!(
            data,
            r#"{"unusedExpectDirectiveDiagnostics":[{"code":1,"messageText":"first"},{"code":2,"messageText":"second"}],"directives":[[2,3,5,9,1,1]]}"#
        );
        let mut decoded = DiagnosticDirectives::default();
        json_unmarshal(data.as_bytes(), &mut decoded, &[]).expect("unmarshal");
        assert_eq!(decoded, diagnostic_directives);
    }

    // Go: host_test.go:646 TestRunnerTransformSupplementalOutputs
    #[test]
    fn test_runner_transform_supplemental_outputs() {
        let host = new_host(
            &test_ctx(),
            fake_spawner(response_mapper(|_p| {
                Box::new(TransformResult {
                    mapped_output: MappedOutput {
                        text: "export default 1;".to_string(),
                        extension: ".ts".to_string(),
                        ..Default::default()
                    },
                    supplemental: vec![
                        SupplementalOutput {
                            mapped_output: MappedOutput {
                                text: "declare const first: string;".to_string(),
                                extension: ".ts".to_string(),
                                diagnostic_directives: protocol_diagnostic_directives(
                                    vec![MappedDiagnosticDirective {
                                        virtual_end: 7,
                                        policy: DiagnosticDirectivePolicy::IGNORE,
                                        ..Default::default()
                                    }],
                                    Vec::new(),
                                ),
                                ..Default::default()
                            },
                        },
                        SupplementalOutput {
                            mapped_output: MappedOutput {
                                text: "declare const second: number;".to_string(),
                                extension: ".mjs".to_string(),
                                ..Default::default()
                            },
                        },
                    ],
                    ..Default::default()
                })
            })),
            locale::DEFAULT,
        );
        let mapper = vue_mapper_with_extension();
        let result = host
            .transform(&mapper, request("/component.vue", "component"))
            .expect("transform");
        assert_eq!(result.supplemental.len(), 2);
        assert_eq!(result.supplemental[0].text, "declare const first: string;");
        assert_eq!(result.supplemental[0].virtual_extension, ".ts");
        assert!(result.supplemental[0].mappings.is_some());
        assert_eq!(result.supplemental[0].diagnostic_directives.len(), 1);
        assert_eq!(
            result.supplemental[0].diagnostic_directives[0]
                .virtual_range
                .end(),
            7
        );
        assert_eq!(result.supplemental[1].virtual_extension, ".mjs");
        assert!(result.supplemental[1].mappings.is_some());
        let _ = host.close();
    }

    // Go: host_test.go:677 TestRunnerTransformInvalidSupplementalDiagnosticDirective
    #[test]
    fn test_runner_transform_invalid_supplemental_diagnostic_directive() {
        let host = new_host(
            &test_ctx(),
            fake_spawner(response_mapper(|_p| {
                Box::new(TransformResult {
                    mapped_output: MappedOutput {
                        text: "export {};".to_string(),
                        extension: ".ts".to_string(),
                        ..Default::default()
                    },
                    supplemental: vec![
                        SupplementalOutput {
                            mapped_output: MappedOutput {
                                text: "export {};".to_string(),
                                extension: ".ts".to_string(),
                                ..Default::default()
                            },
                        },
                        SupplementalOutput {
                            mapped_output: MappedOutput {
                                text: "export {};".to_string(),
                                extension: ".ts".to_string(),
                                diagnostic_directives: protocol_diagnostic_directives(
                                    vec![MappedDiagnosticDirective {
                                        policy: DiagnosticDirectivePolicy::EXPECT,
                                        ..Default::default()
                                    }],
                                    Vec::new(),
                                ),
                                ..Default::default()
                            },
                        },
                    ],
                    ..Default::default()
                })
            })),
            locale::DEFAULT,
        );
        let mapper = vue_mapper_with_extension();
        let err = host
            .transform(&mapper, request("/component.vue", "component"))
            .err()
            .expect("transform error");
        let directive_error =
            errors::as_type::<DiagnosticDirectiveError>(&err).expect("DiagnosticDirectiveError");
        assert_eq!(
            directive_error.kind,
            DiagnosticDirectiveErrorKind::EXPECT_MISSING_UNUSED_DIAGNOSTIC
        );
        assert_eq!(directive_error.index, 0);
        assert_eq!(directive_error.supplemental_index, 1);
        let _ = host.close();
    }

    // Go: host_test.go:703 TestRunnerRejectsInvalidVirtualExtension
    #[test]
    fn test_runner_rejects_invalid_virtual_extension() {
        for supplemental in [false, true] {
            for extension in ["", ".coffee"] {
                let host = new_host(
                    &test_ctx(),
                    fake_spawner(response_mapper(move |_p| {
                        let mut canonical_extension = extension.to_string();
                        let mut supplemental_outputs: Vec<SupplementalOutput> = Vec::new();
                        if supplemental {
                            canonical_extension = ".ts".to_string();
                            supplemental_outputs = vec![SupplementalOutput {
                                mapped_output: MappedOutput {
                                    text: "export {};".to_string(),
                                    extension: extension.to_string(),
                                    ..Default::default()
                                },
                            }];
                        }
                        Box::new(TransformResult {
                            mapped_output: MappedOutput {
                                text: "export {};".to_string(),
                                extension: canonical_extension,
                                ..Default::default()
                            },
                            supplemental: supplemental_outputs,
                            ..Default::default()
                        })
                    })),
                    locale::DEFAULT,
                );
                let mapper = vue_mapper_with_extension();
                assert_error_contains(
                    host.transform(&mapper, request("/component.vue", "component")),
                    "invalid virtual extension",
                );
                let _ = host.close();
            }
        }
    }

    // Go: host_test.go:730 TestRunnerPositionEncodings
    #[test]
    fn test_runner_position_encodings() {
        for encoding in [PositionEncoding::UTF8, PositionEncoding::UTF16] {
            let r = new_host(
                &test_ctx(),
                fake_spawner(Arc::new(UnicodeMapper {
                    encoding: encoding.clone(),
                    source: None,
                })),
                locale::DEFAULT,
            );
            let mapper = mapper("", &encoding.0, "", "mapper");
            let result = r
                .transform(&mapper, request("/a.vue", "éx"))
                .expect("transform");
            let mappings = result.mappings.as_deref();
            let segments = spanmap::SpanMap::segments(mappings);
            assert_eq!(segments.len(), 2);
            assert_eq!(segments[0].virtual_end, 2);
            assert_eq!(segments[0].original_end, 2);
            assert_eq!(segments[1].virtual_start, 2);
            assert_eq!(segments[1].original_start, 2);
            assert_eq!(result.text, "éx");
            let problem = spanmap::SpanMap::validate(mappings, &result.text, "éx");
            assert!(problem.is_none(), "{problem:?}");
            let (mapped, fidelity) = spanmap::SpanMap::virtual_to_original_position(mappings, 2);
            assert_eq!(mapped, 2);
            assert_eq!(fidelity, spanmap::Fidelity::EXACT);
            assert_eq!(result.diagnostics[0].pos, 2);
            assert_eq!(result.diagnostics[0].end, 3);
            assert_eq!(result.diagnostic_directives[0].original_range.pos(), 2);
            assert_eq!(result.diagnostic_directives[0].original_range.end(), 3);
            assert_eq!(result.diagnostic_directives[0].virtual_range.pos(), 2);
            assert_eq!(result.diagnostic_directives[0].virtual_range.end(), 3);
            let _ = r.close();
        }
    }

    // Go: host_test.go:765 TestRunnerRejectsUnsupportedPositionEncoding
    #[test]
    fn test_runner_rejects_unsupported_position_encoding() {
        let r = new_host(
            &test_ctx(),
            fake_spawner(Arc::new(UnicodeMapper {
                encoding: PositionEncoding(Cow::Borrowed("utf-32")),
                source: None,
            })),
            locale::DEFAULT,
        );
        let mapper = mapper("", "invalid", "", "mapper");
        assert_error_contains(
            r.transform(&mapper, request("/a.vue", "x")),
            "unsupported position encoding",
        );
        let _ = r.close();
    }

    // Go: host_test.go:774 TestRunnerRejectsInvalidDiagnosticSource
    #[test]
    fn test_runner_rejects_invalid_diagnostic_source() {
        for source in [
            "",
            " ",
            "ts",
            "TS",
            "d.ts",
            "json",
            "typescript",
            "TypeScript",
            "tsc",
            "TSC",
        ] {
            let handler = UnicodeMapper {
                encoding: PositionEncoding::UTF8,
                source: Some(source.to_string()),
            };
            let r = new_host(
                &test_ctx(),
                fake_spawner(Arc::new(handler)),
                locale::DEFAULT,
            );
            let mapper = mapper("", "invalid", "", "mapper");
            let result = r.transform(&mapper, request("/a.vue", "x"));
            if source.trim().is_empty() {
                assert_error_contains(result, "diagnostic source must not be empty");
            } else {
                assert_error_contains(result, "is reserved by TypeScript");
            }
            let _ = r.close();
        }
    }

    // Go: host_test.go:793 TestRunnerRejectsPositionsInsideUnicodeCharacters
    #[test]
    fn test_runner_rejects_positions_inside_unicode_characters() {
        for (encoding, content) in [
            (PositionEncoding::UTF8, "é"),
            (PositionEncoding::UTF16, "😀"),
        ] {
            let r = new_host(
                &test_ctx(),
                fake_spawner(Arc::new(InvalidDiagnosticMapper {
                    encoding: encoding.clone(),
                })),
                locale::DEFAULT,
            );
            let mapper = mapper("", &encoding.0, "", "mapper");
            assert_error_contains(
                r.transform(&mapper, request("/a.vue", content)),
                "splits a Unicode code point",
            );
            let _ = r.close();
        }
    }

    // Go: host_test.go:813 TestRunnerConsolidatesByIdentity
    #[test]
    fn test_runner_consolidates_by_identity() {
        let spawner = Rc::new(FakeSpawner::default());
        let r = new_host(&test_ctx(), spawner.clone(), locale::DEFAULT);

        // Two logically-separate mappers with the same identity share one process.
        let vue_a = mapper("a", "vue", "1.0.0", "vue-mapper");
        let vue_b = mapper("b", "vue", "1.0.0", "vue-mapper");
        let svelte = mapper("", "svelte", "2.0.0", "svelte-mapper");
        let project = r
            .project(ProjectSpec {
                mappers: vec![vue_a.clone(), vue_b.clone(), svelte.clone()],
                compiler_options: default_options(),
                ..Default::default()
            })
            .expect("project");

        for m in [&vue_a, &vue_b, &vue_a, &svelte] {
            project.transform(m, request("/x", "y")).expect("transform");
        }
        assert_eq!(
            spawner.spawns.load(Ordering::SeqCst),
            2,
            "expected one process per identity"
        );
        let _ = project.close();
        let _ = r.close();
    }

    // Go: host_test.go:833 TestRunnerLeaseLifecycle
    #[test]
    fn test_runner_lease_lifecycle() {
        let spawner = Rc::new(FakeSpawner::default());
        let r = new_host(&test_ctx(), spawner.clone(), locale::DEFAULT);

        let vue_a = mapper("a", "vue", "1.0.0", "vue-mapper");
        let vue_b = mapper("b", "vue", "1.0.0", "vue-mapper");
        let svelte = mapper("", "svelte", "2.0.0", "svelte-mapper");

        let release_vue_a = r.acquire(&[vue_a.clone(), vue_a.clone()]);
        let release_vue_b = r.acquire(&[vue_b]);
        let release_svelte = r.acquire(&[svelte.clone()]);
        for mapper in [&vue_a, &svelte] {
            r.transform(mapper, request("/x", "y")).expect("transform");
        }
        assert_eq!(spawner.spawns.load(Ordering::SeqCst), 2);

        release_vue_a();
        assert_eq!(
            spawner.closes.load(Ordering::SeqCst),
            0,
            "shared vue process should remain owned"
        );
        release_svelte();
        assert_eq!(
            spawner.closes.load(Ordering::SeqCst),
            1,
            "final release should close the process"
        );
        release_vue_b();
        release_vue_b();
        assert_eq!(
            spawner.closes.load(Ordering::SeqCst),
            2,
            "final vue owner should close once"
        );

        let release_new = r.acquire(&[vue_a.clone()]);
        r.transform(&vue_a, request("/x", "y")).expect("transform");
        assert_eq!(
            spawner.spawns.load(Ordering::SeqCst),
            3,
            "reacquiring should spawn a fresh process lazily"
        );
        release_new();
        assert_eq!(spawner.closes.load(Ordering::SeqCst), 3);
        let _ = r.close();
    }

    // Go: host_test.go:869 recordingMapper
    // recordingMapper captures project configuration and lifecycle requests for host protocol tests.
    #[derive(Default)]
    struct RecordingState {
        received: String,
        received_options: String,
        received_locale: String,
        project_handles: Vec<String>,
        closed_handles: Vec<String>,
        transform_handle: String,
        transform_params: String,
    }

    // PORT: Go keeps every field under `mu`; the test configuration fields
    // are set once, before the mapper runs, so only the recorded state is
    // behind the mutex here. Go nil `watchedFiles` and `configIdentity` are
    // `None`.
    #[derive(Default)]
    struct RecordingMapper {
        state: Mutex<RecordingState>,
        watched_files: Option<Vec<String>>,
        config_identity: Option<String>,
        dynamic_config: bool,
        option_diagnostics: Vec<OptionDiagnosticResult>,
    }

    impl MapperHandler for RecordingMapper {
        // Go: host_test.go:890 recordingMapper.handlesProjects
        fn handles_projects(&self) -> bool {
            true
        }

        // Go: host_test.go:900 recordingMapper.HandleRequest
        fn handle_request(
            &self,
            method: &str,
            params: &JsonValue,
        ) -> std::result::Result<Option<Box<dyn AnyValue>>, GoError> {
            match method {
                METHOD_INITIALIZE => {
                    let p: InitializeParams = unmarshal_params(params)?;
                    lock(&self.state).received_locale = p.locale;
                    Ok(Some(initialize_result("mapper")))
                }
                METHOD_OPEN_PROJECT => {
                    let p: OpenProjectParams = unmarshal_params(params)?;
                    let raw = json_marshal(&p.compiler_options, &[]).map_err(errors::from_value)?;
                    {
                        let mut state = lock(&self.state);
                        state.project_handles.push(p.project_handle.clone());
                        state.received = raw;
                        state.received_options = String::from_utf8_lossy(&p.options.0).into_owned();
                    }
                    let mut watched_files = self.watched_files.clone();
                    if !self.dynamic_config
                        && watched_files.is_none()
                        && self.config_identity.is_none()
                        && self.option_diagnostics.is_empty()
                    {
                        return Ok(Some(Box::new(OpenProjectResult::default())));
                    }
                    if self.dynamic_config && watched_files.is_none() {
                        watched_files = Some(vec![tspath::combine_paths(
                            &tspath::get_directory_path(&p.config_file_name),
                            &["mapper.config.js"],
                        )]);
                    }
                    let mut config_identity = String::new();
                    if self.dynamic_config {
                        config_identity =
                            format!("config:{}", String::from_utf8_lossy(&p.options.0));
                    }
                    if let Some(identity) = &self.config_identity {
                        config_identity = identity.clone();
                    }
                    Ok(Some(Box::new(OpenProjectResult {
                        config_identity,
                        watched_files: watched_files.unwrap_or_default(),
                        option_diagnostics: self.option_diagnostics.clone(),
                    })))
                }
                METHOD_CLOSE_PROJECT => {
                    let p: CloseProjectParams = unmarshal_params(params)?;
                    lock(&self.state).closed_handles.push(p.project_handle);
                    Ok(None)
                }
                METHOD_TRANSFORM => {
                    let p: TransformParams = unmarshal_params(params)?;
                    {
                        let mut state = lock(&self.state);
                        state.transform_handle = p.project_handle.clone();
                        state.transform_params = String::from_utf8_lossy(&params.0).into_owned();
                    }
                    Ok(Some(Box::new(TransformResult {
                        mapped_output: MappedOutput {
                            text: p.content,
                            extension: ".ts".to_string(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })))
                }
                _ => Err(unexpected_method(method)),
            }
        }
    }

    /// Forwards to a shared `RecordingMapper`, so the test keeps a handle.
    struct SharedRecordingMapper(Arc<RecordingMapper>);

    impl MapperHandler for SharedRecordingMapper {
        fn handles_projects(&self) -> bool {
            self.0.handles_projects()
        }

        fn handle_request(
            &self,
            method: &str,
            params: &JsonValue,
        ) -> std::result::Result<Option<Box<dyn AnyValue>>, GoError> {
            self.0.handle_request(method, params)
        }
    }

    fn recording_spawner(mapper: &Arc<RecordingMapper>) -> Rc<FakeSpawner> {
        fake_spawner(Arc::new(SharedRecordingMapper(mapper.clone())))
    }

    fn dynamic_mapper(options: &str, compiler_options: &[&str]) -> Rc<Mapper> {
        Rc::new(Mapper {
            definition: Definition {
                options: JsonValue(options.as_bytes().to_vec()),
                ..Default::default()
            },
            manifest: Manifest {
                name: "dynamic".to_string(),
                version: "1.0.0".to_string(),
                exec: vec!["mapper".to_string()],
                compiler_options: compiler_options.iter().map(|s| (*s).to_string()).collect(),
                dynamic_config: true,
            },
            ..Default::default()
        })
    }

    fn project_spec(
        config_file_name: &str,
        mappers: &[&Rc<Mapper>],
        compiler_options: &Option<Rc<CompilerOptions>>,
    ) -> ProjectSpec {
        ProjectSpec {
            config_file_name: config_file_name.to_string(),
            mappers: mappers.iter().map(|m| Rc::clone(m)).collect(),
            compiler_options: compiler_options.clone(),
        }
    }

    // Go: host_test.go:971 TestProjectLifecycle
    #[test]
    fn test_project_lifecycle() {
        let mapper_process = Arc::new(RecordingMapper {
            dynamic_config: true,
            ..Default::default()
        });
        let spawner = recording_spawner(&mapper_process);
        let host = new_host(&test_ctx(), spawner.clone(), locale::DEFAULT);

        let static_mapper = Rc::new(Mapper {
            definition: Definition {
                options: JsonValue(br#"{"mode":"static"}"#.to_vec()),
                ..Default::default()
            },
            manifest: Manifest {
                name: "static".to_string(),
                version: "1.0.0".to_string(),
                exec: vec!["mapper".to_string()],
                ..Default::default()
            },
            ..Default::default()
        });
        let static_project = host
            .project(project_spec(
                "/repo/tsconfig.json",
                &[&static_mapper],
                &default_options(),
            ))
            .expect("project");
        assert_eq!(
            spawner.spawns.load(Ordering::SeqCst),
            0,
            "static identity should not spawn the mapper"
        );
        let static_identities = static_project.identities().expect("identities");
        assert_eq!(static_identities.len(), 1);
        static_project.close().expect("close");

        let dynamic_a = dynamic_mapper(r#"{"mode":"a"}"#, &["jsx"]);
        let dynamic_b = dynamic_mapper(r#"{"mode":"b"}"#, &[]);
        let dynamic_a_options = default_options();
        let project_a = host
            .project(project_spec(
                "/repo/a/tsconfig.json",
                &[&dynamic_a, &dynamic_b],
                &dynamic_a_options,
            ))
            .expect("project");
        let project_a_reversed = host
            .project(project_spec(
                "/repo/reversed/tsconfig.json",
                &[&dynamic_b, &dynamic_a],
                &dynamic_a_options,
            ))
            .expect("project");
        let project_different_options = host
            .project(project_spec(
                "/repo/options/tsconfig.json",
                &[&dynamic_a],
                &Some(Rc::new(CompilerOptions {
                    jsx: JsxEmit::REACT,
                    ..Default::default()
                })),
            ))
            .expect("project");
        let project_b = host
            .project(project_spec(
                "/repo/b/tsconfig.json",
                &[&dynamic_a],
                &default_options(),
            ))
            .expect("project");
        let project_a_again = host
            .project(project_spec(
                "/repo/a/tsconfig.json",
                &[&dynamic_a, &dynamic_b],
                &dynamic_a_options,
            ))
            .expect("project");
        assert_eq!(
            spawner.spawns.load(Ordering::SeqCst),
            0,
            "getting dynamic projects should not start the mapper"
        );
        let project_a_identities = project_a.identities().expect("identities");
        let project_a_reversed_identities = project_a_reversed.identities().expect("identities");
        let project_different_option_identities =
            project_different_options.identities().expect("identities");
        let project_b_identities = project_b.identities().expect("identities");
        assert_eq!(project_a_identities.len(), 2);
        assert_eq!(project_a_reversed_identities.len(), 2);
        assert_eq!(project_a_identities[0], project_a_reversed_identities[1]);
        assert_eq!(project_a_identities[1], project_a_reversed_identities[0]);
        assert_ne!(
            project_a_identities[0],
            project_different_option_identities[0]
        );
        assert_eq!(project_b_identities.len(), 1);
        assert_eq!(
            spawner.spawns.load(Ordering::SeqCst),
            1,
            "dynamic projects should share one mapper process"
        );
        let project_a_watched_files = project_a.watched_files().expect("watched files");
        let project_b_watched_files = project_b.watched_files().expect("watched files");
        assert_eq!(project_a_watched_files.len(), 1);
        assert_eq!(project_b_watched_files.len(), 1);

        project_a
            .transform(&dynamic_b, request("/repo/a/file.ext", "x"))
            .expect("transform");
        {
            let state = lock(&mapper_process.state);
            assert!(state.project_handles[..2].contains(&state.transform_handle));
        }

        project_a_again.close().expect("close");
        project_a.close().expect("close");
        project_a_reversed.close().expect("close");
        project_different_options.close().expect("close");
        project_b.close().expect("close");
        let timings = host.timings();
        let dynamic_timings = timings.mappers[&dynamic_a.identity()];
        assert_eq!(dynamic_timings.spawn.count, 1);
        assert_eq!(dynamic_timings.initialize.count, 1);
        assert_eq!(dynamic_timings.open_project.count, 6);
        assert_eq!(dynamic_timings.transform.count, 1);
        assert_eq!(dynamic_timings.close_project.count, 6);
        let state = lock(&mapper_process.state);
        assert_eq!(state.project_handles.len(), 6);
        assert_eq!(state.closed_handles.len(), 6);
        drop(state);
        let _ = host.close();
    }

    // Go: host_test.go:1074 TestProjectMethodsAfterHostClose
    #[test]
    fn test_project_methods_after_host_close() {
        let mapper_process = Arc::new(RecordingMapper {
            dynamic_config: true,
            ..Default::default()
        });
        let host = new_host(
            &test_ctx(),
            recording_spawner(&mapper_process),
            locale::DEFAULT,
        );
        let mapper = dynamic_mapper("", &[]);
        let project = host
            .project(project_spec(
                "/repo/tsconfig.json",
                &[&mapper],
                &default_options(),
            ))
            .expect("project");
        project
            .transform(&mapper, request("/repo/file.ext", "x"))
            .expect("transform");
        let identity = project
            .identity(&Rc::new(Mapper::default()))
            .expect("identity");
        assert_eq!(identity, "");
        host.close().expect("close");

        project.refresh().expect("refresh");
        let identities = project.identities().expect("identities");
        assert_eq!(identities.len(), 0);
        let identity = project.identity(&mapper).expect("identity");
        assert_eq!(identity, "");
        let identity = project
            .identity(&Rc::new(Mapper::default()))
            .expect("identity");
        assert_eq!(identity, "");
        let watched_files = project.watched_files().expect("watched files");
        assert_eq!(watched_files.len(), 0);
        assert_eq!(project.diagnostics().len(), 0);
        assert_error_contains(
            project.transform(&mapper, request("/repo/file.ext", "x")),
            "content mapper project is closed",
        );
        // Go: defer project.Close()
        let _ = project.close();
    }

    /// The project error inside a transform error (Go
    /// `errors.AsType[*TransformError]` then `errors.AsType[*ProjectError]`).
    fn project_error_kind(err: &GoError) -> ProjectErrorKind {
        assert!(
            errors::as_type::<TransformError>(err).is_some(),
            "expected TransformError, got {err:?}"
        );
        errors::as_type::<ProjectError>(err)
            .unwrap_or_else(|| panic!("expected ProjectError, got {err:?}"))
            .kind
    }

    // Go: host_test.go:1112 TestProjectRejectsRelativeWatchedFiles
    #[test]
    fn test_project_rejects_relative_watched_files() {
        let mapper_process = Arc::new(RecordingMapper {
            watched_files: Some(vec!["mapper.config.js".to_string()]),
            dynamic_config: true,
            ..Default::default()
        });
        let host = new_host(
            &test_ctx(),
            recording_spawner(&mapper_process),
            locale::DEFAULT,
        );

        let project_mapper = Rc::new(Mapper {
            definition: Definition {
                package: "dynamic".to_string(),
                ..Default::default()
            },
            ..(*dynamic_mapper("", &[])).clone()
        });
        let project = host
            .project(project_spec(
                "/repo/tsconfig.json",
                &[&project_mapper],
                &default_options(),
            ))
            .expect("project");
        let err = project
            .transform(&project_mapper, request("/repo/file.ext", "x"))
            .err()
            .expect("transform error");
        assert_eq!(
            project_error_kind(&err),
            ProjectErrorKind::NON_ABSOLUTE_WATCHED_FILE
        );
        let _ = host.close();
    }

    // Go: host_test.go:1135 TestDynamicProjectRequiresConfigIdentity
    #[test]
    fn test_dynamic_project_requires_config_identity() {
        let mapper_process = Arc::new(RecordingMapper {
            config_identity: Some(String::new()),
            dynamic_config: true,
            ..Default::default()
        });
        let host = new_host(
            &test_ctx(),
            recording_spawner(&mapper_process),
            locale::DEFAULT,
        );

        let project_mapper = Rc::new(Mapper {
            definition: Definition {
                package: "dynamic".to_string(),
                ..Default::default()
            },
            ..(*dynamic_mapper("", &[])).clone()
        });
        let project = host
            .project(project_spec(
                "/repo/tsconfig.json",
                &[&project_mapper],
                &default_options(),
            ))
            .expect("project");
        let err = project
            .transform(&project_mapper, request("/repo/file.ext", "x"))
            .err()
            .expect("transform error");
        assert_eq!(
            project_error_kind(&err),
            ProjectErrorKind::MISSING_CONFIG_IDENTITY
        );
        let _ = host.close();
    }

    // Go: host_test.go:1159 TestStaticMapperRejectsDynamicProjectResponseFields
    #[test]
    fn test_static_mapper_rejects_dynamic_project_response_fields() {
        let tests: [(&str, RecordingMapper, ProjectErrorKind); 2] = [
            (
                "config identity",
                RecordingMapper {
                    config_identity: Some("dynamic".to_string()),
                    ..Default::default()
                },
                ProjectErrorKind::UNEXPECTED_CONFIG_IDENTITY,
            ),
            (
                "watched files",
                RecordingMapper {
                    watched_files: Some(vec!["/repo/mapper.config.js".to_string()]),
                    ..Default::default()
                },
                ProjectErrorKind::UNEXPECTED_WATCHED_FILES,
            ),
        ];
        for (name, mapper_process, kind) in tests {
            let host = new_host(
                &test_ctx(),
                recording_spawner(&Arc::new(mapper_process)),
                locale::DEFAULT,
            );
            let project_mapper = mapper("", "static", "1.0.0", "mapper");
            let project = host
                .project(project_spec(
                    "/repo/tsconfig.json",
                    &[&project_mapper],
                    &default_options(),
                ))
                .expect("project");
            let err = project
                .transform(&project_mapper, request("/repo/file.ext", "x"))
                .err()
                .unwrap_or_else(|| panic!("{name}: expected an error"));
            assert_eq!(project_error_kind(&err), kind, "{name}");
            let _ = project.close();
            let _ = host.close();
        }
    }

    // Go: host_test.go:1191 TestProjectRejectsInvalidOptionDiagnosticPath
    #[test]
    fn test_project_rejects_invalid_option_diagnostic_path() {
        let mapper_process = Arc::new(RecordingMapper {
            option_diagnostics: vec![OptionDiagnosticResult {
                path: vec![JsonValue(b"null".to_vec())],
                message_text: "Invalid option.".to_string(),
                code: 123,
            }],
            ..Default::default()
        });
        let host = new_host(
            &test_ctx(),
            recording_spawner(&mapper_process),
            locale::DEFAULT,
        );
        let project_mapper = mapper("", "mapper", "1.0.0", "mapper");
        let project = host
            .project(ProjectSpec {
                mappers: vec![project_mapper.clone()],
                compiler_options: default_options(),
                ..Default::default()
            })
            .expect("project");
        let err = project
            .transform(&project_mapper, request("/repo/file.ext", "x"))
            .err()
            .expect("transform error");
        assert_eq!(
            project_error_kind(&err),
            ProjectErrorKind::MALFORMED_RESPONSE
        );
        let _ = project.close();
        let _ = host.close();
    }

    // Go: host_test.go:1215 TestRunnerForwardsProjectOptions
    #[test]
    fn test_runner_forwards_project_options() {
        let mapper_process = Arc::new(RecordingMapper::default());
        let (diagnostic_locale, ok) = locale::parse("cs-CZ");
        assert!(ok);
        let r = new_host(
            &test_ctx(),
            recording_spawner(&mapper_process),
            diagnostic_locale,
        );

        let mapper_definition = Rc::new(Mapper {
            definition: Definition {
                options: JsonValue(br#"{"strictTemplates":true}"#.to_vec()),
                ..Default::default()
            },
            manifest: Manifest {
                name: "vue".to_string(),
                version: "1.0.0".to_string(),
                exec: vec!["vue-mapper".to_string()],
                compiler_options: vec!["target".to_string(), "jsx".to_string()],
                ..Default::default()
            },
            ..Default::default()
        });
        let compiler_options = Rc::new(CompilerOptions {
            target: ScriptTarget::ES2020,
            strict: Tristate::True,
            ..Default::default()
        });
        let project = r
            .project(ProjectSpec {
                mappers: vec![mapper_definition.clone()],
                compiler_options: Some(compiler_options.clone()),
                ..Default::default()
            })
            .expect("project");
        project
            .transform(&mapper_definition, request("/a.vue", "x"))
            .expect("transform");

        let want = json_marshal(&CompilerOptionsJSON(&compiler_options), &[]).expect("marshal");
        {
            let state = lock(&mapper_process.state);
            assert_eq!(state.received, want);
            assert_eq!(state.received_options, r#"{"strictTemplates":true}"#);
            assert_eq!(state.received_locale, "cs-CZ");
            assert!(!state.transform_params.contains(r#""options""#));
            assert!(!state.transform_params.contains(r#""compilerOptions""#));
        }
        let _ = project.close();
        let _ = r.close();
    }

    // Go: host_test.go:1247 TestHostSetLocaleRestartsMapper
    #[test]
    fn test_host_set_locale_restarts_mapper() {
        let mapper_process = Arc::new(RecordingMapper::default());
        let spawner = recording_spawner(&mapper_process);
        let r = new_host(&test_ctx(), spawner.clone(), locale::DEFAULT);

        let definition = mapper("", "vue", "1.0.0", "vue-mapper");
        let release = r.acquire(&[definition.clone()]);

        r.transform(&definition, request("/a.vue", "x"))
            .expect("transform");
        assert_eq!(spawner.spawns.load(Ordering::SeqCst), 1);

        let (french, ok) = locale::parse("fr");
        assert!(ok);
        r.set_locale(french);
        assert_eq!(spawner.closes.load(Ordering::SeqCst), 1);

        r.transform(&definition, request("/a.vue", "x"))
            .expect("transform");
        assert_eq!(spawner.spawns.load(Ordering::SeqCst), 2);
        assert_eq!(lock(&mapper_process.state).received_locale, "fr");
        // Go: defer release()
        release();
        let _ = r.close();
    }

    // Go: host_test.go:1275 TestHostSetLocaleWaitsForTransform
    // PORT: Go runs the transform and SetLocale on two goroutines and checks
    // that SetLocale waits for the transform. The port's host is
    // dispatch-thread state, so a transform and SetLocale cannot overlap;
    // this checks what remains: SetLocale after the transform closes the
    // mapper process once.
    #[test]
    fn test_host_set_locale_waits_for_transform() {
        let mapper_process = Arc::new(RecordingMapper::default());
        let spawner = recording_spawner(&mapper_process);
        let r = new_host(&test_ctx(), spawner.clone(), locale::DEFAULT);
        let definition = mapper("", "vue", "1.0.0", "vue-mapper");
        let release = r.acquire(&[definition.clone()]);

        r.transform(&definition, request("/a.vue", "x"))
            .expect("transform");

        let (french, ok) = locale::parse("fr");
        assert!(ok);
        r.set_locale(french);
        assert_eq!(spawner.closes.load(Ordering::SeqCst), 1);
        release();
        let _ = r.close();
    }
}
