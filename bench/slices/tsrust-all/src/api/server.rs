//! Port of internal/api/server.go.

use crate::api::prelude::*;

use crate::api::callbackfs::{};
use crate::contentmapper;
use crate::frontend::bundled;
use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::frontend::vfs::{self, };
use crate::gostd::{GoError, errors};
use crate::ipc::{
    
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

// Go: server.go:46 StdioServer
// StdioServer runs an API session over STDIO using MessagePack protocol.
// This is the entry point for the synchronous STDIO-based API used by
// native TypeScript tooling integration.

// Go: server.go:51 NewStdioServer
// NewStdioServer creates a new STDIO-based API server.
// PORT: Go keeps the `*StdioServerOptions` pointer; the server owns the
// options here (they hold the stdin and stdout handles).




// Go `defer f()`: runs `f` when the guard leaves scope, on every return
// path and during a panic.
struct Defer<F: FnMut()>(F);

impl<F: FnMut()> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}


// Go: server.go serverRunError (ts#64276)

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
