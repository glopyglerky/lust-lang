//! Go `net/url` subset (go1.27.1 `src/net/url/url.go`,
//! `src/net/url/encoding_table.go`; go1.27.1 changed only `URL.String` and
//! added the `Clone` methods, which are not ported): `Parse` for
//! `lsproto.DocumentUri.FileName`, `PathEscape`, `QueryEscape`,
//! `PathUnescape` and what they call. Other `net/url` functions are not
//! ported.
//!
//! PORT: Go strings hold any bytes. A port string is the port form of a Go
//! string (see `scanner_util::GO_STRING_MARKER`). `unescape` and `escape`
//! read its Go bytes (`go_string_bytes`). A string that `unescape` builds
//! from bytes, for example "caf\xff" from "caf%FF", is in the value form
//! (`go_value_from_bytes`), like a file name from the OS. `escape` escapes
//! every byte that is not ASCII, so its result is ASCII.

use crate::prelude::*;

use crate::gostd::errors::{self, GoError};
use crate::gostd::netip;
use crate::gostd::strconv;
use std::fmt;

// Go: net/url/url.go:30 urlstrictcolons
// PORT: GODEBUG is not read. The go1.27.1 default (strict colons for http and
// https, Old "0" not set) applies.
fn urlstrictcolons_value() -> &'static str {
    ""
}

// Go: net/url/url.go:33 Error
/// Error reports an error and the operation and URL that caused it.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub op: String,
    pub url: String,
    pub err: GoError,
}

impl Error {
    // Go: net/url/url.go:39 Unwrap
    pub fn unwrap(&self) -> GoError {
        self.err.clone()
    }

    // Go: net/url/url.go:40 Error

}


const UPPERHEX: &[u8; 16] = b"0123456789ABCDEF";

// Go: net/url/url.go:58 ishex
fn ishex(c: u8) -> bool {
    TABLE[c as usize] & HEX_CHAR != 0
}

// Go: net/url/url.go:63 unhex
/// Precondition: ishex(c) is true.
fn unhex(c: u8) -> u8 {
    9 * (c >> 6) + (c & 15)
}

// Go: net/url/url.go:67 EscapeError
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EscapeError(pub String);


// Go: net/url/url.go:73 InvalidHostError
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidHostError(pub String);


// Go: net/url/url.go:80 shouldEscape
/// See the reference implementation in gen_encoding_table.go.
fn should_escape(c: u8, mode: Encoding) -> bool {
    TABLE[c as usize] & mode == 0
}

// Go: net/url/url.go:89 QueryUnescape

// Go: net/url/url.go:100 PathUnescape

// Go: net/url/url.go:106 unescape


// Go: net/url/url.go:186 QueryEscape

// Go: net/url/url.go:192 PathEscape

// Go: net/url/url.go:196 escape

// Go: net/url/url.go:276 URL
/// A URL represents a parsed URL (technically, a URI reference).
///
/// The general form represented is:
///
/// `[scheme:][//[userinfo@]host][/]path[?query][#fragment]`
///
/// URLs that do not start with a slash after the scheme are interpreted as:
///
/// `scheme:opaque[?query][#fragment]`
///
/// Note that the Path field is stored in decoded form: /%47%6f%2f becomes /Go/.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct URL {
    pub scheme: String,
    pub opaque: String,         // encoded opaque data
    pub user: Option<Userinfo>, // username and password information
    pub host: String,           // "host" or "host:port" (see Hostname and Port methods)
    pub path: String,           // path (relative paths may omit leading slash)
    pub fragment: String,       // fragment for references (without '#')

    /// RawQuery contains the encoded query values, without the initial '?'.
    /// Use URL.Query to decode the query.
    pub raw_query: String,

    /// RawPath is an optional field containing an encoded path hint.
    /// See the EscapedPath method for more details.
    ///
    /// In general, code should call EscapedPath instead of reading RawPath.
    pub raw_path: String,

    /// RawFragment is an optional field containing an encoded fragment hint.
    /// See the EscapedFragment method for more details.
    ///
    /// In general, code should call EscapedFragment instead of reading RawFragment.
    pub raw_fragment: String,

    /// ForceQuery indicates whether the original URL contained a query ('?') character.
    /// When set, the String method will include a trailing '?', even when RawQuery is empty.
    pub force_query: bool,

    /// OmitHost indicates the URL has an empty host (authority).
    /// When set, the String method will not include the host when it is empty.
    pub omit_host: bool,
}

// Go: net/url/url.go:311 User
/// User returns a [Userinfo] containing the provided username
/// and no password set.
pub fn user(username: &str) -> Userinfo {
    Userinfo {
        username: username.to_string(),
        password: String::new(),
        password_set: false,
    }
}

// Go: net/url/url.go:323 UserPassword
/// UserPassword returns a [Userinfo] containing the provided username
/// and password.
pub fn user_password(username: &str, password: &str) -> Userinfo {
    Userinfo {
        username: username.to_string(),
        password: password.to_string(),
        password_set: true,
    }
}

// Go: net/url/url.go:331 Userinfo
/// The Userinfo type is an immutable encapsulation of username and
/// password details for a [URL]. An existing Userinfo value is guaranteed
/// to have a username set (potentially empty, as allowed by RFC 2396),
/// and optionally a password.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Userinfo {
    username: String,
    password: String,
    password_set: bool,
}

impl Userinfo {
    // Go: net/url/url.go:338 Username
    /// Username returns the username.
    pub fn username(&self) -> String {
        self.username.clone()
    }

    // Go: net/url/url.go:346 Password
    /// Password returns the password in case it is set, and whether it is set.
    pub fn password(&self) -> (String, bool) {
        (self.password.clone(), self.password_set)
    }

    // Go: net/url/url.go:355 String
}

// Go: net/url/url.go:369 getScheme

// Go: net/url/url.go:399 Parse

// Go: net/url/url.go:420 ParseRequestURI

// Go: net/url/url.go:432 parse

// Go: net/url/url.go:512 parseAuthority

// Go: net/url/url.go:549 parseHost

impl URL {
    // Go: net/url/url.go:660 setPath

    // Go: net/url/url.go:687 EscapedPath

    // Go: net/url/url.go:727 setFragment

    // Go: net/url/url.go:750 EscapedFragment
}

// Go: net/url/url.go:703 validEncoded
/// validEncoded reports whether s is a valid encoded path or fragment,
/// according to mode.
/// It must not contain any bytes that require escaping during encoding.
fn valid_encoded(s: &str, mode: Encoding) -> bool {
    for &c in s.as_bytes() {
        // RFC 3986, Appendix A.
        // pchar = unreserved / pct-encoded / sub-delims / ":" / "@".
        // shouldEscape is not quite compliant with the RFC,
        // so we check the sub-delims ourselves and let
        // shouldEscape handle the others.
        match c {
            b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'=' | b':'
            | b'@' => {
                // ok
            }
            b'[' | b']' => {
                // ok - not specified in RFC 3986 but left alone by modern browsers
            }
            b'%' => {
                // ok - percent encoded, will decode
            }
            _ => {
                if should_escape(c, mode) {
                    return false;
                }
            }
        }
    }
    true
}

// Go: net/url/url.go:762 validOptionalPort
/// validOptionalPort reports whether port is either an empty string
/// or matches /^:\d*$/
fn valid_optional_port(port: &str) -> bool {
    if port.is_empty() {
        return true;
    }
    if !port.starts_with(':') {
        return false;
    }
    for b in port[1..].chars() {
        if !b.is_ascii_digit() {
            return false;
        }
    }
    true
}

// Go: net/url/url.go:1283 validUserinfo
/// validUserinfo reports whether s is a valid userinfo string per RFC 3986
/// Section 3.2.1.
/// It doesn't validate pct-encoded. The caller does that via func unescape.
fn valid_userinfo(s: &str) -> bool {
    for r in s.chars() {
        if r.is_ascii_uppercase() {
            continue;
        }
        if r.is_ascii_lowercase() {
            continue;
        }
        if r.is_ascii_digit() {
            continue;
        }
        match r {
            '-' | '.' | '_' | ':' | '~' | '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ','
            | ';' | '=' | '%' => continue,
            '@' => {
                // `RFC 3986 section 3.2.1` does not allow '@' in userinfo.
                // It is a delimiter between userinfo and host.
                // However, URLs are diverse, and in some cases,
                // the userinfo may contain an '@' character,
                // for example, in "http://username:p@ssword@google.com",
                // the string "username:p@ssword" should be treated as valid userinfo.
                // Ref:
                //   https://go.dev/issue/3439
                //   https://go.dev/issue/22655
                continue;
            }
            _ => return false,
        }
    }
    true
}

// Go: net/url/url.go:1317 stringContainsCTLByte
/// stringContainsCTLByte reports whether s contains any ASCII control character.
fn string_contains_ctl_byte(s: &str) -> bool {
    for &b in s.as_bytes() {
        if b < b' ' || b == 0x7f {
            return true;
        }
    }
    false
}

// Go: net/url/encoding_table.go:9 encoding
type Encoding = u8;

const ENCODE_PATH: Encoding = 1 << 0;
const ENCODE_PATH_SEGMENT: Encoding = 1 << 1;
const ENCODE_HOST: Encoding = 1 << 2;
const ENCODE_ZONE: Encoding = 1 << 3;
const ENCODE_USER_PASSWORD: Encoding = 1 << 4;
const ENCODE_QUERY_COMPONENT: Encoding = 1 << 5;
const ENCODE_FRAGMENT: Encoding = 1 << 6;
// hexChar is actually NOT an encoding mode, but there are only seven
// encoding modes. We might as well abuse the otherwise unused most
// significant bit in uint8 to indicate whether a character is
// hexadecimal.
const HEX_CHAR: Encoding = 1 << 7;

// Go: net/url/encoding_table.go:27 table
// Generated from the Go table: bit 0 encodePath, 1 encodePathSegment,
// 2 encodeHost, 3 encodeZone, 4 encodeUserPassword, 5 encodeQueryComponent,
// 6 encodeFragment, 7 hexChar. A set bit means the byte is left as is in
// that mode.
static TABLE: [Encoding; 256] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x4c, 0x0c, 0x00, 0x5f, 0x00, 0x5f, 0x0c, 0x4c, 0x4c, 0x4c, 0x5f, 0x5d, 0x7f, 0x7f, 0x41,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x4f, 0x5d, 0x0c, 0x5f, 0x0c, 0x40,
    0x43, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f,
    0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x0c, 0x00, 0x0c, 0x00, 0x7f,
    0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f,
    0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x00, 0x00, 0x00, 0x7f, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
