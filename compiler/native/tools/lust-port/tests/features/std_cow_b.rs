// feature: std.cow
// form: Cow in structs and hashed keys, Cow<[u8]> and Cow<Path>, to_mut, From, lossy
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
struct Diag {
    code: i32,
}
impl Diag {
    fn code(&self) -> i32 {
        self.code
    }
}

struct Frame<'a> {
    names: HashSet<Cow<'a, str>>,
    last: Cow<'a, Diag>,
}

fn bytes(b: &[u8]) -> Cow<'_, [u8]> {
    if b.len() > 2 { Cow::Owned(b[..2].to_vec()) } else { Cow::Borrowed(b) }
}

fn os_path(path: &str) -> Cow<'_, Path> {
    match bytes(path.as_bytes()) {
        Cow::Borrowed(_) => Cow::Borrowed(Path::new(path)),
        Cow::Owned(v) => Cow::Owned(PathBuf::from(String::from_utf8_lossy(&v).into_owned())),
    }
}

fn main() {
    let mut f = Frame { names: HashSet::new(), last: Cow::Owned(Diag { code: 0 }) };
    f.names.insert(Cow::Borrowed("k"));
    f.names.insert(Cow::Owned("j".to_string()));
    let d = Diag { code: 7 };
    f.last = Cow::Borrowed(&d);
    println!("{} {} {}", f.names.contains("k"), f.names.contains("j"), f.last.code());
    f.last.to_mut().code = 9;
    println!("{} {} {}", f.last.code(), d.code, matches!(f.last, Cow::Owned(_)));
    let raw = [1u8, 2, 3];
    let v = bytes(&raw);
    println!("{:?} {} {}", v, v[0], v.clone().into_owned().len());
    let p = os_path("a/b.txt");
    println!("{:?} {}", p, p.display());
    let from: Cow<str> = "xy".into();
    let st = String::from("zw");
    let from_ref: Cow<str> = (&st).into();
    let back: String = from.into();
    println!("{} {}", back, from_ref);
    let raw2: &[u8] = &[b'a', 0xff, b'b'];
    println!("{}", String::from_utf8_lossy(raw2).into_owned());
}
