// PathBuf and OsString From impls (W-SHIM): &str, String, OsString, &Path.
use std::ffi::OsString;
use std::path::{Path, PathBuf};
fn main() {
    let a = PathBuf::from("a/b");
    let b = PathBuf::from(String::from("/x//y/"));
    let c = PathBuf::from(OsString::from("o"));
    let d = PathBuf::from(Path::new("p/q.rs"));
    let e = OsString::from(String::from("s"));
    println!("{:?} {:?} {:?} {:?} {:?} {:?}", a, b, c, d, e, OsString::from(d.clone()));
}
