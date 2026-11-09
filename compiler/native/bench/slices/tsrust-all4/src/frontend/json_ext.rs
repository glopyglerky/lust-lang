//! Go JSON v2 default arshalers that `frontend/json.rs` lacks, for the
//! language-service port (plan contract C2).
//!
//! Behavior reference: `github.com/go-json-experiment/json`
//! v0.0.0-20260601182631-00ed12fed2a6 (`arshal_default.go`,
//! `arshal_any.go`, `jsontext/encode.go`, `jsontext/state.go`,
//! `internal/jsonwire`). Go reflects over types; each Rust type here has a
//! hand-written `MarshalerTo` / `UnmarshalerFrom` with the same default rules.
//!
//! Impls that already exist and are not added again: `json.rs` (String,
//! bool, f64 unmarshal; `FxHashMap<String, V>` unmarshal; str, String, bool,
//! `[T]`, `Vec<T>`, `IndexMap<String, V>` marshal) and `packagejson.rs`
//! (`IndexMap<String, V>` unmarshal, Go `collections.OrderedMap` rules).
//! `IndexMap<DocumentUri, V>` lives in `lsp/lsproto/lsp.rs`.
//!
//! Unmarshal errors of the arshalers here are Go v2 `SemanticError` texts
//! (see `SemanticError`). The JSON pointer is added by `unmarshal_root`; an
//! error that reaches the caller through `json.rs` `json_unmarshal` has no
//! pointer. The `json.rs` impls keep their own texts, except that their kind
//! errors (a string, a boolean or a number of the wrong kind) use
//! `unmarshal_kind_error`.
//!
//! PORT: integer arshalers do not port the stringified form that v2 uses
//! for integer map keys (no LSP type has integer keys).

use crate::frontend::prelude::*;

use std::any::Any;
use std::cell::RefCell;

/// Go `any` holding a value that the LSP layer can marshal
/// (`RequestMessage.Params`, `ResponseMessage.Result`, `Message.msg`).
/// Read it with `downcast_ref::<T>()`. Inbound LSP params and results are a
/// raw `JsonValue`; `lsproto::unmarshal_params` and `unmarshal_result`
/// decode them.
///
/// Never box a `Box<dyn AnyValue>` again: the blanket impl makes the box
/// itself an `AnyValue`, and a downcast of the outer box fails.
pub trait AnyValue: Any + MarshalerTo + std::fmt::Debug + Send {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + MarshalerTo + std::fmt::Debug + Send> AnyValue for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}



/// Go `json.Value` (`jsontext.Value`): raw JSON bytes.
/// PORT: Go nil and empty are both an empty `Vec`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JsonValue(pub Vec<u8>);

impl AsRef<[u8]> for JsonValue {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Go `omitzero` test (`reflect.Value.IsZero`, or a type's `IsZero`
/// method).
pub trait IsZero {
    fn is_zero(&self) -> bool;
}

impl IsZero for String {
    fn is_zero(&self) -> bool {
        self.is_empty()
    }
}

impl IsZero for bool {
    fn is_zero(&self) -> bool {
        !*self
    }
}

impl IsZero for i32 {
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl IsZero for u32 {
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl IsZero for u64 {
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

// Go reflect: a float is zero only when all bits are zero, so -0 is not.
impl IsZero for f64 {
    fn is_zero(&self) -> bool {
        self.to_bits() == 0
    }
}

// PORT: Go omits only a nil slice; Rust cannot tell nil from empty.
impl<T> IsZero for Vec<T> {
    fn is_zero(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsZero for Option<T> {
    fn is_zero(&self) -> bool {
        self.is_none()
    }
}

// PORT: Go omits only a nil map; Rust cannot tell nil from empty.
impl<K, V, S> IsZero for IndexMap<K, V, S> {
    fn is_zero(&self) -> bool {
        self.is_empty()
    }
}

// Go: a nil `any` is zero; a non-nil `any` is not.

impl IsZero for JsonValue {
    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

// Go jsontext `Kind.normalize`.
fn normalize_kind(c: u8) -> u8 {
    match c {
        b'-' | b'0'..=b'9' => b'0',
        _ => c,
    }
}

fn is_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\r' | b'\n')
}

fn skip_ws(data: &[u8], mut p: usize) -> usize {
    while p < data.len() && is_ws(data[p]) {
        p += 1;
    }
    p
}

fn skip_ws_back(data: &[u8], mut p: usize) -> usize {
    while p > 0 && is_ws(data[p - 1]) {
        p -= 1;
    }
    p
}


// Go `newUnmarshalErrorAfterWithValue(dec, t, err)`: Go keeps the value of a
// string or a number, and the text shows it when it is shorter than 100 bytes.

// ---------------------------------------------------------------------------
// Go v2 unmarshal errors
// ---------------------------------------------------------------------------

/// Where the decoder stands when an unmarshal error is made. It gives the Go
/// `where` argument of `appendStackPointer` (the JSON pointer) and the byte
/// offset of the error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorPos {
    /// Before the value (Go `newUnmarshalErrorBefore`, where = +1).
    Before,
    /// Just after the value, or after the name of an object member (Go
    /// `newUnmarshalErrorAfter`, where = -1).
    After,
    /// Just after the closing token of an object or array that was read token
    /// by token (where = -1; the offset is that of the closing token).
    AfterEnd,
}

/// Go v2 `*json.SemanticError` of an unmarshal (errors.go:73).
///
/// PORT: `JsonError` holds only text. The `SemanticError` of the last error
/// made on this thread is kept beside it, so that a caller up the stack can
/// fill in the Go type or the JSON pointer and write the text again, as Go
/// fills in the fields of the same error value.
#[derive(Clone, Debug)]
pub struct SemanticError {
    /// `false` while this is a plain error that an `UnmarshalJSONFrom` method
    /// returned: its text is `err` alone. The caller of the method wraps it
    /// (`wrap_method_error`).
    pub wrapped: bool,
    /// Go `JSONKind` (0 when unknown).
    pub json_kind: u8,
    /// Go `JSONValue`: the raw string or number that failed, or empty.
    pub json_value: String,
    /// Go `GoType.String()`.
    pub go_type: String,
    /// Go `JSONPointer`. `None` until `unmarshal_root` finds it.
    pub pointer: Option<String>,
    /// Go `ByteOffset`. The text shows it only when the pointer is empty.
    pub byte_offset: usize,
    pub pos: ErrorPos,
    /// The text of Go `Err` (empty for nil).
    pub err: String,
}

thread_local! {
    static LAST_ERROR: RefCell<Option<SemanticError>> = const { RefCell::new(None) };
}

impl SemanticError {
    // Go `newUnmarshalErrorAfter` for a default arshaler.
    fn after(json_kind: u8, json_value: &str, go_type: &str, err: &str) -> SemanticError {
        SemanticError {
            wrapped: true,
            json_kind,
            json_value: json_value.to_string(),
            go_type: go_type.to_string(),
            pointer: None,
            byte_offset: 0,
            pos: ErrorPos::After,
            err: err.to_string(),
        }
    }



}

// Go: errors.go:356 (*SemanticError).Error
// PORT: Go picks "cannot" or "unable to" at random once per process; the
// port always writes "cannot". Go prints only the kind of a type whose name
// is longer than 100 bytes; no port type has such a name.

// Go: internal/jsonwire/wire.go:165 TruncatePointer






// One open object or array of the Go decoder state (jsontext `stateEntry`,
// with the last object name read in it).
struct StackEntry {
    is_object: bool,
    // Names and values read so far.
    len: usize,
    name: String,
}

// The open objects and arrays below the top level after the tokens in
// `data[..end]`.

// Go: jsontext/state.go:180 appendStackPointer, with where = +1 for
// `Before` and -1 otherwise.
fn stack_pointer(stack: &[StackEntry], pos: ErrorPos) -> String {
    let before = pos == ErrorPos::Before;
    let mut b = String::new();
    for (i, e) in stack.iter().enumerate() {
        // By default point to the previous array element.
        let mut index = e.len.wrapping_sub(1);
        if i == stack.len() - 1 {
            if (!before && e.len == 0) || (before && e.is_object && e.len.is_multiple_of(2)) {
                return b;
            }
            if before && !e.is_object {
                // Point to the next array element.
                index = e.len;
            }
        }
        b.push('/');
        if e.is_object {
            // Per RFC 6901, section 3, escape '~' and '/' characters.
            for c in e.name.chars() {
                match c {
                    '~' => b.push_str("~0"),
                    '/' => b.push_str("~1"),
                    c => b.push(c),
                }
            }
        } else {
            b.push_str(&index.to_string());
        }
    }
    b
}



// The top-level arguments of a generic Rust type name ("A, B<C, D>").

// Go: jsonwire/decode.go:593 ParseUint
// ParseUint parses b as a decimal unsigned integer according to
// a strict subset of the JSON number grammar, returning the value if valid.
// It returns (0, false) if there is a syntax error and
// returns (math.MaxUint64, false) if there is an overflow.

// Go: arshal_default.go:492 makeIntArshaler (unmarshal)
// Returns the value to store. `null` stores 0. A JSON string is an error
// (not stringified), and so is a number with a fraction or exponent.

// Go: arshal_default.go:591 makeUintArshaler (unmarshal)




// Go: jsonwire/encode.go:213 AppendFloat (bits == 64)
// Go formats with strconv 'f' or 'e' and precision -1 (shortest digits that
// round-trip). Rust `{}` and `{:e}` print the same shortest digits.

// ---------------------------------------------------------------------------
// Numbers
// ---------------------------------------------------------------------------

// Go: arshal_default.go:470 makeIntArshaler (marshal)
impl MarshalerTo for i32 {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        enc.push_str(&self.to_string());
        Ok(())
    }
}

// Go: arshal_default.go:470 makeIntArshaler (marshal), for Go `int`
impl MarshalerTo for i64 {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        enc.push_str(&self.to_string());
        Ok(())
    }
}

// Go: arshal_default.go:569 makeUintArshaler (marshal)
impl MarshalerTo for u32 {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        enc.push_str(&self.to_string());
        Ok(())
    }
}

// Go: arshal_default.go:569 makeUintArshaler (marshal)
impl MarshalerTo for u64 {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        enc.push_str(&self.to_string());
        Ok(())
    }
}

// Go: arshal_default.go:659 makeFloatArshaler (marshal)
// NaN and infinities are an error ("nonfinite" format is not used).

// Go: arshal_default.go:492 makeIntArshaler (unmarshal)

// Go: arshal_default.go:591 makeUintArshaler (unmarshal)

// Go: arshal_default.go:591 makeUintArshaler (unmarshal)

// ---------------------------------------------------------------------------
// Pointers, slices, arrays
// ---------------------------------------------------------------------------

// Go: arshal_default.go:1714 makePointerArshaler (marshal): nil writes null.
impl<T: MarshalerTo> MarshalerTo for Option<T> {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        match self {
            None => {
                enc.push_str("null");
                Ok(())
            }
            Some(v) => v.marshal_json_to(enc),
        }
    }
}

// Go: arshal_default.go:1742 makePointerArshaler (unmarshal): null sets nil;
// otherwise a nil pointer gets a new zero value and the value decodes into
// the pointee (merging into an existing one).

// PORT: `Box<T>` is a Go pointer on a type cycle. `null` is handled by the
// enclosing `Option`, so the box only forwards.
impl<T: MarshalerTo + ?Sized> MarshalerTo for Box<T> {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        (**self).marshal_json_to(enc)
    }
}

impl<T: UnmarshalerFrom + ?Sized> UnmarshalerFrom for Box<T> {
    fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError> {
        (**self).unmarshal_json_from(dec)
    }
}

// Go: arshal_default.go:1528 makeSliceArshaler (unmarshal): null sets nil,
// each element starts from its zero value, `[]` sets an empty slice.
// PORT: nil and empty are both an empty `Vec`.

// Go: arshal_default.go:1612 makeArrayArshaler (marshal)

// Go: arshal_default.go:1640 makeArrayArshaler (unmarshal): null sets the
// zero array; the JSON array must have exactly 2 elements.

// ---------------------------------------------------------------------------
// any (LspAny)
// ---------------------------------------------------------------------------

// Go: arshal_any.go:32 marshalValueAny
// PORT: objects write in insertion order (Go map order is random).

// Go: arshal_default.go:1847 makeInterfaceArshaler (unmarshal, `any`)
// `null` sets nil. A nil `any` decodes a fresh value (`unmarshalValueAny`,
// or the typed arshalers when duplicate names are allowed). A non-nil `any`
// decodes into a copy of its dynamic value.
// PORT: `IndexMap<String, V>` decodes with the `packagejson.rs` impl
// (OrderedMap rules), so merging into an existing object with
// `AllowDuplicateNames` replaces values instead of merging them.

// Go: arshal_any.go:64 unmarshalValueAny
// PORT: Go also returns the partial value with an error; callers here stop
// on the first error, so only the error is returned.

// Go: arshal_any.go:177 unmarshalObjectAny

// Go: arshal_any.go:266 unmarshalArrayAny

// Go: arshal_default.go:1847 makeInterfaceArshaler (unmarshal) for a Go
// `any` field held as `Option<Box<dyn AnyValue>>` (`RequestMessage.Params`,
// `ResponseMessage.Result`). `null` sets nil; a nil `any` gets a fresh
// `LspAny`; an existing `LspAny` decodes in place.

// ---------------------------------------------------------------------------
// json.Value
// ---------------------------------------------------------------------------

// Go: jsontext/value.go:238 Value.MarshalJSON, then the encoder writes the
// value with `WriteValue` (compact, canonical strings).

// Go: jsontext/value.go:248 Value.UnmarshalJSON: a copy of the raw input,
// `null` included.

// ---------------------------------------------------------------------------
// Encoder whitespace (compact and multiline)
// ---------------------------------------------------------------------------

// Go jsontext encoder state for `WriteToken` / `reformatValue`: one entry
// per open object or array (is object, number of names and values).
struct TokenWriter<'i> {
    stack: Vec<(bool, usize)>,
    top_len: usize,
    // Some((prefix, indent)) when `Multiline` is set.
    multiline: Option<(&'i str, &'i str)>,
}

impl TokenWriter<'_> {
    fn last(&self) -> (bool, usize) {
        self.stack.last().copied().unwrap_or((false, self.top_len))
    }

    fn increment(&mut self) {
        match self.stack.last_mut() {
            Some(e) => e.1 += 1,
            None => self.top_len += 1,
        }
    }

    // Go: jsontext/state.go:372 NeedIndent
    fn need_indent(&self, next: u8) -> usize {
        let will_end = next == b'}' || next == b']';
        let depth = self.stack.len() + 1;
        let (is_object, len) = self.last();
        let need_object_value = is_object && len % 2 == 1;
        let need_implicit_comma = !need_object_value && len > 0 && !will_end;
        if depth == 1 {
            0 // top-level values are never indented
        } else if len == 0 && will_end {
            0 // an empty object or array is never indented
        } else if len == 0 || need_implicit_comma {
            depth
        } else if will_end {
            depth - 1
        } else {
            0
        }
    }

    // Go: jsontext/encode.go:346 WriteToken with MayAppendDelim
    // (state.go:389), appendWhitespace (encode.go:634) and AppendIndent
    // (encode.go:652).

    // Reads one complete value from `dec` and writes it.
}

// Go: jsontext/encode.go:526 WriteValue (compact output)
// Writes one raw JSON value: whitespace is removed, names must be unique,
// strings are re-quoted in canonical form (invalid UTF-8 becomes U+FFFD, as
// `internal/json.Marshal` sets AllowInvalidUTF8), numbers are copied
// verbatim, and only whitespace may follow the value.

// Go: jsonwire/wire.go:62 QuoteRune
// QuoteRune quotes the first rune in the input.

// ---------------------------------------------------------------------------
// Struct helpers (for hand-written and generated arshalers)
// ---------------------------------------------------------------------------

/// Go v2 struct marshal: `{`.
pub fn write_object_start(enc: &mut String) {
    enc.push('{');
}

/// Go v2 struct marshal: `}`.
pub fn write_object_end(enc: &mut String) {
    enc.push('}');
}





// Go: internal/json/json.go:40 MarshalIndent with
// jsontext.WithIndentPrefix(prefix) and jsontext.WithIndent(indent):
// multiline output, a space after each colon, no space after commas, no
// newline inside an empty object or array, no trailing newline.

#[cfg(test)]
mod unmarshal_error_tests {
    use super::*;

    fn err_text<T: UnmarshalerFrom + Default>(data: &str) -> String {
        let mut v = T::default();
        unmarshal_root(data.as_bytes(), &mut v)
            .expect_err("want an error")
            .to_string()
    }

    // Expected texts come from Go JSON v2 `json.Unmarshal`
    // (go-json-experiment v0.0.0-20260601182631-00ed12fed2a6). Go writes
    // "cannot" or "unable to"; the port writes "cannot".
    #[test]
    fn unmarshal_root_errors_match_go() {
        assert_eq!(
            err_text::<Vec<u32>>(r#"[1,"x"]"#),
            r#"json: cannot unmarshal JSON string into Go uint32 within "/1""#
        );
        assert_eq!(
            err_text::<Vec<u32>>(" {}"),
            "json: cannot unmarshal JSON object into Go []uint32 after offset 1"
        );
        assert_eq!(
            err_text::<[u32; 2]>("[1]"),
            "json: cannot unmarshal JSON array into Go [2]uint32 after offset 2: too few array elements"
        );
        assert_eq!(
            err_text::<LspAny>(r#"{"a":[1,1e999]}"#),
            r#"json: cannot unmarshal JSON number 1e999 into Go float64 within "/a/1": value out of range"#
        );
        assert_eq!(
            err_text::<FxHashMap<String, Vec<u32>>>(r#"{"a/b":[0,-1]}"#),
            r#"json: cannot unmarshal JSON number -1 into Go uint32 within "/a~1b/1": invalid syntax"#
        );
    }

    // The pointer holds Go's bytes of the names (a real U+FDD0 is one
    // char), and `%q` quotes them. Text from Go N (encoding/json/v2 of Go
    // 1.27.1, followups25 round b tools-b/gojsonptr).
    #[test]
    fn pointer_names_are_quoted_from_go_bytes() {
        assert_eq!(
            err_text::<FxHashMap<String, Vec<u32>>>("{\"\u{fdd0}\u{fdd0}\":[0,-1]}"),
            r#"json: cannot unmarshal JSON number -1 into Go uint32 within "/\ufdd0\ufdd0/1": invalid syntax"#
        );
    }
}
