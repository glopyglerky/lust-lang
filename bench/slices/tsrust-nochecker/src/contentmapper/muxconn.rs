//! A content mapper connection that several threads can call at once.
//!
//! PORT: Go `ipc.AsyncConn` is already concurrent: its `Run` goroutine
//! reads, and each `Call` waits for its own response. The ipc port reads
//! each response inside `call` on the dispatch thread (ipc/conn_async.rs),
//! so a mapper got one request at a time, while the parse goroutines of Go
//! keep many in flight. This connection reads on its own thread, as `Run`
//! does, and routes each response to the call that waits for it, so the
//! parse workers can transform content-mapped files while the loader works
//! (`ConcurrentTransform`). It has the parts of `AsyncConn` that the mapper
//! host uses: the handler of a request from the mapper runs on its own
//! thread (Go `handlers.Go`), and the server timing requests get the answer
//! of a connection that collects no timing.

use crate::core::{};
use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::gostd::{GoError, context, errors};
use crate::ipc::{self, };
use crate::jsonrpc;
use crate::rustc_hash::FxHashMap;
use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;


/// How often a waiting call looks at its context: a response wakes it at
/// once, a cancelled context within this time.
const CONTEXT_POLL: Duration = Duration::from_millis(20);







#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::gostd::context;
    use crate::ipc::{Conn, ReadWriteCloser};
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    /// The client end of a socket pair. `close` shuts it down, which ends a
    /// blocked read.
    struct End(UnixStream);

    impl ReadWriteCloser for End {
        fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            (&self.0).read(buf)
        }

        fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
            (&self.0).write(buf)
        }

        fn flush(&self) -> std::io::Result<()> {
            (&self.0).flush()
        }

        fn close(&self) -> Result<(), GoError> {
            let _ = self.0.shutdown(std::net::Shutdown::Both);
            Ok(())
        }
    }

    /// Go `rejectHandler` (contentmapper/hostimpl.go:1364).
    struct Reject;

    impl ipc::Handler for Reject {
        fn handle_request(
            &self,
            _ctx: &Context,
            method: &str,
            _params: JsonValue,
        ) -> Result<Option<Box<dyn AnyValue>>, GoError> {
            Err(errors::new(format!(
                "content mapper sent an unexpected request: {method}"
            )))
        }

        fn handle_notification(
            &self,
            _ctx: &Context,
            _method: &str,
            _params: JsonValue,
        ) -> Result<(), GoError> {
            Ok(())
        }
    }

    fn connect() -> (Arc<MuxConn>, UnixStream) {
        connect_with(Arc::new(Reject))
    }

    fn connect_with(handler: Arc<dyn ipc::Handler + Send + Sync>) -> (Arc<MuxConn>, UnixStream) {
        let (client, server) = UnixStream::pair().expect("socket pair");
        let client: Arc<dyn ReadWriteCloser> = Arc::new(End(client));
        let conn = MuxConn::start(
            Some(client.clone()),
            Arc::new(move || {
                Box::new(ipc::new_jsonrpc_protocol(client.clone())) as Box<dyn ipc::Protocol>
            }),
            handler,
        );
        (conn, server)
    }

    /// Reads one message with its `Content-Length` header.
    fn read_framed(stream: &UnixStream) -> String {
        let mut header = Vec::new();
        let mut byte = [0u8; 1];
        while !header.ends_with(b"\r\n\r\n") {
            (&*stream).read_exact(&mut byte).expect("header byte");
            header.push(byte[0]);
        }
        let length: usize = String::from_utf8(header)
            .expect("utf-8 header")
            .trim()
            .strip_prefix("Content-Length: ")
            .and_then(|length| length.parse().ok())
            .expect("Content-Length header");
        let mut body = vec![0u8; length];
        (&*stream).read_exact(&mut body).expect("body");
        String::from_utf8(body).expect("utf-8 body")
    }

    fn write_framed(stream: &UnixStream, body: &str) {
        write!(&*stream, "Content-Length: {}\r\n\r\n{body}", body.len()).expect("write");
    }

    /// The value of `"key":"…"` in a message.
    fn string_field(message: &str, key: &str) -> String {
        let start = message.find(&format!("\"{key}\":\"")).expect(key) + key.len() + 4;
        message[start..].split('"').next().unwrap().to_string()
    }

    // Several threads call at once; the peer answers in the reverse order.
    #[test]
    fn concurrent_calls_get_their_own_responses() {
        const CALLS: i32 = 8;
        let (conn, server) = connect();
        let peer = std::thread::spawn(move || {
            let requests: Vec<String> = (0..CALLS).map(|_| read_framed(&server)).collect();
            for request in requests.iter().rev() {
                let id = string_field(request, "id");
                let params = &request[request.find("\"params\":").unwrap() + 9..];
                let n = params.trim_end_matches('}');
                write_framed(
                    &server,
                    &format!(r#"{{"jsonrpc":"2.0","id":"{id}","result":{n}}}"#),
                );
            }
            server
        });
        let callers: Vec<_> = (0..CALLS)
            .map(|n| {
                let conn = conn.clone();
                std::thread::spawn(move || {
                    let result = conn
                        .call(&context::background(), "echo", Some(Box::new(n)))
                        .expect("the call returns");
                    assert_eq!(result.0, n.to_string().as_bytes());
                })
            })
            .collect();
        for caller in callers {
            caller.join().expect("caller thread");
        }
        drop(peer.join().expect("peer thread"));
    }

    #[test]
    fn call_returns_when_peer_closes() {
        let (conn, server) = connect();
        let peer = std::thread::spawn(move || {
            read_framed(&server);
            drop(server);
        });
        let err = conn
            .call(&context::background(), "transform", None)
            .expect_err("the call fails when the peer closes");
        peer.join().expect("peer thread");
        assert!(errors::is(&err, &ERR_CONN_CLOSED), "{}", err.error());
        // Later calls return the same error at once.
        let again = conn
            .call(&context::background(), "transform", None)
            .expect_err("a later call fails");
        assert!(errors::is(&again, &ERR_CONN_CLOSED), "{}", again.error());
    }

    // An error response fails only its own call, with Go's text; the
    // connection stays open.
    #[test]
    fn error_response_fails_only_its_call() {
        let (conn, server) = connect();
        let peer = std::thread::spawn(move || {
            let first = read_framed(&server);
            write_framed(
                &server,
                &format!(
                    r#"{{"jsonrpc":"2.0","id":"{}","error":{{"code":-32603,"message":"boom"}}}}"#,
                    string_field(&first, "id")
                ),
            );
            let second = read_framed(&server);
            write_framed(
                &server,
                &format!(
                    r#"{{"jsonrpc":"2.0","id":"{}","result":2}}"#,
                    string_field(&second, "id")
                ),
            );
            server
        });
        let err = conn
            .call(&context::background(), "transform", None)
            .expect_err("the error response fails the call");
        assert_eq!(err.error(), "ipc: remote error [-32603]: boom");
        let result = conn
            .call(&context::background(), "transform", None)
            .expect("the next call returns");
        assert_eq!(result.0, b"2");
        drop(peer.join().expect("peer thread"));
    }

    // A call whose context ends while it waits returns the context's error.
    // The late response is dropped, and the next call gets its own.
    fn context_end_leaves_the_connection_open(
        make: fn() -> (Context, Option<context::CancelFunc>),
        want: &GoError,
    ) {
        let (conn, server) = connect();
        let (read, request_read) = mpsc::channel();
        let (late, answer_late) = mpsc::channel::<()>();
        let peer = std::thread::spawn(move || {
            let first = read_framed(&server);
            read.send(()).unwrap();
            answer_late.recv().unwrap();
            write_framed(
                &server,
                &format!(
                    r#"{{"jsonrpc":"2.0","id":"{}","result":1}}"#,
                    string_field(&first, "id")
                ),
            );
            let second = read_framed(&server);
            write_framed(
                &server,
                &format!(
                    r#"{{"jsonrpc":"2.0","id":"{}","result":2}}"#,
                    string_field(&second, "id")
                ),
            );
            server
        });
        let (ctx, cancel) = make();
        let waiting = {
            let conn = conn.clone();
            std::thread::spawn(move || conn.call(&ctx, "transform", None))
        };
        request_read.recv().unwrap();
        if let Some(cancel) = cancel {
            cancel();
        }
        let err = waiting
            .join()
            .expect("caller thread")
            .expect_err("the call ends with its context");
        assert!(errors::is(&err, want), "{}", err.error());
        late.send(()).unwrap();
        let result = conn
            .call(&context::background(), "transform", None)
            .expect("the next call returns");
        assert_eq!(result.0, b"2");
        drop(peer.join().expect("peer thread"));
    }

    #[test]
    fn cancelled_call_leaves_the_connection_open() {
        context_end_leaves_the_connection_open(
            || {
                let (ctx, cancel) = context::with_cancel(&context::background());
                (ctx, Some(cancel))
            },
            &context::CANCELED,
        );
    }

    // The peer does not answer before the deadline (it waits for the end of
    // the call), so the call times out.
    #[test]
    fn call_times_out_at_its_deadline() {
        context_end_leaves_the_connection_open(
            || {
                let (ctx, _cancel) =
                    context::with_timeout(&context::background(), Duration::from_millis(1));
                (ctx, None)
            },
            &context::DEADLINE_EXCEEDED,
        );
    }

    // The mapper protocol has no requests from the mapper: the handler
    // answers one with an error, on its own thread, while the read loop
    // goes on.
    #[test]
    fn request_from_the_peer_is_rejected() {
        let (conn, server) = connect();
        let peer = std::thread::spawn(move || {
            let call = read_framed(&server);
            write_framed(
                &server,
                r#"{"jsonrpc":"2.0","id":"m1","method":"readFile"}"#,
            );
            let rejection = read_framed(&server);
            let id = string_field(&call, "id");
            write_framed(
                &server,
                &format!(r#"{{"jsonrpc":"2.0","id":"{id}","result":true}}"#),
            );
            rejection
        });
        let result = conn
            .call(&context::background(), "transform", None)
            .expect("the call returns");
        let rejection = peer.join().expect("peer thread");
        assert_eq!(result.0, b"true");
        assert!(
            rejection.contains(r#""id":"m1""#)
                && rejection.contains("content mapper sent an unexpected request: readFile"),
            "{rejection}"
        );
    }

    /// A handler that panics.
    struct Panics;

    impl ipc::Handler for Panics {
        fn handle_request(
            &self,
            _ctx: &Context,
            _method: &str,
            _params: JsonValue,
        ) -> Result<Option<Box<dyn AnyValue>>, GoError> {
            panic!("handler panic");
        }

        fn handle_notification(
            &self,
            _ctx: &Context,
            _method: &str,
            _params: JsonValue,
        ) -> Result<(), GoError> {
            Ok(())
        }
    }

    // Go recovers a panic of the handler and answers the request with it
    // (ipc/conn_async.go:206-223); the read loop goes on. With no answer,
    // the peer's read times out and the call fails.
    #[test]
    fn handler_panic_is_answered_with_an_error() {
        let (conn, server) = connect_with(Arc::new(Panics));
        server
            .set_read_timeout(Some(Duration::from_secs(30)))
            .expect("read timeout");
        let peer = std::thread::spawn(move || {
            let call = read_framed(&server);
            write_framed(
                &server,
                r#"{"jsonrpc":"2.0","id":"m1","method":"readFile"}"#,
            );
            let answer = read_framed(&server);
            let id = string_field(&call, "id");
            write_framed(
                &server,
                &format!(r#"{{"jsonrpc":"2.0","id":"{id}","result":true}}"#),
            );
            answer
        });
        let result = conn
            .call(&context::background(), "transform", None)
            .expect("the call returns");
        let answer = peer.join().expect("peer thread");
        assert_eq!(result.0, b"true");
        assert!(
            answer.contains(r#""id":"m1""#)
                && answer.contains(r#""code":-32603"#)
                && answer.contains(r#""message":"panic: handler panic\n"#),
            "{answer}"
        );
    }

    /// The JSON-RPC protocol with error answers that fail: the first one
    /// panics (`panic_once`), or each one returns an error (`refuse`).
    struct FaultyErrorAnswers {
        inner: Box<dyn ipc::Protocol>,
        panic_once: Arc<std::sync::atomic::AtomicBool>,
        refuse: bool,
    }

    impl ipc::Protocol for FaultyErrorAnswers {
        fn read_message(&mut self) -> Result<Message, GoError> {
            self.inner.read_message()
        }

        fn write_request(
            &mut self,
            id: Option<&jsonrpc::ID>,
            method: &str,
            params: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            self.inner.write_request(id, method, params)
        }

        fn write_notification(
            &mut self,
            method: &str,
            params: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            self.inner.write_notification(method, params)
        }

        fn write_response(
            &mut self,
            id: Option<&jsonrpc::ID>,
            result: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            self.inner.write_response(id, result)
        }

        fn write_error(
            &mut self,
            id: Option<&jsonrpc::ID>,
            err: &jsonrpc::ResponseError,
        ) -> Result<(), GoError> {
            if self.refuse {
                return Err(errors::new("write refused"));
            }
            if self.panic_once.swap(false, Ordering::SeqCst) {
                panic!("write panic");
            }
            self.inner.write_error(id, err)
        }
    }

    /// `connect_with` over `FaultyErrorAnswers`.
    fn connect_faulty(
        handler: Arc<dyn ipc::Handler + Send + Sync>,
        panic_once: bool,
        refuse: bool,
    ) -> (Arc<MuxConn>, UnixStream) {
        let (client, server) = UnixStream::pair().expect("socket pair");
        let client: Arc<dyn ReadWriteCloser> = Arc::new(End(client));
        let panic_once = Arc::new(std::sync::atomic::AtomicBool::new(panic_once));
        let conn = MuxConn::start(
            Some(client.clone()),
            Arc::new(move || {
                Box::new(FaultyErrorAnswers {
                    inner: Box::new(ipc::new_jsonrpc_protocol(client.clone())),
                    panic_once: panic_once.clone(),
                    refuse,
                }) as Box<dyn ipc::Protocol>
            }),
            handler,
        );
        (conn, server)
    }

    // Go's recover also covers the write of the answer
    // (ipc/conn_async.go:207-247). When that write panics, the request gets
    // the panic as its answer, and the write lock that the panic poisoned
    // still works for later calls.
    #[test]
    fn answer_write_panic_is_answered_with_an_error() {
        let (conn, server) = connect_faulty(Arc::new(Reject), true, false);
        server
            .set_read_timeout(Some(Duration::from_secs(30)))
            .expect("read timeout");
        let peer = std::thread::spawn(move || {
            let call = read_framed(&server);
            write_framed(
                &server,
                r#"{"jsonrpc":"2.0","id":"m1","method":"readFile"}"#,
            );
            let answer = read_framed(&server);
            let id = string_field(&call, "id");
            write_framed(
                &server,
                &format!(r#"{{"jsonrpc":"2.0","id":"{id}","result":true}}"#),
            );
            let call = read_framed(&server);
            let id = string_field(&call, "id");
            write_framed(
                &server,
                &format!(r#"{{"jsonrpc":"2.0","id":"{id}","result":false}}"#),
            );
            answer
        });
        let result = conn
            .call(&context::background(), "transform", None)
            .expect("the call returns");
        assert_eq!(result.0, b"true");
        let result = conn
            .call(&context::background(), "transform", None)
            .expect("a later call returns");
        assert_eq!(result.0, b"false");
        let answer = peer.join().expect("peer thread");
        assert!(
            answer.contains(r#""id":"m1""#)
                && answer.contains(r#""code":-32603"#)
                && answer.contains(r#""message":"panic: write panic\n"#),
            "{answer}"
        );
    }

    // When the panic answer cannot be written, Go's handleRequest returns
    // "ipc: failed to write panic error response: ... (original panic: ...)"
    // (ipc/conn_async.go:220), and the read loop ends the pending calls
    // with it (recordRequestError, :123). A call that nothing ends fails at
    // its 30 s deadline.
    #[test]
    fn a_failed_panic_answer_ends_the_calls() {
        let (conn, server) = connect_faulty(Arc::new(Panics), false, true);
        let peer = std::thread::spawn(move || {
            read_framed(&server);
            write_framed(
                &server,
                r#"{"jsonrpc":"2.0","id":"m1","method":"readFile"}"#,
            );
            server
        });
        let (ctx, _cancel) = context::with_timeout(&context::background(), Duration::from_secs(30));
        let err = conn
            .call(&ctx, "transform", None)
            .expect_err("the call ends with the request error");
        drop(peer.join().expect("peer thread"));
        let text = err.error();
        assert!(errors::is(&err, &ERR_CONN_CLOSED), "{text}");
        assert!(
            text.contains(
                "ipc: failed to write panic error response: write refused \
                 (original panic: handler panic)"
            ),
            "{text}"
        );
    }

    // After a failed panic answer Go closes the transport at once
    // (ipc/conn_async.go:99-105), with no call in flight: the peer reads
    // EOF, and a later call fails with the request error.
    #[test]
    fn a_failed_panic_answer_closes_the_transport() {
        let (conn, server) = connect_faulty(Arc::new(Panics), false, true);
        server
            .set_read_timeout(Some(Duration::from_secs(30)))
            .expect("read timeout");
        write_framed(
            &server,
            r#"{"jsonrpc":"2.0","id":"m1","method":"readFile"}"#,
        );
        let mut byte = [0u8; 1];
        let read = (&server).read(&mut byte).expect("EOF, not the timeout");
        assert_eq!(read, 0, "the peer got a message");
        let err = conn
            .call(&context::background(), "transform", None)
            .expect_err("a call after the close fails");
        let text = err.error();
        assert!(errors::is(&err, &ERR_CONN_CLOSED), "{text}");
        assert!(
            text.contains("ipc: failed to write panic error response: write refused"),
            "{text}"
        );
    }

    /// The JSON-RPC protocol with error answers that panic, with a Go
    /// panic value (`go_panic`) or a plain Rust panic.
    struct PanickingErrorAnswers {
        inner: Box<dyn ipc::Protocol>,
        go_value: bool,
    }

    impl ipc::Protocol for PanickingErrorAnswers {
        fn read_message(&mut self) -> Result<Message, GoError> {
            self.inner.read_message()
        }

        fn write_request(
            &mut self,
            id: Option<&jsonrpc::ID>,
            method: &str,
            params: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            self.inner.write_request(id, method, params)
        }

        fn write_notification(
            &mut self,
            method: &str,
            params: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            self.inner.write_notification(method, params)
        }

        fn write_response(
            &mut self,
            id: Option<&jsonrpc::ID>,
            result: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            self.inner.write_response(id, result)
        }

        fn write_error(
            &mut self,
            _id: Option<&jsonrpc::ID>,
            _err: &jsonrpc::ResponseError,
        ) -> Result<(), GoError> {
            if self.go_value {
                crate::core::go_panic("write panic".to_string());
            }
            panic!("write panic");
        }
    }

    /// Set in the child processes of `a_panic_in_the_panic_answer_ends_the_process`:
    /// "go" or "rust", the kind of panic of the panic answer.
    const CRASH_CHILD_ENV: &str = "GOPORT_MUXCONN_CRASH_CHILD";
    const CRASH_CHILD_TEST: &str =
        "contentmapper::muxconn::tests::a_panic_in_the_panic_answer_ends_the_process";

    // Go recovers a panic of the handler, but a panic in the write of that
    // panic answer (ipc/conn_async.go:207-221) is not recovered. It ends
    // the process: `WaitGroup.Go` (go1.27.1 sync/waitgroup.go:236) panics
    // again with it, and the runtime exits 2. A Go panic ends the port the
    // same way; any other panic is a port gap and exits `EXIT_UNPORTED`
    // (`lsp::server::go_crash`). Each case runs in a child process.
    #[test]
    fn a_panic_in_the_panic_answer_ends_the_process() {
        if let Ok(kind) = std::env::var(CRASH_CHILD_ENV) {
            let (client, server) = UnixStream::pair().expect("socket pair");
            let client: Arc<dyn ReadWriteCloser> = Arc::new(End(client));
            let go_value = kind == "go";
            let conn = MuxConn::start(
                Some(client.clone()),
                Arc::new(move || {
                    Box::new(PanickingErrorAnswers {
                        inner: Box::new(ipc::new_jsonrpc_protocol(client.clone())),
                        go_value,
                    }) as Box<dyn ipc::Protocol>
                }),
                Arc::new(Panics),
            );
            write_framed(
                &server,
                r#"{"jsonrpc":"2.0","id":"m1","method":"readFile"}"#,
            );
            let (ctx, _cancel) =
                context::with_timeout(&context::background(), Duration::from_secs(30));
            let result = conn.call(&ctx, "transform", None);
            panic!(
                "the process did not end: {:?}",
                result.map_err(|err| err.error())
            );
        }
        // `/proc/self/exe` still names this binary when a build replaces it.
        let proc_exe = std::path::Path::new("/proc/self/exe");
        let exe = if proc_exe.exists() {
            proc_exe.to_path_buf()
        } else {
            std::env::current_exe().expect("test binary")
        };
        for (kind, code, text) in [
            ("go", 2, "panic: write panic [recovered, repanicked]"),
            ("rust", crate::execute::tsc::EXIT_UNPORTED, "write panic"),
        ] {
            let output = std::process::Command::new(&exe)
                .args([
                    "--exact",
                    CRASH_CHILD_TEST,
                    "--nocapture",
                    "--test-threads",
                    "1",
                ])
                .env(CRASH_CHILD_ENV, kind)
                .stdin(std::process::Stdio::null())
                .output()
                .expect("run the child test");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.code(), Some(code), "{kind}:\n{stderr}");
            assert!(stderr.contains(text), "{kind}:\n{stderr}");
            assert!(
                !stderr.contains("the process did not end"),
                "{kind}:\n{stderr}"
            );
        }
    }

    // Go answers the server timing request before the handler.
    #[test]
    fn server_timing_request_is_answered() {
        let (conn, server) = connect();
        let peer = std::thread::spawn(move || {
            let call = read_framed(&server);
            write_framed(
                &server,
                r#"{"jsonrpc":"2.0","id":"m1","method":"getServerTiming"}"#,
            );
            let answer = read_framed(&server);
            let id = string_field(&call, "id");
            write_framed(
                &server,
                &format!(r#"{{"jsonrpc":"2.0","id":"{id}","result":true}}"#),
            );
            answer
        });
        conn.call(&context::background(), "transform", None)
            .expect("the call returns");
        let answer = peer.join().expect("peer thread");
        assert!(
            answer.contains(r#""id":"m1""#) && answer.contains(r#""result":{"#),
            "{answer}"
        );
    }

    /// A protocol whose read panics.
    struct PanicOnRead;

    impl ipc::Protocol for PanicOnRead {
        fn read_message(&mut self) -> Result<Message, GoError> {
            panic!("read panic");
        }

        fn write_request(
            &mut self,
            _id: Option<&jsonrpc::ID>,
            _method: &str,
            _params: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            Ok(())
        }

        fn write_notification(
            &mut self,
            _method: &str,
            _params: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            Ok(())
        }

        fn write_response(
            &mut self,
            _id: Option<&jsonrpc::ID>,
            _result: Option<Box<dyn AnyValue>>,
        ) -> Result<(), GoError> {
            Ok(())
        }

        fn write_error(
            &mut self,
            _id: Option<&jsonrpc::ID>,
            _err: &jsonrpc::ResponseError,
        ) -> Result<(), GoError> {
            Ok(())
        }
    }

    // A panic of the read loop ends the calls, and its payload waits for
    // the thread that resumes it.
    #[test]
    fn read_panic_ends_the_calls() {
        let conn = MuxConn::start(None, Arc::new(|| Box::new(PanicOnRead)), Arc::new(Reject));
        let err = conn
            .call(&context::background(), "transform", None)
            .expect_err("the call ends");
        assert!(errors::is(&err, &ERR_CONN_CLOSED), "{}", err.error());
        assert!(conn.read_panicked());
        let payload = conn.take_read_panic().expect("the payload");
        assert_eq!(payload.downcast_ref::<&str>(), Some(&"read panic"));
        assert!(conn.take_read_panic().is_none());
    }
}
