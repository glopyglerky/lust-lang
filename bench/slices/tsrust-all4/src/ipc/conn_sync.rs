//! Port of internal/ipc/conn_sync.go (internal/api/conn_sync.go before tsgo#4712).

use crate::ipc::prelude::*;

use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::gostd::{Context, GoError, errors, strconv};
use crate::ipc::conn::{recovered_value};
use crate::ipc::protocol::{};
use crate::ipc::transport::ReadWriteCloser;
use crate::jsonrpc;
use std::sync::Arc;
use std::time::Instant;

// Go: ipc/conn_sync.go:18 SyncConn
// SyncConn manages bidirectional communication with synchronous request handling.
// Requests are handled one at a time inline, and outgoing calls are serialized.

// Go: ipc/conn_sync.go:34 NewSyncConn
// NewSyncConn creates a new sync connection with the given transport and handler.

// PORT: the Go methods are inherent methods; `impl Conn` below forwards to
// them, so callers need not import `Conn`.


// Go: ipc/conn_sync_test.go
// PORT: Go shares `noOpHandler` with conn_async_test.go (one test package);
// here the conn_async tests export it. Go `nil` for the transport is
// `NilTransport`.
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::gostd::context;
    use crate::ipc::conn_async::tests::{
        NilTransport, NoOpHandler, message, response_write_failed,
    };

    // Go: ipc/conn_sync_test.go:15 syncFailingResponseProtocol
    struct SyncFailingResponseProtocol {
        message: Option<Message>,
        response_err: GoError,
    }

    impl Protocol for SyncFailingResponseProtocol {
        fn read_message(&mut self) -> Result<Message, GoError> {
            self.message.take().ok_or_else(|| errors::EOF.clone())
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
            Err(self.response_err.clone())
        }

        fn write_error(
            &mut self,
            _id: Option<&jsonrpc::ID>,
            _err: &jsonrpc::ResponseError,
        ) -> Result<(), GoError> {
            Err(self.response_err.clone())
        }
    }

    // Go: ipc/conn_sync_test.go:45 panicHandler
    struct PanicHandler;

    impl Handler for PanicHandler {
        fn handle_request(
            &self,
            _ctx: &Context,
            _method: &str,
            _params: JsonValue,
        ) -> Result<Option<Box<dyn AnyValue>>, GoError> {
            panic!("handler panic")
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

    // Go: ipc/conn_sync_test.go:55 TestSyncConnRunReturnsResponseWriteFailure
    #[test]
    fn test_sync_conn_run_returns_response_write_failure() {
        let response_err = response_write_failed();
        let protocol = SyncFailingResponseProtocol {
            message: Some(message(Some(jsonrpc::new_id_int(1)), "transform")),
            response_err: response_err.clone(),
        };
        let conn = new_sync_conn(
            Arc::new(NilTransport),
            Box::new(protocol),
            Rc::new(NoOpHandler),
        );

        let err = conn
            .run(&context::background())
            .expect_err("run returns the response write error");
        assert!(
            errors::is(&err, &response_err),
            "expected response write error, got {}",
            err.error()
        );
    }

    // Go: ipc/conn_sync_test.go:68 TestSyncConnRunReturnsPanicResponseWriteFailure
    #[test]
    fn test_sync_conn_run_returns_panic_response_write_failure() {
        let response_err = response_write_failed();
        let protocol = SyncFailingResponseProtocol {
            message: Some(message(Some(jsonrpc::new_id_int(1)), "transform")),
            response_err: response_err.clone(),
        };
        let conn = new_sync_conn(
            Arc::new(NilTransport),
            Box::new(protocol),
            Rc::new(PanicHandler),
        );

        let err = conn
            .run(&context::background())
            .expect_err("run returns the panic response write error");
        assert!(
            errors::is(&err, &response_err),
            "expected panic response write error, got {}",
            err.error()
        );
        assert!(
            err.error().contains("original panic: handler panic"),
            "{}",
            err.error()
        );
    }
}
