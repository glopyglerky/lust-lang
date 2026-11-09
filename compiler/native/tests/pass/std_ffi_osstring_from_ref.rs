// `impl<T: ?Sized + AsRef<OsStr>> From<&T> for OsString`: every AsRef<OsStr>
// type by reference, through From, Into and an `impl Into<OsString>` param.
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

fn conv(s: impl Into<OsString>) -> OsString {
    s.into()
}

struct W(String);
impl AsRef<OsStr> for W {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

fn main() {
    let s = String::from("s");
    let o = OsString::from("o");
    println!("{:?} {:?} {:?}", OsString::from("a"), OsString::from(&s), OsString::from(&o));
    println!("{:?} {:?}", conv(OsStr::new("b")), conv(Path::new("c/d")));
    let pb = PathBuf::from("e");
    let x: OsString = (&pb).into();
    println!("{:?} {:?}", x, OsString::from(&W("w".to_string())));
    let p = Path::new("f/g");
    let parts: Vec<OsString> = p.components().map(|c| OsString::from(&c)).collect();
    println!("{:?} {:?}", parts, OsString::from(&p.iter()));
}
