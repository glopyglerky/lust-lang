//! Port of Go `api/encoder/decoder.go`: rebuilds AST nodes from the binary
//! format that `encoder.rs` writes.
//!
//! PORT notes for the whole file:
//! - Go `int` is 64-bit. The decoder reads offsets and node indices from
//!   untrusted `uint32` fields, so Go `int` is `i64` here and keeps Go's range
//!   checks (`readLE32` returns 0 out of range; a bad index panics in both).
//! - Nodes are made by a factory `NodeFactory` (synthetic arena), as Go makes
//!   them with `ast.NewNodeFactory`.

use crate::api::encoder::prelude::*;

use crate::frontend::parser::{ExternalModuleIndicatorOptions, SourceFileParseOptions};
use crate::frontend::tspath::Path;
use crate::gostd::errors;

// Go: api/encoder/decoder.go:14 astDecoder

// Go: api/encoder/decoder.go:33 DecodeSourceFile

// Go: api/encoder/decoder.go:45 DecodeNodes

// Go: api/encoder/decoder.go:53 newASTDecoder


// Go: api/encoder/decoder.go:197 childIterator
/// childIterator helps walk through children based on a bitmask.
pub struct ChildIterator<'a> {
    indices: &'a [i64],
    pos: i64,
}

// Go: api/encoder/decoder.go:202 newChildIter
pub fn new_child_iter(indices: &[i64]) -> ChildIterator<'_> {
    ChildIterator { indices, pos: 0 }
}

impl ChildIterator<'_> {
    // Go: api/encoder/decoder.go:204 (*childIterator).next
    /// next returns the index of the next child, advancing the position.
    pub fn next(&mut self) -> i64 {
        if self.pos >= self.indices.len() as i64 {
            return 0;
        }
        let ci = self.indices[self.pos as usize];
        self.pos += 1;
        ci
    }

    // Go: api/encoder/decoder.go:214 (*childIterator).nextIf
    /// nextIf returns the index of the next child if the corresponding mask bit is set.
    pub fn next_if(&mut self, mask: u8, bit: u8) -> i64 {
        if mask & (1 << bit) == 0 {
            return 0;
        }
        self.next()
    }
}


// Go: api/encoder/decoder.go:343 readLE32

// Hand-written commonData decoding functions. Each extracts the original values
// from the 6-bit commonData that were packed by the corresponding
// getNodeCommonData_* function.

// Go: api/encoder/decoder.go:354 decodeNodeCommonData_SyntheticExpression
// PORT: Go returns `(any, bool)`; the `any` is the checker type, `TypeId` here.

// Hand-written extended data decoding functions for literal nodes.


#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::ScriptKind;
    use crate::frontend::parser;
    use crate::scanner_util::go_string_from_bytes;
    use std::rc::Rc;

    // Go `%q` of a bad source file name quotes its bytes (decoder.go:271,
    // wrapped at :172). The encoder writes the Go bytes of the name
    // (`StringTable`); the test then makes the name relative. A byte that is
    // not valid UTF-8 is `\x..` and a real U+FDD0 stays one char.
    #[test]
    fn a_bad_file_name_is_quoted_from_its_go_bytes() {
        let go_name: &[u8] = b"/x\xff\xef\xb7\x90\xef\xb7\x90.ts";
        let name = go_string_from_bytes(go_name.to_vec());
        let file = Rc::new(parser::parse_source_file(
            &SourceFileParseOptions {
                file_name: name.clone(),
                path: Path(name),
                ..Default::default()
            },
            "let x = 1;",
            ScriptKind::TS,
        ));
        crate::program::note_parsed_source_file(&file);
        let (mut buf, _) = crate::api::encoder::encode_source_file(file.root).expect("encode");
        let at = buf
            .windows(go_name.len())
            .position(|bytes| bytes == go_name)
            .expect("the name's Go bytes in the string data");
        buf[at] = b'_';
        assert_eq!(
            decode_source_file(&buf).expect_err("a bad name").error(),
            r#"at node 1 (kind KindSourceFile): invalid source file name "_x\xff\ufdd0\ufdd0.ts""#
        );
    }
}
