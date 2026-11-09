// An empty struct keeps its written shape across edits on one warm
// daemon: a unit struct is a value, `struct Z {}` only takes braces
// (rustc E0423 for `Z` as a value), `struct Z();` is a constructor fn.
#[derive(Debug)]
struct Z {}
fn main() {
    let z = Z;
    let w = Z {};
    println!("{:?} {:?}", z, w);
}
