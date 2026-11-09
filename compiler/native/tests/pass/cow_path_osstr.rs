// Cow<Path> and Cow<OsStr>: unsized slice newtypes whose Owned types
// are PathBuf and OsString (written `ToOwned` impls in std). Views,
// to_mut, into_owned with exact capacity, clone, ==, and the generic
// From<&B> and From<Owned> impls.
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

fn os_path(path: &str, owned: bool) -> Cow<'_, Path> {
    if owned {
        Cow::Owned(PathBuf::from(path.replace("\\", "/")))
    } else {
        Cow::Borrowed(Path::new(path))
    }
}

fn main() {
    let mut c = os_path("a/b.txt", false);
    println!("{:?} {:?} {}", c.file_name(), c.extension(), matches!(c, Cow::Borrowed(_)));
    c.to_mut().push("c");
    println!("{:?} {} {}", c, c.display(), matches!(c, Cow::Owned(_)));
    let o: PathBuf = c.into_owned();
    println!("{:?} {}", o, o.capacity());
    let d: Cow<Path> = os_path("a\\b", true);
    println!("{:?} {} {}", d, d.clone() == d, d == os_path("a/b", false));
    let f: Cow<Path> = Path::new("q").into();
    let g: Cow<Path> = PathBuf::from("r").into();
    let h: Cow<Path> = Cow::from(Path::new("s"));
    println!("{:?} {:?} {:?}", f, g, h);
    let p = Path::new("p/q").to_owned();
    println!("{:?}", p);

    let s: Cow<OsStr> = Cow::Borrowed(OsStr::new("ab"));
    println!("{:?} {} {:?}", s, s.len(), s.to_str());
    let so: OsString = s.into_owned();
    println!("{:?} {}", so, so.capacity());
    let t: Cow<OsStr> = OsString::from("cd").into();
    let u: Cow<OsStr> = OsStr::new("ef").into();
    println!("{:?} {:?} {}", t, u, t.clone().into_owned() == OsString::from("cd"));
    let mut owned_back = OsString::from("longer value");
    OsStr::new("xy").clone_into(&mut owned_back);
    println!("{:?} {}", owned_back, owned_back.capacity());
}
