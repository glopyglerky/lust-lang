// `==` and friends through `PartialEq<Rhs>` / `PartialOrd<Rhs>` impls
// whose Rhs differs from Self (W-SHIM phase 2 slice 4): std's
// PathBuf/Path and OsString impls and user impls, with the reference
// on either side.
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Meters(u32);

#[derive(Debug)]
struct Feet(u32);

impl PartialEq<u32> for Meters {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<&str> for Meters {
    fn eq(&self, other: &&str) -> bool {
        match *other {
            "zero" => self.0 == 0,
            _ => false,
        }
    }
}

impl PartialEq<Feet> for Meters {
    fn eq(&self, other: &Feet) -> bool {
        self.0 * 3 == other.0
    }
}

impl PartialOrd<u32> for Meters {
    fn partial_cmp(&self, other: &u32) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<Meters> for u32 {
    fn eq(&self, other: &Meters) -> bool {
        *self == other.0
    }
}

fn eq_to<T: PartialEq<u32>>(x: &T, v: u32) -> bool {
    x == &v
}

fn main() {
    let pb = PathBuf::from("a/b");
    let p: &Path = Path::new("a/b");
    let q: &Path = Path::new("a/c");
    println!("{} {} {} {}", pb == p, pb == q, p == pb, q == pb);
    println!("{} {} {}", pb != q, &pb == p, pb == *p);
    println!("{} {}", pb == *"a/b", *p == *"a/b");
    let owned = String::from("a/b");
    println!("{} {}", pb == owned, owned == pb);
    let os = OsString::from("a/b");
    println!("{} {}", pb == os, os == pb);
    println!("{} {}", p == "a/b", "a/c" == p);
    let m = Meters(7);
    println!("{} {} {} {}", m == 7u32, m == 8u32, m != 8u32, 7u32 == m);
    println!("{} {} {}", m == "zero", Meters(0) == "zero", Meters(0) != "one");
    println!("{} {}", m == Feet(21), m == Feet(20));
    println!("{} {} {} {}", m < 8u32, m > 8u32, m <= 7u32, m >= 9u32);
    println!("{} {} {}", &m == &7u32, m == 7u32, eq_to(&m, 7));
    let s = String::from("hi");
    let t: &str = "hi";
    println!("{} {}", s == t, t == s);
    println!("{}", OsStr::new("x") == OsString::from("x").as_os_str());
}
