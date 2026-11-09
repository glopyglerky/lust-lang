use crate::api::prelude::*;

// Port of Go `internal/api/session.go`, lines 1272-2484: the
// `resolve*PropertyOf*` helpers, the checker query handlers, node building
// and printing, emit (tsgo#4699), intrinsic types, diagnostics, `resolveNodeHandle`,
// `computeSnapshotChanges`, `Close`, and the references, signature usage and
// completion handlers. The file header of `session_p1.rs` holds the PORT
// notes for both files (registry entries keep the owning checker; handles
// cross to the setup checker only through `checker_symbol`, `checker_type`
// and `checker_signature`).

use crate::api::encoder;
use crate::api::requestfilesystem;
use crate::emitter::emitter::EmitOnly;
use crate::emitter::program_emit::{self, };
use crate::frontend::compiler;
use crate::frontend::core_context::{self, CheckerLifetime};
use crate::frontend::core_ls_ext::{diff_maps, diff_ordered_maps};
use crate::frontend::json_ext::AnyValue;
use crate::frontend::tspath;
use crate::gostd::{self, GoError, errors};
use crate::program::ls_program;
use crate::project;
use std::sync::{Arc, Mutex, PoisonError};


// Go: api/session.go:3642 emitToOutput (tsgo#4699)
// PORT: the write callback runs on the checker threads, so the outputs are
// in an `Arc<Mutex>` (Go `mu`). The caller keeps `program` current.


// Go: api/session.go:3693 getEmitOnly (tsgo#4699)
// PORT: Go converts the number to `compiler.EmitOnly` (EmitAll 0,
// EmitOnlyJs 1, EmitOnlyDts 2); the port matches it to the enum.

// Go: api/session.go:3704 emitProgram (tsgo#4699)
// PORT: Go `program.Emit` of the project program. The port runs the compile
// emit (`program_emit::emit`) with the program current (`ls_program::enter`).
// The first emit of a program version makes its compile checker pool;
// `ls_program` frees it with the program.
// `program_emit::emit` takes no context and always returns a result, so
// Go's nil result branches (a canceled `ctx`) do not happen here.

// Go: api/session.go:3715 nonNilDiagnostics (tsgo#4699)
// PORT: a Rust `Vec` has no nil. An empty list marshals as `[]`, the same as
// Go's non-nil empty slice.




// Go: api/session.go:4560 computeSnapshotChanges
// computeSnapshotChanges computes the per-project source file differences between
// two snapshots. It uses DiffOrderedMaps on projects to find changed/removed projects,
// then DiffMaps on FilesByPath for each changed project to collect file-level changes.



// Go: api/session.go:4740 formatSessionID
pub fn format_session_id(id: u64) -> String {
    format!("api-session-{id}")
}


// Go: encoding/base64/base64.go:139 (*Encoding).EncodeToString (StdEncoding)
// PORT: the crate has no base64 dependency. Go `base64.StdEncoding`: the
// standard alphabet with `=` padding.
// PERF: (apiperf1) Go `Encode` (base64.go:145) writes each 3 bytes as 4
// bytes of a buffer of the final size; so does this. The text is ASCII, so
// the UTF-8 check at the end is one fast pass. A `String::push` for each
// byte was 5% of an API createSourceFile loop of a 260 KB file.

#[cfg(test)]
mod base64_tests {
    use super::*;

    // Go: encoding/base64/base64_test.go pairs (RFC 3548 examples and the
    // padding cases), StdEncoding.
    #[test]
    fn encode_to_string_matches_go() {
        for (raw, encoded) in [
            (&b""[..], ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (b"foob", "Zm9vYg=="),
            (b"fooba", "Zm9vYmE="),
            (b"foobar", "Zm9vYmFy"),
            (b"\x14\xfb\x9c\x03\xd9\x7e", "FPucA9l+"),
            (b"\x14\xfb\x9c\x03\xd9", "FPucA9k="),
            (b"\x14\xfb\x9c\x03", "FPucAw=="),
        ] {
            assert_eq!(base64_std_encoding_encode_to_string(raw), encoded);
            assert_eq!(
                base64_std_encoding_decode_string(encoded).expect("valid base64"),
                raw
            );
        }
    }
}

// Go: encoding/base64/base64.go:429 (*Encoding).DecodeString (StdEncoding)
// PORT: the crate has no base64 dependency. This is Go `Decode` as a loop of
// `decodeQuantum` (base64.go:312): the standard alphabet, `=` padding
// required, not strict, `\r` and `\n` skipped. Go's 8- and 4-byte fast paths
// decode valid input the same way and fall back to `decodeQuantum` at the
// same offset, so results and error offsets are equal. Go returns the
// partial data with the error; the only caller uses the error alone. Go
// `CorruptInputError` is a `GoError` with the same text.
pub fn base64_std_encoding_decode_string(s: &str) -> Result<Vec<u8>, GoError> {
    fn decode_map(c: u8) -> u8 {
        match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => 0xFF,
        }
    }
    // Go: base64.go:303 CorruptInputError.Error
    fn corrupt_input_error(offset: usize) -> GoError {
        errors::new(format!("illegal base64 data at input byte {offset}"))
    }

    let src = s.as_bytes();
    let mut dst: Vec<u8> = Vec::with_capacity(src.len() / 4 * 3);
    if src.is_empty() {
        return Ok(dst);
    }
    let mut si = 0usize;
    while si < src.len() {
        // Go: base64.go:312 decodeQuantum
        let mut dbuf = [0u8; 4];
        let mut dlen = 4usize;
        let mut err: Option<GoError> = None;

        let mut j = 0usize;
        while j < dbuf.len() {
            if src.len() == si {
                if j == 0 {
                    return Ok(dst);
                }
                // j == 1, or padding is required (StdEncoding)
                return Err(corrupt_input_error(si - j));
            }
            let input = src[si];
            si += 1;

            let out = decode_map(input);
            if out != 0xFF {
                dbuf[j] = out;
                j += 1;
                continue;
            }

            if input == b'\n' || input == b'\r' {
                continue;
            }

            if input != b'=' {
                return Err(corrupt_input_error(si - 1));
            }

            // We've reached the end and there's padding
            match j {
                0 | 1 => {
                    // incorrect padding
                    return Err(corrupt_input_error(si - 1));
                }
                2 => {
                    // "==" is expected, the first "=" is already consumed.
                    // skip over newlines
                    while si < src.len() && (src[si] == b'\n' || src[si] == b'\r') {
                        si += 1;
                    }
                    if si == src.len() {
                        // not enough padding
                        return Err(corrupt_input_error(src.len()));
                    }
                    if src[si] != b'=' {
                        // incorrect padding
                        return Err(corrupt_input_error(si - 1));
                    }

                    si += 1;
                }
                _ => {}
            }

            // skip over newlines
            while si < src.len() && (src[si] == b'\n' || src[si] == b'\r') {
                si += 1;
            }
            if si < src.len() {
                // trailing garbage
                err = Some(corrupt_input_error(si));
            }
            dlen = j;
            break;
        }

        // Convert 4x 6bit source bytes into 3 bytes
        let val = u32::from(dbuf[0]) << 18
            | u32::from(dbuf[1]) << 12
            | u32::from(dbuf[2]) << 6
            | u32::from(dbuf[3]);
        let bytes = [(val >> 16) as u8, (val >> 8) as u8, val as u8];
        match dlen {
            4 => dst.extend_from_slice(&bytes[..3]),
            3 => dst.extend_from_slice(&bytes[..2]),
            2 => dst.extend_from_slice(&bytes[..1]),
            _ => {}
        }

        if let Some(err) = err {
            return Err(err);
        }
    }
    Ok(dst)
}

// Go: strconv/number.go:104 ParseUint and internal/strconv/atoi.go:47 ParseUint
// PORT: Go stdlib. Only `2 <= base <= 36` is ported (the caller passes 10);
// Go `*strconv.NumError` is a `GoError` with the same text.

// Go: api/session.go:3543 decodePrintNode (ts#64320)

// Go: api/session.go:3556 newPrinter (ts#64320)
// PORT: private, so it does not collide with `printer::new_printer` in the
// api prelude; it calls that one by path.

