// The tuple form: `Z()` constructs, and `Z` is the constructor fn.
#[derive(Debug)]
struct Z();
fn main() {
    let z = Z();
    let f: fn() -> Z = Z;
    let w = Z {};
    println!("{:?} {:?} {:?}", z, f(), w);
}
