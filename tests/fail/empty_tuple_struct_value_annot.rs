// expect: L0110
// rustc E0308: for `struct Z();`, `Z` is the constructor fn, not a value of `Z`.
struct Z();
fn main() {
    let _z: Z = Z;
}
