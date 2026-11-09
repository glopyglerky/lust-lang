//! Port of internal/api/protocol_msgpack.go.
//!
//! PORT: Go wraps the connection in a 4096-byte `bufio.Reader` and
//! `bufio.Writer`; the port uses `BufReader` and `BufWriter` of the same
//! size. The frames are written by hand, as in Go. An `std::io::Error`
//! becomes a `GoError` with the same text; end of input is `errors::EOF`.

use crate::api::prelude::*;

use crate::frontend::json::{JsonError, MarshalerTo, json_marshal};
use crate::frontend::json_ext::{AnyValue, JsonValue};
use crate::gostd::{GoError, errors};
use crate::ipc::{ConnReader, ReadWriteCloser};
use crate::jsonrpc;
use std::io::{BufReader, BufWriter, Read, Write};
use std::sync::Arc;

// Go: protocol_msgpack.go:15 MessageType
// MessageType represents the type of message in the msgpack protocol.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MessageType(pub u8);

// Go: protocol_msgpack.go:16
impl MessageType {
    pub const UNKNOWN: MessageType = MessageType(0);
    pub const REQUEST: MessageType = MessageType(1);
    pub const CALL_RESPONSE: MessageType = MessageType(2);
    pub const CALL_ERROR: MessageType = MessageType(3);
    pub const RESPONSE: MessageType = MessageType(4);
    pub const ERROR: MessageType = MessageType(5);
    pub const CALL: MessageType = MessageType(6);

    // Go: protocol_msgpack.go:27 IsValid
    pub fn is_valid(self) -> bool {
        self >= MessageType::REQUEST && self <= MessageType::CALL
    }
}

// Go: protocol_msgpack.go:31
// MessagePack format constants
const MSGPACK_FIXED_ARRAY3: u8 = 0x93;
const MSGPACK_BIN8: u8 = 0xC4;
const MSGPACK_BIN16: u8 = 0xC5;
const MSGPACK_BIN32: u8 = 0xC6;
const MSGPACK_U8: u8 = 0xCC;

// Go: protocol_msgpack.go:42 MessagePackProtocol
// MessagePackProtocol implements the Protocol interface using a custom
// msgpack-based tuple format: [MessageType, method, payload].

// Go: protocol_msgpack.go:46
// var _ Protocol = (*MessagePackProtocol)(nil) is the `impl Protocol` below.

// Go: protocol_msgpack.go:50 NewMessagePackProtocol
// NewMessagePackProtocol creates a new msgpack protocol handler.
// PORT: Go takes an `io.ReadWriter`; the port takes the shared connection.


// PORT: the unexported helpers sit in this inherent impl, after the
// `Protocol` impl; Go has readTuple and readBin between ReadMessage and
// WriteRequest.

// Go `fmt.Errorf("%w: ...", ErrInvalidRequest, ...)`.

fn io_error(err: std::io::Error) -> GoError {
    errors::new(err.to_string())
}

// Go `bufio.Reader.ReadByte`: `io.EOF` at the end of input.

// Go `io.ReadFull` (and `binary.Read`, which uses it): `io.EOF` when no byte
// was read, `io.ErrUnexpectedEOF` when the input ends inside `buf`.
// PORT: `io.ErrUnexpectedEOF` is a plain error with the Go text.

// Go `bufio.Writer.WriteByte` / `Write` and `binary.Write`.
fn write_all<W: Write>(w: &mut W, data: &[u8]) -> Result<(), GoError> {
    w.write_all(data).map_err(io_error)
}

// Go: protocol_msgpack.go:281 RawBinary
// RawBinary is a marker type for binary data that should be written
// directly by MessagePackProtocol instead of being JSON-encoded.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawBinary(pub Vec<u8>);

// Go v2 marshal of a `[]byte` type: a base64 (standard encoding) string.
// Only reached when a RawBinary result goes through the JSON-RPC protocol.

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    /// A connection that reads `input` and keeps what is written.
    struct Pipe {
        input: Mutex<std::io::Cursor<Vec<u8>>>,
        output: Mutex<Vec<u8>>,
    }

    impl ReadWriteCloser for Pipe {
        fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.input.lock().expect("input").read(buf)
        }
        fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
            self.output.lock().expect("output").extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        fn close(&self) -> Result<(), GoError> {
            Ok(())
        }
    }

    /// The tuple frame `[msg_type, method, payload]` with bin8 strings.
    fn frame(msg_type: MessageType, method: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut out = vec![MSGPACK_FIXED_ARRAY3, msg_type.0];
        for data in [method, payload] {
            out.extend_from_slice(&[MSGPACK_BIN8, data.len() as u8]);
            out.extend_from_slice(data);
        }
        out
    }

    // Go `string(methodBytes)` keeps any bytes (protocol_msgpack.go:135):
    // `%q` of an unknown method quotes them (proto.go:2029), and the error
    // response gives the method back as the same bytes (writeTuple). Texts
    // from Go N (`tsgo --api`, followups25 skeptic apiraw.py): a real U+FDD0
    // stays one char, and a byte that is not valid UTF-8 is `\x..`.
    #[test]
    fn a_method_keeps_its_go_bytes() {
        let method: &[u8] = b"a\xef\xb7\x90\xef\xb7\x90b\xff";
        let pipe = Arc::new(Pipe {
            input: Mutex::new(std::io::Cursor::new(frame(
                MessageType::REQUEST,
                method,
                b"null",
            ))),
            output: Mutex::new(Vec::new()),
        });
        let mut protocol = new_message_pack_protocol(pipe.clone());
        let msg = protocol.read_message().expect("read the request");
        let err = crate::api::proto::unmarshal_payload(&msg.method, b"null")
            .err()
            .expect("an unknown method");
        assert_eq!(err.error(), r#"unknown API method "a\ufdd0\ufdd0b\xff""#);
        protocol
            .write_error(
                msg.id.as_ref(),
                &jsonrpc::ResponseError {
                    code: jsonrpc::CODE_INTERNAL_ERROR,
                    message: err.error(),
                    data: None,
                },
            )
            .expect("write the error");
        assert_eq!(
            *pipe.output.lock().expect("output"),
            frame(MessageType::ERROR, method, err.error().as_bytes())
        );
    }
}
