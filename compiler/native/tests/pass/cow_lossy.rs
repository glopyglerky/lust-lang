// The lossy UTF-8 decoders that return Cow<str>: borrowed when the
// bytes are valid, owned with U+FFFD replacements (and std's buffer
// growth) when they are not.
use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::Path;

fn check(raw: &[u8]) {
    let c = String::from_utf8_lossy(raw);
    let owned = matches!(c, Cow::Owned(_));
    let s = c.into_owned();
    println!("{:?} {} {} {}", s, owned, s.len(), s.capacity());
}

fn main() {
    let ok: &[u8] = b"hi";
    let mixed: &[u8] = &[b'a', 0xff, b'b'];
    check(ok);
    check(mixed);
    check(b"\xff\xff\xff");
    check(b"hello world this is valid except \xff end");
    check(b"\xc3\x28");
    check(&[0xffu8; 40]);
    check(b"");
    check("héllo".as_bytes());
    check(b"\xe2\x82");
    check(b"\xf0\x9f\x92");
    println!("{}", OsStr::new("a").to_string_lossy());
    println!("{} {}", Path::new("x/y").to_string_lossy(), matches!(Path::new("x").to_string_lossy(), Cow::Borrowed(_)));
}
