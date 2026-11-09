//! Port of internal/api/server.go.

use crate::api::prelude::*;

use crate::api::callbackfs::{};
use crate::contentmapper;
use crate::frontend::bundled;
use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::frontend::vfs::{self, Fs};
use crate::gostd::{Context, GoError, errors};
use crate::ipc::{
    Transport, 
    new_stdio_transport, 
};
use crate::lsp::lsproto;
use crate::project;
use std::cell::Cell;
use std::io::{Read, Write};
use std::time::Duration;

// Go: server.go:17 StdioServerOptions
// StdioServerOptions configures the STDIO-based API server.
// PORT: Go `io.ReadCloser` / `io.WriteCloser` / `io.Writer` are boxed
// `Read` / `Write` values; a nil one is `None`. `In` and `Async` are Rust
// keywords, so the fields are `in_` and `async_`. The Go
// `contentmapper.Spawner` interface is `Rc<dyn contentmapper::Spawner>`,
// and a nil one is `None` (tsgo#4712).
#[derive(Default)]
pub struct StdioServerOptions {
    pub in_: Option<Box<dyn Read + Send>>,
    pub out: Option<Box<dyn Write + Send>>,
    pub err: Option<Box<dyn Write + Send>>,
    pub cwd: String,
    pub default_library_path: String,
    // PipePath, if set, listens on a named pipe (Windows) or Unix domain
    // socket instead of using In/Out for communication.
    pub pipe_path: String,
    // Callbacks specifies which filesystem operations should be delegated
    // to the client (e.g., "readFile", "fileExists"). Empty means no callbacks.
    pub callbacks: Vec<String>,
    // Async enables JSON-RPC protocol with async connection handling.
    // When false (default), uses MessagePack protocol with sync connection.
    pub async_: bool,
    // CollectTiming enables per-request server processing-time measurement.
    // When enabled, the server accumulates each request's processing time into
    // running totals and a recent-request ring buffer. Response messages are
    // left unchanged; the client folds this data into its own timing snapshot
    // on demand via getServerTiming / resetServerTiming requests.
    pub collect_timing: bool,
    // RunExternalCode allows configured content mappers to execute.
    pub run_external_code: bool,
    pub content_mapper_spawner: Option<Rc<dyn contentmapper::Spawner>>,
}

// Go: server.go:46 StdioServer
// StdioServer runs an API session over STDIO using MessagePack protocol.
// This is the entry point for the synchronous STDIO-based API used by
// native TypeScript tooling integration.
pub struct StdioServer {
    options: StdioServerOptions,
}

// Go: server.go:51 NewStdioServer
// NewStdioServer creates a new STDIO-based API server.
// PORT: Go keeps the `*StdioServerOptions` pointer; the server owns the
// options here (they hold the stdin and stdout handles).
pub fn new_stdio_server(options: StdioServerOptions) -> StdioServer {
    if options.cwd.is_empty() {
        crate::core::go_panic("StdioServerOptions.Cwd is required".to_string());
    }

    StdioServer { options }
}




// Go `defer f()`: runs `f` when the guard leaves scope, on every return
// path and during a panic.
struct Defer<F: FnMut()>(F);

impl<F: FnMut()> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}

impl StdioServer {
    // Go: server.go:62 Run
    // Run starts the server and blocks until the connection closes.
    // PORT: `&mut self` because Accept moves the stdin and stdout handles
    // out of the options.
}

// Go: server.go serverRunError (ts#64276)
fn server_run_error(ctx: &Context, err: Result<(), GoError>) -> Result<(), GoError> {
    if ctx.err().is_some() {
        return Ok(());
    }
    err
}

// Go: api/server_test.go (ts#64276)
#[cfg(test)]
mod server_tests {
    use super::*;
    use crate::gostd::context;

    // PORT: no Go counterpart. A free that a nested message leaves (a
    // request that a client makes while it answers a callback, ts#64299)
    // waits for the end of the top-level message.
    #[test]
    fn frees_wait_for_the_end_of_the_top_level_message() {
        struct Answers;
        impl Handler for Answers {
            fn handle_request(
                &self,
                _ctx: &Context,
                _method: &str,
                _params: JsonValue,
            ) -> Result<Option<Box<dyn AnyValue>>, GoError> {
                Ok(None)
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
        struct Freed(Rc<Cell<bool>>);
        impl Drop for Freed {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }
        crate::gostd::local::keep_garbage();
        let handler = FreeAfterMessage {
            inner: Rc::new(Answers),
            depth: Cell::new(0),
        };
        let freed = Rc::new(Cell::new(false));
        {
            let _outer = handler.enter();
            {
                let _nested = handler.enter();
                crate::gostd::local::drop_later(Box::new(Freed(freed.clone())));
            }
            assert!(!freed.get(), "a nested message keeps its frees");
        }
        assert!(freed.get(), "the top-level message runs them at its end");
        let freed = Rc::new(Cell::new(false));
        crate::gostd::local::drop_later(Box::new(Freed(freed.clone())));
        assert!(
            handler
                .handle_notification(&context::background(), "x", JsonValue::default())
                .is_ok()
        );
        assert!(freed.get(), "a message runs the frees that wait");
    }

    // Go: api/server_test.go:14 TestServerRunError/EOF
    #[test]
    fn test_server_run_error_eof() {
        assert!(server_run_error(&context::background(), Ok(())).is_ok());
    }

    // Go: api/server_test.go:19 TestServerRunError/context cancellation
    #[test]
    fn test_server_run_error_context_cancellation() {
        let (ctx, cancel) = context::with_cancel(&context::background());
        cancel();
        assert!(server_run_error(&ctx, Err(context::CANCELED.clone())).is_ok());
    }

    // Go: api/server_test.go:26 TestServerRunError/unrelated cancellation
    #[test]
    fn test_server_run_error_unrelated_cancellation() {
        let err = context::CANCELED.clone();
        let result = server_run_error(&context::background(), Err(err.clone()));
        assert!(result.is_err_and(|result| errors::is(&result, &err)));
    }

    // Go: api/server_test.go:32 TestServerRunError/context cancellation supersedes server error
    #[test]
    fn test_server_run_error_context_cancellation_supersedes_server_error() {
        let (ctx, cancel) = context::with_cancel(&context::background());
        cancel();
        assert!(server_run_error(&ctx, Err(errors::new("server failed"))).is_ok());
    }
}
