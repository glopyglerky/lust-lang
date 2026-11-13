// std::ffi (W-SHIM): OsStr and OsString as Unix byte strings: Debug,
// to_str, len, ASCII case, push, into_string, and comparisons.
use std::ffi::{OsStr, OsString};

fn main() {
    for s in ["", "abc", "héllo", "tab\there", "quo\"te", "MiXeD"] {
        let o = OsStr::new(s);
        println!("{:?} {} {} {:?} {:?} {:?}", o, o.len(), o.is_empty(), o.to_str(), o.to_ascii_lowercase(), o.to_ascii_uppercase());
        println!("{:?} {}", o.as_encoded_bytes(), o.eq_ignore_ascii_case("mixed"));
    }
    let mut s: OsString = "ab".into();
    s.push("cd");
    s.push(OsStr::new("é"));
    let t: OsString = String::from("x").into();
    println!("{:?} {} {:?} {}", s, s.len(), t, s.to_str() == Some("abcdé"));
    println!("{} {} {}", s == *"abcdé", *OsStr::new("a") < *OsStr::new("b"), t.as_os_str() == OsStr::new("x"));
    let copy = s.clone();
    s.clear();
    println!("{:?} {:?} {}", s, copy, s.is_empty());
    match copy.into_string() {
        Ok(text) => println!("text {}", text),
        Err(e) => println!("bad {:?}", e),
    }
    let (a, b): (OsString, OsString) = ("b".into(), "a".into());
    println!("{} {:?} {}", a > b, a.cmp(&b), a == b);
}
