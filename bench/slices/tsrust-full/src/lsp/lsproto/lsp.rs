//! Port of internal/lsp/lsproto/lsp.go.
//!
//! PORT: `DocumentUri`, `URI` and `Method` are Go string types; their JSON
//! impls are the v2 string arshaler. `IndexMap<DocumentUri, V>` (Go
//! `map[DocumentUri]V`) gets the v2 map arshaler here, through the
//! goport_util trait `JsonMapKey`.
//!
//! The `err*` helpers make the plain errors that generated `UnmarshalJSONFrom`
//! methods return. Each one records where the method left the decoder, so
//! that `unmarshal_root` can give the error its Go v2 JSON pointer.

use crate::lsp::lsproto::prelude::*;

use crate::frontend::bundled::is_bundled;
use std::marker::PhantomData;

// Go: lsp.go:17 DocumentUri
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentUri(pub String); // !!!

impl DocumentUri {
    // Go: lsp.go:19 FileName

    // Go: lsp.go:52 Path
}

// Go: lsp.go:57 fixWindowsURIPath

// Go: lsp.go:66 HasTextDocumentURI
pub trait HasTextDocumentURI {
    fn text_document_uri(&self) -> DocumentUri;
}

// Go: lsp.go:70 HasTextDocumentPosition
pub trait HasTextDocumentPosition: HasTextDocumentURI {
    fn text_document_position(&self) -> Position;
}

// Go: lsp.go:75 HasLocations
pub trait HasLocations {
    fn get_locations(&self) -> Option<&Vec<Location>>;
}

// Go: lsp.go:79 HasLocation
pub trait HasLocation {
    fn get_location(&self) -> Location;
}

// Go: lsp.go:83 URI
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct URI(pub String); // !!!

// Go: lsp.go:85 Method

// Go `%s` / `%v` of a string type prints the string.
impl std::fmt::Display for DocumentUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for URI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}


// Go v2 string arshaler for the three string types.


impl IsZero for DocumentUri {
    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
}



impl IsZero for URI {
    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
}




// PORT: the JSON impls of `IndexMap<DocumentUri, V>` (Go `map[DocumentUri]V`).
// The orphan rule keeps `MarshalerTo` and `UnmarshalerFrom` for `IndexMap`
// in goport_util, which calls these through `JsonMapKey`.

// Go: boolToInt (removed in tsgo#4471)
// PORT: Go tsgo#4471 replaced the generated union and `Registration`
// marshal checks with the reflective `marshalUnion` and `countNonNil`
// (structcodec.go:129, :155). Rust has no reflection, so the generated code
// keeps its per-field checks and this helper; the behavior is the same.
pub fn bool_to_int(b: bool) -> i32 {
    if b {
        return 1;
    }
    0
}

// Go `%v` of a `jsontext.Kind` (jsontext/token.go:643 Kind.String).

// Go: lsp.go:87 errNotObject
// Generated methods return it before they read the value.

// Go: lsp.go:91 errNull
// Generated methods return it after the member name.

// Go: lsp.go:95 errMissing
// Generated methods return it after the closing `}`.

// Go: lsp.go:99 errInvalidKind
// Generated methods return it after a peek, before they read the value.

// Go: lsp.go:103 errInvalidValue
// Generated methods return it after they read the value.

// Go: lsp.go:107 errLiteralMismatch
// Generated methods return it after they read the value.

// Go: lsp.go:111 assertOnlyOne
pub fn assert_only_one(message: &str, count: i32) {
    if count != 1 {
        crate::core::go_panic(message.to_string());
    }
}

// Go: lsp.go:117 assertAtMostOne
pub fn assert_at_most_one(message: &str, count: i32) {
    if count > 1 {
        crate::core::go_panic(message.to_string());
    }
}

// Go: lsp.go:124 jsonKeyCheck
// jsonKeyCheck compares a raw JSON key token (including quotes) against a Go string.
pub fn json_key_check(name: &[u8], key: &str) -> bool {
    name.len() == key.len() + 2 && name[0] == b'"' && &name[1..name.len() - 1] == key.as_bytes()
}

// Go: lsp.go:131 jsonObjectRawField
// jsonObjectRawField scans the top-level keys of a JSON object looking for the
// given field name, and returns its raw JSON value (e.g. `"full"` with quotes).
// Returns nil if the field is not found.

// Go: structcodec.go:172 discriminatedStructDecoder
/// The state of [`scan_discriminated_struct`]: the discriminator value and
/// the object members that come before it.
///
/// PORT: Go keeps each earlier member as its name and raw value
/// (`deferredStructField`); here each is the raw name (with quotes) and the
/// raw value.
#[derive(Debug, Default)]
pub struct DiscriminatedStructDecoder {
    type_name: &'static str,
    discriminator: &'static str,
    pub discriminator_value: Vec<u8>,
    has_discriminator: bool,
    deferred: Vec<(Vec<u8>, Vec<u8>)>,
    closed: bool,
}

// Go: structcodec.go:184 scanDiscriminatedStruct
// scanDiscriminatedStruct advances through an object until it finds the
// discriminator. Only fields preceding the discriminator are retained.

impl DiscriminatedStructDecoder {
    // Go: structcodec.go:226 (discriminatedStructDecoder).invalidDiscriminator
    // PORT: with no discriminator the scan read the closing `}`; else it
    // stopped just after the discriminator value.
}

// Go: structcodec.go:235 unmarshalDiscriminatedArm
// unmarshalDiscriminatedArm decodes the retained and remaining object fields
// into a concrete union arm, assigning it only after the full object succeeds.
// PORT: Go feeds the discriminator, the retained fields and then the rest of
// the object to one reflective struct decoder. Rust has no reflection: the
// members are joined, in that order, into one object that the arm's
// generated decoder reads. The caller assigns the arm.

// Go: lsp.go:161 jsonObjectHasKey
// jsonObjectHasKey scans the top-level keys of a JSON object looking for any of the
// given keys. Returns the index of the first key found, or -1 if none match.
// Bails early on first match without decoding any values.

// Inspired by https://www.youtube.com/watch?v=dab3I-HcTVk

// Go: lsp.go:188 RequestInfo
// PORT: Go `_ [0]Params` / `_ [0]Resp` are `PhantomData`. Go builds the
// value with a struct literal; `new` is the const constructor the
// generated `*_INFO` consts use.






// Go: lsp.go:215 NotificationInfo





// Go: lsp.go:266 Null
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Null;

// Go: lsp.go:271 (Null) UnmarshalJSONFrom

// Go: lsp.go:282 (Null) MarshalJSONTo
impl MarshalerTo for Null {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        enc.push_str("null");
        Ok(())
    }
}

// Go reflect: an empty struct is always zero.
impl IsZero for Null {
    fn is_zero(&self) -> bool {
        true
    }
}

// Go: lsp.go:235 UnmarshalParams
// UnmarshalParams decodes the params of an inbound request or notification
// message into the requested type. Inbound messages store their params as a
// raw [json.Value] (see [Message.UnmarshalJSON]); decoding is deferred to the
// point of dispatch so that param types for methods the server never handles
// are not forced into the binary.
//
// A [NoParams] method must be given no params; every other method must be given
// params as an object or array. A violation returns [ErrorCodeInvalidParams].
// PORT: Go `T` is a pointer type (or `NoParams`); a successful decode never
// gives a nil pointer, so the port returns the value. `%T` in the error prints
// the Rust debug value.

// Go: lsp.go:283 NoParams
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NoParams;

// Go: lsp.go:288 (NoParams) IsZero
impl IsZero for NoParams {
    fn is_zero(&self) -> bool {
        true
    }
}

// Go v2 default struct arshaler for `struct{}`: `{}`.
impl MarshalerTo for NoParams {
    fn marshal_json_to(&self, enc: &mut String) -> Result<(), JsonError> {
        write_object_start(enc);
        write_object_end(enc);
        Ok(())
    }
}

// Go v2 default struct arshaler for `struct{}`: null or an object (members
// skipped); any other kind is an error.

// Go: lsp.go:287 clientCapabilitiesKey

// Go: lsp.go:289 WithClientCapabilities
// PORT: Go stores the `*ResolvedClientCapabilities` pointer; the context
// holds an `Arc` to it.

// Go: lsp.go:293 GetClientCapabilities

// Go: lsp.go:302 PreferredMarkupKind
// PreferredMarkupKind returns the first (most preferred) markup kind from the given formats,
// or MarkupKindPlainText if the slice is empty.


// Go: lsp.go:316

#[cfg(test)]
mod unmarshal_error_tests {
    use super::*;

    /// The error of `unmarshal_params::<T>` for a request whose raw params
    /// are `data`.
    fn params_err_text<T: UnmarshalerFrom + Default + std::fmt::Debug + 'static>(
        data: &str,
    ) -> String {
        let req = RequestMessage {
            params: Some(Box::new(JsonValue(data.as_bytes().to_vec()))),
            ..RequestMessage::default()
        };
        unmarshal_params::<T>(&req)
            .expect_err("want an error")
            .error()
    }

    // Expected texts come from Go at the pin (tsgo-oracle-52168999f3dc; the
    // probe is in continuation-r97-goport/upstream/bumpA2/evidence/
    // lsp-invalid-params-N.txt). Since tsgo#4471 Go decodes params at
    // dispatch; the text is the old inner JSON error after "InvalidParams: ".
    // Go writes "cannot" or "unable to".
    #[test]
    fn params_errors_match_go() {
        assert_eq!(
            params_err_text::<HoverParams>(r#"{"textDocument":{"uri":"file:///a.ts"}}"#),
            "InvalidParams: json: cannot unmarshal into Go lsproto.HoverParams after offset 38: missing required properties: position"
        );
        assert_eq!(
            params_err_text::<SignatureHelpParams>(
                r#"{"textDocument":{"uri":"file:///a.ts"},"position":{"line":0,"character":0},"context":5}"#
            ),
            r#"InvalidParams: json: cannot unmarshal into Go lsproto.SignatureHelpContext within "/context": expected object start, but encountered number"#
        );
        assert_eq!(
            params_err_text::<HoverParams>(
                r#"{"textDocument":{"uri":5},"position":{"line":0,"character":0}}"#
            ),
            r#"InvalidParams: json: cannot unmarshal JSON number into Go lsproto.DocumentUri within "/textDocument/uri""#
        );
        assert_eq!(
            params_err_text::<HoverParams>(
                r#"{"textDocument":{"uri":"file:///a.ts"},"position":{"line":"x","character":0}}"#
            ),
            r#"InvalidParams: json: cannot unmarshal JSON string into Go uint32 within "/position/line""#
        );
        assert_eq!(
            params_err_text::<HoverParams>("null"),
            "InvalidParams: params must be an object or array"
        );
        assert_eq!(
            params_err_text::<HoverParams>("5"),
            "InvalidParams: params must be an object or array"
        );
        assert_eq!(
            params_err_text::<NoParams>("{}"),
            "InvalidParams: expected no params, got {}"
        );
    }

    // S6-002: a signatureHelp retrigger whose `activeSignatureHelp` is null.
    // Since tsgo#4471 the message reads without error and the handler's
    // `UnmarshalParams` fails (Go at the pin, same evidence file).
    #[test]
    fn null_active_signature_help_matches_go() {
        let data = br#"{"jsonrpc":"2.0","id":4,"method":"textDocument/signatureHelp","params":{"textDocument":{"uri":"file:///a.ts"},"position":{"line":0,"character":0},"context":{"triggerKind":3,"isRetrigger":true,"activeSignatureHelp":null}}}"#;
        let mut msg = Message::default();
        msg.unmarshal_json(data).expect("the message reads");
        let err =
            unmarshal_params::<SignatureHelpParams>(msg.as_request()).expect_err("want an error");
        assert_eq!(
            err.error(),
            r#"InvalidParams: json: cannot unmarshal into Go lsproto.SignatureHelpContext within "/context/activeSignatureHelp": null value is not allowed for field "activeSignatureHelp""#
        );
    }
}
