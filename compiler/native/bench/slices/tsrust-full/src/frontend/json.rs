//! Port of json/json.go.
//!
//! Go `internal/json` is a thin wrapper over `github.com/go-json-experiment/json`
//! (JSON v2) and its `jsontext` package. This crate has no JSON dependency, so
//! the parts of v2 that the compiler reads through are ported here by hand:
//!
//! - `JsonDecoder` is the `jsontext.Decoder` token state machine: strict RFC
//!   8259 grammar, `:` and `,` checks in `PeekKind`, a trailing comma is an
//!   error, strings reject control characters and lone surrogates, and object
//!   names are checked for duplicates unless `AllowDuplicateNames` is set or
//!   the arshaler disabled the namespace.
//! - `UnmarshalerFrom` stands in for both Go `json.UnmarshalerFrom` and the
//!   v2 default arshalers. The impls for `String`, `bool`, `f64` and
//!   `FxHashMap<String, V>` follow the v2 default arshalers (null gives the
//!   zero value, a kind mismatch is an error, maps merge into the existing map).
//! - `MarshalerTo` stands in for the v2 default marshalers of the value kinds
//!   the compiler marshals (strings, booleans, slices and ordered maps).
//!
//! PORT: without legacy flags every v2 unmarshal error is fatal
//! (`isFatalError`), so every impl returns the first error. The decoder's
//! errors are Go's `jsontext.SyntacticError` texts (prefix, JSON pointer and
//! offset), which the LSP and the API show (`SyntaxErr`). Most unmarshal
//! errors of the impls in this file are not the v2 texts; no caller shows
//! them. The exceptions are the ones the LSP and the API show: a kind
//! mismatch of a string, a boolean or a number is a v2 `SemanticError`
//! (`json_ext::unmarshal_kind_error`), and `json_unmarshal_decode` gives a
//! method's plain error the Go type (as the v2 arshaler of a type with a
//! method does). `json_ext` has the v2 `SemanticError` texts for the LSP and
//! API types.

use crate::frontend::prelude::*;
use std::borrow::Cow;

/// Go JSON v2 `*json.SemanticError` / `*jsontext.SyntacticError`.
/// PORT: one error type with a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonError {
    pub message: String,
}

impl JsonError {
    fn new(message: impl Into<String>) -> JsonError {
        JsonError {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Go `json.Options` values that the compiler passes.
/// PORT: Go options are opaque values; this enum lists the ones used here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonOption {
    AllowDuplicateNames(bool),
    AllowInvalidUtf8(bool),
}

/// Resolved decoder and encoder flags.
// PORT: `port_form` is not a Go option. A decoded string is a Go string, so
// the decoder writes it in the port form (see
// `scanner_util::GO_STRING_MARKER`): each real U+FDD0 becomes two. With
// `port_form`, the JSON text already holds port form strings (the port's
// own build worker protocol, `append_json_quote_port_form`), and each char
// is kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct JsonOptions {
    pub allow_duplicate_names: bool,
    pub allow_invalid_utf8: bool,
    pub port_form: bool,
}

impl JsonOptions {
    fn from_options(opts: &[JsonOption]) -> JsonOptions {
        let mut o = JsonOptions::default();
        for opt in opts {
            match *opt {
                JsonOption::AllowDuplicateNames(v) => o.allow_duplicate_names = v,
                JsonOption::AllowInvalidUtf8(v) => o.allow_invalid_utf8 = v,
            }
        }
        o
    }
}

/// Go `jsontext.Token`, as returned by `Decoder.ReadToken`.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonToken {
    Null,
    False,
    True,
    String(String),
    /// The raw number text.
    Number(String),
    BeginObject,
    EndObject,
    BeginArray,
    EndArray,
}

impl JsonToken {
    /// Go `jsontext.Token.Kind`.
    #[must_use]
    pub fn kind(&self) -> u8 {
        match self {
            JsonToken::Null => b'n',
            JsonToken::False => b'f',
            JsonToken::True => b't',
            JsonToken::String(_) => b'"',
            JsonToken::Number(_) => b'0',
            JsonToken::BeginObject => b'{',
            JsonToken::EndObject => b'}',
            JsonToken::BeginArray => b'[',
            JsonToken::EndArray => b']',
        }
    }
}

// Go jsontext `maxNestingDepth`.
const MAX_NESTING_DEPTH: usize = 10000;

// One open JSON object or array (Go jsontext `stateEntry` plus the
// namespace of seen object names).
struct Frame<'a> {
    is_object: bool,
    len: usize,
    // `None` when duplicate names are allowed or the namespace is disabled.
    // PERF: a name is its unquoted text before the port form conversion
    // (`scan_string`), which is one to one, so equal names are still equal.
    // A name with no escape borrows the input.
    names: Option<FxHashSet<Cow<'a, str>>>,
    // The offset of the opening quote of the last object name read in this
    // object (Go `objectNameStack`). Only the error texts read it.
    name: usize,
}

/// PORT: not in Go (perf). A `JsonToken` whose text borrows the input
/// where it can (`JsonDecoder::read_token_ref`): a string with no escape
/// and no port form change, and each number.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonTokenRef<'a> {
    Null,
    False,
    True,
    String(Cow<'a, str>),
    /// The raw number text.
    Number(&'a str),
    BeginObject,
    EndObject,
    BeginArray,
    EndArray,
}

/// A token of `JsonDecoder::next_token`: a `JsonToken` whose string is the
/// unquoted text before the port form conversion, and whose number is the
/// raw text, both borrowed from the input when they can be.
enum RawToken<'a> {
    Null,
    False,
    True,
    String(Cow<'a, str>),
    Number(&'a str),
    BeginObject,
    EndObject,
    BeginArray,
    EndArray,
}

/// Go `jsontext.Decoder` over a complete input buffer.
pub struct JsonDecoder<'a> {
    buf: &'a [u8],
    // Go `prevEnd`: the end of the last read token or value.
    pos: usize,
    // Open containers. The top-level virtual array is `top_len`.
    stack: Vec<Frame<'a>>,
    top_len: usize,
    pub options: JsonOptions,
    // PERF: the name sets of closed objects, empty, for the next objects.
    spare_names: Vec<FxHashSet<Cow<'a, str>>>,
}

fn is_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\r' | b'\n')
}

// Go `jsontext.Kind.normalize`.
fn normalize_kind(c: u8) -> u8 {
    match c {
        b'-' | b'0'..=b'9' => b'0',
        _ => c,
    }
}

fn hex_val(c: u8) -> Option<u32> {
    match c {
        b'0'..=b'9' => Some(u32::from(c - b'0')),
        b'a'..=b'f' => Some(u32::from(c - b'a' + 10)),
        b'A'..=b'F' => Some(u32::from(c - b'A' + 10)),
        _ => None,
    }
}

// Decodes one UTF-8 character at the start of `b`.

impl<'a> JsonDecoder<'a> {
    #[must_use]
    pub fn new(buf: &'a [u8], options: JsonOptions) -> JsonDecoder<'a> {
        JsonDecoder {
            buf,
            pos: 0,
            stack: Vec::new(),
            top_len: 0,
            options,
            spare_names: Vec::new(),
        }
    }

    fn skip_ws(&self, mut p: usize) -> usize {
        while p < self.buf.len() && is_ws(self.buf[p]) {
            p += 1;
        }
        p
    }

    fn last_len(&self) -> usize {
        self.stack.last().map_or(self.top_len, |f| f.len)
    }

    fn last_is_object(&self) -> bool {
        self.stack.last().is_some_and(|f| f.is_object)
    }

    fn need_object_value(&self) -> bool {
        self.last_is_object() && self.last_len() % 2 == 1
    }


    fn increment(&mut self) {
        match self.stack.last_mut() {
            Some(f) => f.len += 1,
            None => self.top_len += 1,
        }
    }

    // Go jsontext `stateMachine.needDelim`.
    fn need_delim(&self, next: u8) -> u8 {
        if self.need_object_value() {
            b':'
        } else if self.last_len() > 0 && next != b'}' && next != b']' && !self.stack.is_empty() {
            b','
        } else {
            0
        }
    }

    // Go jsontext `decoderState.PeekKind` without the cache: the position and
    // normalized kind of the next token, after whitespace and one delimiter.
    // The errors are Go's (the decoder reads a complete buffer, so Go's
    // `fetch` fails at its end with `io.ErrUnexpectedEOF`).


    /// Go `jsontext.Decoder.InputOffset`: the end of the last token or value
    /// read.
    #[must_use]
    pub fn input_offset(&self) -> usize {
        self.pos
    }

    /// Go `jsontext.Decoder.DisableNamespace` (the export helper
    /// `Tokens.Last.DisableNamespace`): stop duplicate-name checks for the
    /// innermost open object.
    pub fn disable_namespace(&mut self) {
        if let Some(names) = self.stack.last_mut().and_then(|f| f.names.take()) {
            self.keep_names(names);
        }
    }

    /// Keeps the emptied name set `names` for a later object.
    fn keep_names(&mut self, mut names: FxHashSet<Cow<'a, str>>) {
        names.clear();
        self.spare_names.push(names);
    }

    /// Go `jsontext.Decoder.StackDepth` plus the length of the innermost
    /// container. Used to check that an unmarshaler reads exactly one value.
    fn depth_length(&self) -> (usize, usize) {
        (self.stack.len(), self.last_len())
    }

    // Go `stateMachine.appendLiteral` / `appendNumber` for the token that
    // starts at `start`.








    // Go `jsonwire.ConsumeString` plus unquoting. Returns the end offset and
    // the unquoted text, before the port form conversion (`string_value`).
    // PERF: a string with no escape and no control character whose bytes
    // are valid UTF-8 is its own unquoted text, so it is borrowed. Any
    // other string, and each error, goes through `consume_string_chars`.


    // Go `jsonwire.ConsumeStringResumable` (with `validateUTF8` unless
    // `allow_invalid_utf8`) plus unquoting. An error is Go's error and its
    // offset (`consumeString`: at the start of a cut escape, and at the end
    // of the input when the string is cut).

    // Go `jsonwire.ConsumeNumber`: `-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?`.
}

// ---------------------------------------------------------------------------
// Go jsontext syntactic errors
// ---------------------------------------------------------------------------

/// The `Err` of a Go `jsontext.SyntacticError` that the decoder makes.
#[derive(Clone, Debug, PartialEq, Eq)]
enum SyntaxErr {
    /// Go `jsonwire.InvalidTextError`: its `Label`, `What` (the raw input
    /// text) and `Where`.
    InvalidText {
        label: &'static str,
        what: Vec<u8>,
        at: String,
    },
    /// Go `io.ErrUnexpectedEOF`.
    UnexpectedEof,
    /// Go `jsonwire.ErrInvalidUTF8`.
    InvalidUtf8,
    /// Go `jsontext.ErrDuplicateName`.
    DuplicateName,
    /// Go `jsontext.ErrNonStringName`.
    NonStringName,
    /// Go jsontext `errMissingValue`.
    MissingValue,
    /// Go jsontext `errMismatchDelim`.
    MismatchDelim,
    /// Go jsontext `errMaxDepth`.
    MaxDepth,
}

/// A `SyntaxErr` of Go's `consumeValue`, its offset, and the pointer
/// tokens (outermost first) that Go's `pointerSuffixError` adds.
type ConsumeError = (usize, SyntaxErr, Vec<String>);

impl SyntaxErr {
    // Go `err.Error()`.
}

// Go: jsonwire/wire.go:129 NewInvalidCharacterError: the first rune of
// `buf[pos..]` (one byte when it is not valid UTF-8).

// Go: jsonwire/wire.go:134 NewInvalidEscapeSequenceError
fn invalid_escape(what: &[u8]) -> SyntaxErr {
    SyntaxErr::InvalidText {
        label: if what.len() > 6 {
            "surrogate pair"
        } else {
            "escape sequence"
        },
        what: what.to_vec(),
        at: "in string".to_string(),
    }
}

// Go: jsonwire/wire.go:146 (*InvalidTextError).Error

// Go `utf8.DecodeRune(b)` size: 0 for no input, 1 for a byte that does not
// start a valid rune.

// Go: utf8.FullRune
fn go_full_rune(b: &[u8]) -> bool {
    let Some(&c) = b.first() else {
        return false;
    };
    let (need, lo, hi) = match c {
        0xC2..=0xDF => (2, 0x80, 0xBF),
        0xE0 => (3, 0xA0, 0xBF),
        0xE1..=0xEC | 0xEE..=0xEF => (3, 0x80, 0xBF),
        0xED => (3, 0x80, 0x9F),
        0xF0 => (4, 0x90, 0xBF),
        0xF1..=0xF3 => (4, 0x80, 0xBF),
        0xF4 => (4, 0x80, 0x8F),
        // ASCII, or a byte that cannot start a rune.
        _ => return true,
    };
    if b.len() >= need {
        return true;
    }
    // Short or invalid.
    (b.len() > 1 && !(lo..=hi).contains(&b[1])) || (b.len() > 2 && !(0x80..=0xBF).contains(&b[2]))
}

// Go: jsonwire/wire.go:62 QuoteRune

// Go: jsonwire/decode.go:299 hasEscapedUTF16Prefix
fn has_escaped_utf16_prefix(b: &[u8], lower_surrogate_half: bool) -> bool {
    for (i, &c) in b.iter().enumerate() {
        let ok = match i {
            0 => c == b'\\',
            1 => c == b'u',
            2 if lower_surrogate_half => c == b'd' || c == b'D',
            3 if lower_surrogate_half => matches!(c, b'c'..=b'f' | b'C'..=b'F'),
            _ => true,
        };
        if !ok || ((2..6).contains(&i) && hex_val(c).is_none()) {
            return false;
        }
    }
    true
}

// Go: jsonwire parseHexUint16

// Go `ConsumeLiteral` of the literal `lit` at `p`, which is not all there:
// the offset and error of `consumeLiteral`.

// Go `jsonwire.ConsumeNumberResumable` through `consumeNumber`: the end
// offset, or the offset and error. A number that the input cuts short is
// `io.ErrUnexpectedEOF` at the start of the number.

// Go `appendEscapePointerName`: a JSON pointer token for an object name.
fn escape_pointer_token(name: &str) -> String {
    let mut b = String::with_capacity(name.len());
    for c in name.chars() {
        // Per RFC 6901, section 3, escape '~' and '/' characters.
        match c {
            '~' => b.push_str("~0"),
            '/' => b.push_str("~1"),
            c => b.push(c),
        }
    }
    b
}


// The pointer token of the object name `quoted` (`unquote_pointer_name`).

// Go: jsontext/errors.go:120 (*SyntacticError).Error
// PORT: the pointer is Go's bytes (`unquote_pointer_name`), so `%q` is
// `quote_bytes`.

impl JsonDecoder<'_> {
    // Go: jsontext/state.go:180 appendStackPointer

    // The JSON string (with its quotes) that starts at `p`.




    // Go `ReadToken`/`ReadValue`/`PeekKind` when the input ends before the
    // next token. At the top level Go returns `io.EOF`, which
    // `json.Unmarshal` (`unmarshalFull`) turns into a `SyntacticError` of
    // `io.ErrUnexpectedEOF` at the end of the input, with no pointer.

    // Go `checkDelimBeforeIOError`: the input ends after `delim`. A string
    // can always come next, so `delim` is checked against one.

    // Go: jsontext/decode.go:391 checkDelim, for a `delim` that `next` does
    // not allow. The error is at the delimiter (or `next` when there is
    // none).

    // The Go error of a `ReadValue` of the value at `start`, which failed
    // in the port with `err`. The state goes back to `before` (the offset,
    // the length and the last name of the innermost container at `depth`).
    // An object or array is checked again as Go's `consumeValue` does, which
    // reads it whole before the state check; a `}` or `]` is Go's
    // `consumeValue` error; any other value fails as it does in
    // `ReadToken`, so `err` is Go's.

    // Go: jsontext/decode.go:857 consumeValue with consumeObject and
    // consumeArray, for the value at `start`, whose `depth` is Go's (the
    // stack depth plus one). Gives the end of the value, or Go's error, its
    // offset and pointer suffix.
    // PORT: Go recurses; this keeps its own stack of open containers.
}

/// Go `json.UnmarshalerFrom` and the JSON v2 default unmarshalers.
pub trait UnmarshalerFrom {
    fn unmarshal_json_from(&mut self, dec: &mut JsonDecoder<'_>) -> Result<(), JsonError>;
}

/// Go `json.MarshalerTo` and the JSON v2 default marshalers.
/// PORT: the encoder is the output string. Output is compact.
pub trait MarshalerTo {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError>;
}

// Go v2 string arshaler: null gives "", a string sets the value, any other
// kind is an error after the value is read.

// Go v2 bool arshaler.

// Go v2 float64 arshaler. An out-of-range number sets ±Inf and is an error,
// like Go `strconv.ParseFloat`.

// Go v2 map arshaler for `map[string]V`. The map merges into the existing
// map. A name that is already in the map is a duplicate only if it came from
// this JSON object (the `seen` set), unless `AllowDuplicateNames` is set.

// Go v2 string marshaler: escapes `"`, `\` and control characters only.






impl MarshalerTo for bool {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        enc.push_str(if *self { "true" } else { "false" });
        Ok(())
    }
}

// Go v2 slice marshaler. PORT: Go marshals a nil slice as `[]` in v2 too
// (`FormatNilSliceAsNull` is off by default), so there is no null case.
impl<T: MarshalerTo> MarshalerTo for [T] {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        enc.push('[');
        for (i, v) in self.iter().enumerate() {
            if i > 0 {
                enc.push(',');
            }
            v.marshal_json_to(enc)?;
        }
        enc.push(']');
        Ok(())
    }
}

impl<T: MarshalerTo> MarshalerTo for Vec<T> {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        self.as_slice().marshal_json_to(enc)
    }
}

// Go `collections.OrderedMap.MarshalJSONTo`: members in insertion order.

impl<T: MarshalerTo + ?Sized> MarshalerTo for &T {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        (**self).marshal_json_to(enc)
    }
}

// Go: json/json.go:11 allowInvalid
// PORT: the Go slice of options is a constant list.
const ALLOW_INVALID: &[JsonOption] = &[JsonOption::AllowInvalidUtf8(true)];

// Go: json/json.go:13 Marshal
// PORT: named `json_marshal` so the glob export stays unambiguous. Go returns
// bytes; the output here is always UTF-8 text. With `AllowInvalidUTF8`, Go
// writes U+FFFD for each invalid byte of a string, which the string
// marshaler does for the port form (`append_json_quote`).
pub fn json_marshal<T: MarshalerTo + ?Sized>(
    input: &T,
    opts: &[JsonOption],
) -> Result<String, JsonError> {
    let mut all: Vec<JsonOption> = ALLOW_INVALID.to_vec();
    all.extend_from_slice(opts);
    let _ = JsonOptions::from_options(&all);
    let mut out = String::new();
    input.marshal_json_to(&mut out)?;
    json_check_nesting_depth(&out)?;
    Ok(out)
}

// Go: jsontext/state.go:302 pushObject and :337 pushArray, the
// `len(m.Stack) == maxNestingDepth` case (errMaxDepth, tsgo pin B
// go-json-experiment/json v0.0.0-20260623181947-01eb4420fa68).
// PORT: the Go encoder checks each `{` and `[` as it writes it, so a value
// nested more than `maxNestingDepth` (10000) levels deep fails to marshal.
// The Rust marshalers write the compact text with no state, so this checks
// the finished text instead: the result is the same error and no output.
// A value that deep has more than 10000 `{` and `[` bytes, so a text with
// fewer is not scanned. The error text is Go's `errMaxDepth` text without
// the v2 wrapping (see the file header).
fn json_check_nesting_depth(compact: &str) -> Result<(), JsonError> {
    if compact.len() <= 2 * MAX_NESTING_DEPTH
        || crate::memchr::memchr2_iter(b'{', b'[', compact.as_bytes()).count() <= MAX_NESTING_DEPTH
    {
        return Ok(());
    }
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for &c in compact.as_bytes() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_string = false;
            }
            continue;
        }
        match c {
            b'"' => in_string = true,
            b'{' | b'[' => {
                if depth == MAX_NESTING_DEPTH {
                    return Err(JsonError::new("exceeded max depth"));
                }
                depth += 1;
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

// Go: json/json.go:22 MarshalEncode
// PORT: not ported. Only the LSP, API and build-info writers use it.

// Go: json/json.go:31 MarshalWrite
// PORT: Go writes through a streaming `jsontext.Encoder` that flushes as it
// goes, so a marshal error can leave partial output. This marshals the whole
// value first (`json_marshal`) and writes nothing on a marshal error. Like Go
// (`jsonflags.OmitTopLevelNewline` in v2 `MarshalWrite`), the output has no
// trailing newline. A write error is returned as a `JsonError`.
pub fn json_marshal_write<T: MarshalerTo + ?Sized>(
    out: &mut dyn std::io::Write,
    input: &T,
    opts: &[JsonOption],
) -> Result<(), JsonError> {
    let b = json_marshal(input, opts)?;
    out.write_all(b.as_bytes())
        .map_err(|err| JsonError::new(err.to_string()))
}

// Go: json/json.go:40 MarshalIndent
// PORT: the Rust marshalers write compact output. Go passes
// `jsontext.WithIndentPrefix` and `jsontext.WithIndent`, which make the
// encoder add whitespace before each token as it writes it
// (`encoderState.WriteToken` and `WriteValue` call `MayAppendDelim` and
// `appendWhitespace`). That whitespace depends only on the token sequence,
// so this marshals compact output and replays its tokens through the same
// rules (`json_append_multiline`).

// Go: jsontext/options.go:232 WithIndent and :265 WithIndentPrefix
// (the character checks). Go panics when the value holds anything but
// spaces and tabs.
// PORT: Go quotes the first bad rune with `jsonwire.QuoteRune`
// (`strconv.QuoteRune`); Rust `{:?}` gives the same `'x'` form for
// printable characters.

/// Go `jsontext.stateEntry`, reduced to what the encoder whitespace reads:
/// the nesting type and the number of tokens written at this level (object
/// names and values both count).
#[derive(Clone, Copy, Debug)]
struct JsonMultilineEntry {
    is_object: bool,
    length: i64,
}

impl JsonMultilineEntry {
    // Go: jsontext/state.go:480 needImplicitColon
    fn need_implicit_colon(self) -> bool {
        self.need_object_value()
    }

    // Go: jsontext/state.go:486 needObjectValue
    fn need_object_value(self) -> bool {
        self.is_object && self.length % 2 == 1
    }

    // Go: jsontext/state.go:493 needImplicitComma
    fn need_implicit_comma(self, next: u8) -> bool {
        !self.need_object_value() && self.length > 0 && next != b'}' && next != b']'
    }
}

/// Go `jsontext.stateMachine`. `last` starts as the top-level virtual array
/// (Go `stateMachine.reset`).
#[derive(Debug)]
struct JsonMultilineState {
    stack: Vec<JsonMultilineEntry>,
    last: JsonMultilineEntry,
}

impl JsonMultilineState {
    // Go: jsontext/state.go:249 Depth
    fn depth(&self) -> usize {
        self.stack.len() + 1
    }

    // Go: jsontext/state.go:372 NeedIndent
    /// NeedIndent reports whether indent whitespace should be injected.
    /// A zero value means that no whitespace should be injected.
    /// A positive value means '\n', indentPrefix, and (n-1) copies of indentBody
    /// should be appended to the output immediately before the next token.
    fn need_indent(&self, next: u8) -> usize {
        let will_end = next == b'}' || next == b']';
        if self.depth() == 1 {
            0 // top-level values are never indented
        } else if self.last.length == 0 && will_end {
            0 // an empty object or array is never indented
        } else if self.last.length == 0 || self.last.need_implicit_comma(next) {
            self.depth()
        } else if will_end {
            self.depth() - 1
        } else {
            0
        }
    }

    // Go: jsontext/state.go:389 MayAppendDelim
    /// MayAppendDelim appends a colon or comma that may precede the next token.
    fn may_append_delim(&self, b: &mut String, next: u8) {
        if self.last.need_implicit_colon() {
            b.push(':');
        } else if self.last.need_implicit_comma(next) && !self.stack.is_empty() {
            // comma not needed for top-level values
            b.push(',');
        }
    }

    // Go: jsontext/state.go:403 needDelim
    /// needDelim reports whether a colon or comma token should be implicitly emitted
    /// before the next token of the specified kind.
    /// A zero value means no delimiter should be emitted.
    fn need_delim(&self, next: u8) -> u8 {
        if self.last.need_implicit_colon() {
            b':'
        } else if self.last.need_implicit_comma(next) && !self.stack.is_empty() {
            // comma not needed for top-level values
            b','
        } else {
            0
        }
    }

    // Go: jsontext/state.go:270 appendLiteral, :284 appendString and :296
    // appendNumber
    fn increment(&mut self) {
        self.last.length += 1;
    }

    // Go: jsontext/state.go:302 pushObject and :337 pushArray
    fn push(&mut self, is_object: bool) {
        self.last.length += 1;
        self.stack.push(self.last);
        self.last = JsonMultilineEntry {
            is_object,
            length: 0,
        };
    }

    // Go: jsontext/state.go:320 popObject and :355 popArray
    fn pop(&mut self) -> Result<(), JsonError> {
        self.last = self
            .stack
            .pop()
            .ok_or_else(|| JsonError::new("mismatching structural token"))?;
        Ok(())
    }
}

// Go: jsontext/encode.go:634 appendWhitespace
/// appendWhitespace appends whitespace that immediately precedes the next token.
// PORT: the flags are the ones `jsonopts.Struct.InitializeMultiline` sets for
// `WithIndentPrefix`/`WithIndent`: `SpaceAfterColon` on, `SpaceAfterComma`
// off, `Multiline` on.
fn json_append_whitespace(
    m: &JsonMultilineState,
    b: &mut String,
    next: u8,
    prefix: &str,
    indent: &str,
) {
    if m.need_delim(next) == b':' {
        // SpaceAfterColon
        b.push(' ');
    } else {
        // SpaceAfterComma is off; Multiline is on.
        json_append_indent(b, m.need_indent(next), prefix, indent);
    }
}

// Go: jsontext/encode.go:652 AppendIndent
/// AppendIndent appends the appropriate number of indentation characters
/// for the current nested level, n.
fn json_append_indent(b: &mut String, mut n: usize, prefix: &str, indent: &str) {
    if n == 0 {
        return;
    }
    b.push('\n');
    b.push_str(prefix);
    while n > 1 {
        b.push_str(indent);
        n -= 1;
    }
}

// PORT: the multiline encoder pass. It reads each token of the compact
// output, drops the compact `:` and `,` (and any whitespace, like Go
// `reformatValue`), and writes the token the way Go `WriteToken` does:
// `MayAppendDelim`, then `appendWhitespace`, then the token bytes, then the
// state machine update. Top-level output has no trailing newline, as in Go
// `json.Marshal`.

/// The size classes of the Go allocator (internal/runtime/gc/sizeclasses.go
/// `SizeClassToSize`, go1.27.1).
const GO_SIZE_CLASSES: [usize; 67] = [
    8, 16, 24, 32, 48, 64, 80, 96, 112, 128, 144, 160, 176, 192, 208, 224, 240, 256, 288, 320, 352,
    384, 416, 448, 480, 512, 576, 640, 704, 768, 896, 1024, 1152, 1280, 1408, 1536, 1792, 2048,
    2304, 2688, 3072, 3200, 3456, 4096, 4864, 5376, 6144, 6528, 6784, 6912, 8192, 9472, 9728,
    10240, 10880, 12288, 13568, 14336, 16384, 18432, 19072, 20480, 21760, 24576, 27264, 28672,
    32768,
];

// Go: runtime/slice.go growslice (`nextslicecap`) and runtime/msize.go
// roundupsize, for a `[]byte` (noscan)

/// Where Go's streaming `jsontext.Encoder` (v2 `MarshalWrite` to an
/// `io.Writer` that is not a `bytes.Buffer`) writes its buffer out, for
/// `json_marshal_indent_write`. Its buffer starts empty and grows by Go
/// `append`. After each token it flushes when the value is complete or the
/// buffer is more than 3/4 full (`NeedFlush`), except where it may have to
/// take back an empty member (`avoidFlush`). After a write it doubles a
/// buffer of up to 2 KiB that is less than half the output so far.
// Go: jsontext/encode.go NeedFlush, Flush and avoidFlush, the appends of
// WriteToken, AppendIndent and jsonwire.AppendQuote (go1.27.1).
#[derive(Default)]
struct JsonStreamPieces {
    /// `cap(e.Buf)`.
    cap: usize,
    /// The output offset where `e.Buf` starts.
    start: usize,
    /// `len(e.Buf)` while a token is appended.
    len: usize,
    /// The output offset where each write ends.
    ends: Vec<usize>,
}

impl JsonStreamPieces {




    /// Go `if e.NeedFlush() { e.Flush() }` after a token, with `b` the output
    /// so far.
    fn after_token(&mut self, b: &str, m: &JsonMultilineState) {
        let len = b.len() - self.start;
        if m.depth() != 1 && len <= 3 * self.cap / 4 {
            return;
        }
        let last = m.last;
        let avoid = last.length == 0
            || last.need_object_value()
            || (last.is_object
                && last.length % 2 == 0
                && len >= 2
                && matches!(
                    &b.as_bytes()[b.len() - 2..],
                    b"ll" | b"\"\"" | b"{}" | b"[]"
                ));
        if avoid {
            return;
        }
        self.ends.push(b.len());
        self.start = b.len();
        if self.cap <= 2048 && self.cap < b.len() / 2 {
            self.cap *= 2;
        }
    }
}


// Go: json/json.go:48 MarshalIndentWrite
// Used by execute/tsc.go:405 showConfig (prefix "", indent four spaces).
// PORT: the indented output comes from `json_marshal_indent` (see the
// PORT notes there and on `json_marshal_write`). There is no trailing
// newline. It goes out in the writes of Go's streaming encoder
// (`JsonStreamPieces`), after the whole value is marshaled.

// Go: json/json.go:56 Unmarshal
// PORT: `out` implements `UnmarshalerFrom` in place of Go reflection. Like
// Go, `out` is not reset on error, and the input must hold exactly one value.

// Go: json/json.go:60 UnmarshalDecode
// PORT: Go merges `opts` into the decoder options; callers here pass none,
// so the decoder options apply. The v2 method arshaler is kept: a plain
// error of the `UnmarshalerFrom` of `T` gets the Go type of `T`
// (`json_ext::wrap_method_error`), as the v2 arshaler of a type with an
// `UnmarshalJSONFrom` method does, and an error that already has its type
// keeps it. The method must read exactly one value.

// Go: json/json.go:64 UnmarshalRead
// PORT: not ported. Only the LSP and API readers use it.

// Go: json/json.go:68 AllowDuplicateNames
#[must_use]
pub fn json_allow_duplicate_names(allow: bool) -> JsonOption {
    JsonOption::AllowDuplicateNames(allow)
}

// Go: json/json.go:72 Deterministic
// Go: json/json.go:76 WithIndent
// PORT: not ported. Map output order is only needed by build-info and
// baseline writers. Indentation is available through `json_marshal_indent`.

// Go: json/json.go:80 NewDecoder
// PORT: Go reads from an `io.Reader`; the Rust decoder reads a complete
// buffer with default options.
#[must_use]
pub fn json_new_decoder(r: &[u8]) -> JsonDecoder<'_> {
    JsonDecoder::new(r, JsonOptions::default())
}

/// A decoder for the port's own protocol text, whose strings are already
/// in the port form (see `JsonOptions::port_form`).
#[must_use]
pub fn json_new_port_form_decoder(r: &[u8]) -> JsonDecoder<'_> {
    JsonDecoder::new(
        r,
        JsonOptions {
            port_form: true,
            ..JsonOptions::default()
        },
    )
}

// Go: json/json.go:84 type aliases (Value, Kind, UnmarshalerFrom,
// MarshalerTo, Decoder, Encoder) and json/json.go:94 token values
// (BeginObject, EndObject, Null, BeginArray, EndArray).
// PORT: `JsonToken` variants and `JsonToken::kind` stand in for the token
// values. `Value` is `&[u8]`, `Kind` is `u8`, `Decoder` is `JsonDecoder`.

#[cfg(test)]
mod marshal_depth_tests {
    use super::*;

    /// A value that writes `self.0` nested arrays, with a string that holds
    /// brackets in the middle.
    struct Nested(usize);

    impl MarshalerTo for Nested {
        fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
            for _ in 0..self.0 {
                enc.push('[');
            }
            enc.push_str(r#""[{\"[""#);
            for _ in 0..self.0 {
                enc.push(']');
            }
            Ok(())
        }
    }

    // Go JSON v2 `json.Marshal` accepts 10000 nested arrays and fails with
    // `exceeded max depth` for 10001 (jsontext `maxNestingDepth`).
    #[test]
    fn marshal_fails_past_max_nesting_depth() {
        assert!(json_marshal(&Nested(MAX_NESTING_DEPTH), &[]).is_ok());
        let err = json_marshal(&Nested(MAX_NESTING_DEPTH + 1), &[]).unwrap_err();
        assert_eq!(err.message, "exceeded max depth");
        let mut out: Vec<u8> = Vec::new();
        assert!(json_marshal_write(&mut out, &Nested(MAX_NESTING_DEPTH + 1), &[]).is_err());
        assert!(out.is_empty());
    }

    // Go jsontext decodes 10000 nested arrays and fails at the next `[`
    // with the pointer "/0" x 10000 and the offset 10000 (coder_test.go
    // "ArraysInvalid"). The text cuts the pointer to about 100 bytes
    // (jsonwire.TruncatePointer).
    #[test]
    fn decode_fails_past_max_nesting_depth() {
        let depth = MAX_NESTING_DEPTH + 1;
        let input = format!("{}{}", "[".repeat(depth), "]".repeat(depth));
        let mut dec = JsonDecoder::new(input.as_bytes(), JsonOptions::default());
        let err = dec.skip_value().unwrap_err();
        let pointer = format!("{}/…/0{}", "/0".repeat(24), "/0".repeat(24));
        assert_eq!(
            err.message,
            format!("jsontext: exceeded max depth within \"{pointer}\" after offset 10000")
        );
    }
}

#[cfg(test)]
mod syntax_error_tests {
    use super::*;

    // Go N texts (encoding/json/v2 of Go 1.27, tsgo pin 673a5f17d713):
    // `json.Unmarshal` into a `jsontext.Value` (Go `ReadValue`, then the end
    // of input check) and `jsontext.Decoder.SkipValue` (Go `ReadToken` for
    // an object or array). `None` is the same text as `ReadValue`.
    #[test]
    fn syntax_errors_match_go() {
        let cases: [(&[u8], &str, Option<&str>); 18] = [
            (
                b"nulx",
                "jsontext: invalid character 'x' in literal null (expecting 'l') after offset 3",
                None,
            ),
            (
                b"1.x",
                "jsontext: invalid character 'x' in number (expecting digit) after offset 2",
                None,
            ),
            (b"-", "jsontext: unexpected EOF", None),
            (
                br#""a\u12x4""#,
                "jsontext: invalid escape sequence `\\u12x4` in string after offset 2",
                None,
            ),
            (
                br#""\ud800\u0041""#,
                "jsontext: invalid surrogate pair `\\ud800\\u0041` in string after offset 1",
                None,
            ),
            (
                b"\"a\x01\"",
                "jsontext: invalid character '\\x01' in string (expecting non-control character) after offset 2",
                None,
            ),
            (b"\"\xff\"", "jsontext: invalid UTF-8 after offset 1", None),
            (
                br#"{"a":1,}"#,
                "jsontext: invalid character '}' at start of string (expecting '\"') after offset 7",
                Some("jsontext: invalid character ',' at start of value after offset 6"),
            ),
            (
                br#"{"a" 1}"#,
                "jsontext: invalid character '1' after object name (expecting ':') within \"/a\" after offset 5",
                None,
            ),
            (
                br#"{"a":1:2}"#,
                "jsontext: invalid character ':' after object value (expecting ',' or '}') after offset 6",
                None,
            ),
            (
                b"[1,]",
                "jsontext: invalid character ']' at start of value within \"/1\" after offset 3",
                Some("jsontext: invalid character ',' at start of value after offset 2"),
            ),
            (
                br#"{"a":[}"#,
                "jsontext: invalid character '}' at start of value within \"/a/0\" after offset 6",
                None,
            ),
            (
                br#"{1:2}"#,
                "jsontext: invalid character '1' at start of string (expecting '\"') after offset 1",
                Some("jsontext: object member name must be a string after offset 1"),
            ),
            (
                br#"{"a":}"#,
                "jsontext: invalid character '}' at start of value within \"/a\" after offset 5",
                Some("jsontext: missing value after object name within \"/a\" after offset 5"),
            ),
            (
                br#"{"a":1} x"#,
                "jsontext: invalid character 'x' after top-level value after offset 8",
                None,
            ),
            (b" ", "jsontext: unexpected EOF after offset 1", None),
            (
                br#"{"a/b~c":{"d":[1,2,x]}}"#,
                "jsontext: invalid character 'x' at start of value within \"/a~1b~0c/d/2\" after offset 19",
                None,
            ),
            (
                b"[1,\xe2\x82",
                "jsontext: invalid character '\\xe2' at start of value within \"/1\" after offset 3",
                None,
            ),
        ];
        for (input, value, tokens) in cases {
            let mut dec = JsonDecoder::new(input, JsonOptions::default());
            let err = dec.read_value().and_then(|_| dec.check_eof()).unwrap_err();
            assert_eq!(err.message, value, "ReadValue of {input:?}");
            if input.ends_with(b" x") {
                continue;
            }
            let mut dec = JsonDecoder::new(input, JsonOptions::default());
            let err = dec.skip_value().unwrap_err();
            assert_eq!(
                err.message,
                tokens.unwrap_or(value),
                "SkipValue of {input:?}"
            );
        }
    }
}

#[cfg(test)]
mod marshal_indent_tests {
    use super::*;

    // Expected values come from Go JSON v2 `json.Marshal` with
    // `jsontext.WithIndentPrefix` and `jsontext.WithIndent`.
    #[test]
    fn multiline_output_matches_go() {
        assert_eq!(
            json_append_multiline(r#"["x",true,[],{},["a",["b"]]]"#, "", "  ").unwrap(),
            "[\n  \"x\",\n  true,\n  [],\n  {},\n  [\n    \"a\",\n    [\n      \"b\"\n    ]\n  ]\n]"
        );
        assert_eq!(
            json_append_multiline(r#"{"a":["x","y:{"],"b":[],"c":[[true],[]]}"#, " ", "\t")
                .unwrap(),
            "{\n \t\"a\": [\n \t\t\"x\",\n \t\t\"y:{\"\n \t],\n \t\"b\": [],\n \t\"c\": [\n \t\t[\n \t\t\ttrue\n \t\t],\n \t\t[]\n \t]\n }"
        );
        assert_eq!(json_marshal_indent("s", "", "  ").unwrap(), "\"s\"");
    }

    /// The compact `--showConfig` value of a config with 31 files (one name
    /// needs an escape).
    const SHOW_CONFIG_31: &str = r#"{"compilerOptions":{"lib":["es2022","dom"],"module":"nodenext","outDir":"./out","paths":{"@a/*":["./src/*"]},"strict":true,"target":"es2022","moduleResolution":"nodenext","moduleDetection":"force"},"files":["./m1.ts","./m10.ts","./m11.ts","./m12.ts","./m13.ts","./m14.ts","./m15.ts","./m16.ts","./m17.ts","./m18.ts","./m19.ts","./m2.ts","./m20.ts","./m21.ts","./m22.ts","./m23.ts","./m24.ts","./m25.ts","./m26.ts","./m27.ts","./m28.ts","./m29.ts","./m3.ts","./m30.ts","./m4.ts","./m5.ts","./m6.ts","./m7.ts","./m8.ts","./m9.ts","./q\"x.ts"],"exclude":["out"]}"#;

    // PORT: not in Go. The writes of Go's streaming encoder for
    // `SHOW_CONFIG_31`, recorded with strace from the pin N oracle: 83 149
    // 178 299 294 46 bytes.
    #[test]
    fn json_stream_pieces_match_go_show_config_writes() {
        let mut pieces = JsonStreamPieces::default();
        let out =
            json_append_multiline_pieces(SHOW_CONFIG_31, "", "    ", Some(&mut pieces)).unwrap();
        assert_eq!(out.len(), 1049);
        assert_eq!(pieces.ends, [83, 232, 410, 709, 1003, 1049]);
    }

    // The write loop of `json_marshal_indent_write`: one write of the
    // writer per piece, so `--showConfig` writes fd 1 in Go's pieces (the
    // system writer writes each one at once, `stdio::CliStdout`).
    #[test]
    fn json_marshal_indent_write_writes_go_pieces() {
        /// A value whose compact text is given.
        struct Compact(&'static str);
        impl MarshalerTo for Compact {
            fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
                enc.push_str(self.0);
                Ok(())
            }
        }
        /// Keeps each write as one piece, an empty one too, as the system
        /// writer writes each one (`GoOutput`).
        #[derive(Default)]
        struct Writes(Vec<Vec<u8>>);
        impl std::io::Write for Writes {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.write_all(buf)?;
                Ok(buf.len())
            }
            fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
                self.0.push(buf.to_vec());
                Ok(())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut writes = Writes::default();
        json_marshal_indent_write(&mut writes, &Compact(SHOW_CONFIG_31), "", "    ").unwrap();
        let sizes: Vec<usize> = writes.0.iter().map(Vec::len).collect();
        assert_eq!(sizes, [83, 149, 178, 299, 294, 46]);
        let text = json_append_multiline(SHOW_CONFIG_31, "", "    ").unwrap();
        assert_eq!(writes.0.concat(), text.into_bytes());
    }
}

#[cfg(test)]
mod decode_string_tests {
    use super::*;

    fn token_ref_of(token: &JsonToken) -> JsonTokenRef<'static> {
        match token {
            JsonToken::Null => JsonTokenRef::Null,
            JsonToken::False => JsonTokenRef::False,
            JsonToken::True => JsonTokenRef::True,
            JsonToken::BeginObject => JsonTokenRef::BeginObject,
            JsonToken::EndObject => JsonTokenRef::EndObject,
            JsonToken::BeginArray => JsonTokenRef::BeginArray,
            JsonToken::EndArray => JsonTokenRef::EndArray,
            JsonToken::String(_) | JsonToken::Number(_) => unreachable!("compared by text"),
        }
    }

    // `scan_string` borrows only where `consume_string_chars` gives the
    // same text, and gives its end offset and errors everywhere.
    #[test]
    fn scan_string_matches_consume_string_chars() {
        let bodies: [&[u8]; 16] = [
            b"",
            b"plain ascii",
            b"tab\there",
            b"quote \\\" and \\\\ and \\/",
            b"\\u0041\\u00e9\\ud83d\\ude00",
            b"\\ud800 lone",
            "caf\u{e9} \u{1f600} \u{fdd0}".as_bytes(),
            b"bad \xff byte",
            b"overlong \xc0\x80",
            b"surrogate \xed\xa0\x80",
            b"cut \xe2\x82",
            b"ctl \x01",
            b"del \x7f",
            b"\\x bad escape",
            b"\\u12 short",
            b"unterminated",
        ];
        for allow_invalid_utf8 in [false, true] {
            let options = JsonOptions {
                allow_invalid_utf8,
                ..JsonOptions::default()
            };
            for body in bodies {
                for terminated in [true, false] {
                    let mut input = vec![b'"'];
                    input.extend_from_slice(body);
                    if terminated {
                        input.extend_from_slice(b"\" ,");
                    }
                    let dec = JsonDecoder::new(&input, options);
                    let fast = dec.scan_string(0).map(|(end, s)| (end, s.into_owned()));
                    let slow = dec.consume_string_chars(0);
                    assert_eq!(
                        fast, slow,
                        "{input:?} allow_invalid_utf8={allow_invalid_utf8}"
                    );
                }
            }
        }
    }

    // A JSON pointer holds Go's bytes of the names, where a real U+FDD0 is
    // one char, and `%q` quotes them (jsontext/errors.go:127 and :135). Texts
    // from Go N (encoding/json/v2 of Go 1.27.1, followups25 round b
    // tools-b/gojsonptr).
    #[test]
    fn pointer_names_are_quoted_from_go_bytes() {
        let read_all = |text: &[u8]| {
            let mut dec = JsonDecoder::new(
                text,
                JsonOptions {
                    allow_invalid_utf8: true,
                    ..JsonOptions::default()
                },
            );
            dec.skip_value().and_then(|()| dec.check_eof())
        };
        assert_eq!(
            read_all("{\"\\ufdd0\\ufdd0\":1,\"\u{fdd0}\u{fdd0}\":2}".as_bytes())
                .unwrap_err()
                .message,
            r#"jsontext: duplicate object member name "\ufdd0\ufdd0""#
        );
        assert_eq!(
            read_all("{\"\u{fdd0}\u{fdd0}\":{\"a\":1,\"a\":2}}".as_bytes())
                .unwrap_err()
                .message,
            r#"jsontext: duplicate object member name "a" within "/\ufdd0\ufdd0""#
        );
        assert_eq!(
            read_all(b"{\"x\xffy\":{\"a\":1,\"a\":2}}")
                .unwrap_err()
                .message,
            "jsontext: duplicate object member name \"a\" within \"/x\u{fffd}y\""
        );
    }

    // Names with and without escapes share one namespace per object, and a
    // later object starts with an empty one.
    #[test]
    fn duplicate_names() {
        let read_all = |text: &str| {
            let mut dec = JsonDecoder::new(text.as_bytes(), JsonOptions::default());
            dec.skip_value().and_then(|()| dec.check_eof())
        };
        // Go N (encoding/json/v2 of Go 1.27, tsgo pin 673a5f17d713) texts.
        assert_eq!(
            read_all(r#"{"a":1,"a":2}"#).unwrap_err().message,
            r#"jsontext: duplicate object member name "a""#
        );
        assert_eq!(
            read_all(r#"{"x":{"é":1,"é":2}}"#).unwrap_err().message,
            r#"jsontext: duplicate object member name "é" within "/x""#
        );
        assert!(read_all(r#"[{"a":1,"b":{"a":2}},{"a":3,"b":4}]"#).is_ok());
        let mut dec = JsonDecoder::new(br#"{"k":"v","n":1}"#, JsonOptions::default());
        let tokens: Vec<JsonToken> = std::iter::from_fn(|| dec.read_token().ok()).collect();
        let mut dec = JsonDecoder::new(
            "{\"k\\n\":\"\u{fdd0}\",\"n\":-1.5e3}".as_bytes(),
            JsonOptions::default(),
        );
        let refs: Vec<JsonTokenRef<'_>> =
            std::iter::from_fn(|| dec.read_token_ref().ok()).collect();
        let mut dec = JsonDecoder::new(
            "{\"k\\n\":\"\u{fdd0}\",\"n\":-1.5e3}".as_bytes(),
            JsonOptions::default(),
        );
        let owned: Vec<JsonToken> = std::iter::from_fn(|| dec.read_token().ok()).collect();
        assert_eq!(refs.len(), owned.len());
        for (token_ref, token) in refs.iter().zip(&owned) {
            match (token_ref, token) {
                (JsonTokenRef::String(a), JsonToken::String(b)) => assert_eq!(a, b),
                (JsonTokenRef::Number(a), JsonToken::Number(b)) => assert_eq!(a, b),
                (a, b) => assert_eq!(a.clone(), token_ref_of(b)),
            }
        }
        assert!(matches!(&refs[2], JsonTokenRef::String(Cow::Owned(_))));
        assert!(matches!(&refs[3], JsonTokenRef::String(Cow::Borrowed("n"))));
        assert_eq!(
            tokens,
            [
                JsonToken::BeginObject,
                JsonToken::String("k".into()),
                JsonToken::String("v".into()),
                JsonToken::String("n".into()),
                JsonToken::Number("1".into()),
                JsonToken::EndObject,
            ]
        );
    }
}
