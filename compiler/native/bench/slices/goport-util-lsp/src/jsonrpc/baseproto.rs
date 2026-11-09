//! Port of internal/jsonrpc/baseproto.go.
//!
//! Base protocol for JSON-RPC with Content-Length headers (as used by LSP).
//! https://microsoft.github.io/language-server-protocol/specifications/base/0.9/specification/
//!
//! PORT: Go wraps an `io.Reader` / `io.Writer` in `bufio`. The Rust reader
//! takes a `BufRead` (the caller supplies the buffer); the writer wraps its
//! `Write` in a 4096-byte `BufWriter` (Go `bufio.NewWriter` size). An
//! `std::io::Error` becomes a `GoError` with the same text. Go `bufio.Writer`
//! keeps the first write error for later writes; `BufWriter` retries.

use crate::jsonrpc::prelude::*;

use crate::gostd::{GoError, errors, strconv};
use std::io::{BufRead, BufWriter, Read, Write};
use std::sync::LazyLock;

// Go: baseproto.go:16

// Go: baseproto.go:22 Reader
/// Reader reads JSON-RPC messages with Content-Length framing.
pub struct Reader {
    r: Box<dyn BufRead + Send>,
}

// Go: baseproto.go:27 NewReader
// NewReader creates a new Reader.
pub fn new_reader(r: Box<dyn BufRead + Send>) -> Reader {
    Reader { r }
}

impl Reader {
    // Go: baseproto.go:34 Read
    // Read reads the next message payload.
}

// Go: io/io.go:353 ReadFull (ReadAtLeast with min = len(buf))
// PORT: `io.ErrUnexpectedEOF` is not a gostd sentinel; it is a plain error
// with the Go text.

// Go: strconv/number.go:137 ParseInt(s, 10, 64) with
// internal/strconv/atoi.go:47 ParseUint and :171 ParseInt.
// Returns the `*strconv.NumError` text on error.
// PORT: only base 10 and 64 bits, the case baseproto uses.

// Go: baseproto.go:79 Writer

// Go: baseproto.go:84 NewWriter
// NewWriter creates a new Writer.


#[cfg(test)]
mod tests {
    use super::*;

    fn read_error(input: &[u8]) -> String {
        new_reader(Box::new(std::io::Cursor::new(input.to_vec())))
            .read()
            .expect_err("want a header error")
            .error()
    }

    // Go `%q` of a header line, and of the Content-Length value in the
    // ParseInt error, quotes the Go bytes (baseproto.go:52 and :56): a byte
    // that is not valid UTF-8 is `\x..`, and a real U+FDD0 stays one char.
    // Texts from Go N (`tsgo --lsp --stdio`, followups25 skeptic lsphdr.py).
    #[test]
    fn header_errors_quote_the_go_bytes() {
        assert_eq!(
            read_error(b"\xff\xfe\r\n"),
            r#"jsonrpc: invalid header: "\xff\xfe\r\n""#
        );
        assert_eq!(
            read_error(b"\xef\xb7\x90\xef\xb7\x90\r\n"),
            r#"jsonrpc: invalid header: "\ufdd0\ufdd0\r\n""#
        );
        assert_eq!(
            read_error(b"\xef\xb7\x90\xf4\x8f\x9f\xbf\r\n"),
            r#"jsonrpc: invalid header: "\ufdd0\U0010f7ff\r\n""#
        );
        assert_eq!(
            read_error(b"Content-Length: \xef\xb7\x90\xef\xb7\x90\r\n\r\n"),
            r#"jsonrpc: invalid content length: parse error: strconv.ParseInt: parsing "\ufdd0\ufdd0": invalid syntax"#
        );
        assert_eq!(
            read_error(b"Content-Length: 1\xff\r\n\r\n"),
            r#"jsonrpc: invalid content length: parse error: strconv.ParseInt: parsing "1\xff": invalid syntax"#
        );
    }
}
