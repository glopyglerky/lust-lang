// expect: L0113
// PathBuf has no PartialEq<&str> (only PartialEq<str>, which needs a
// str place on the right: `pb == *"a/b"`).
use std::path::PathBuf;

fn main() {
    let pb = PathBuf::from("a/b");
    println!("{}", pb == "a/b");
}
