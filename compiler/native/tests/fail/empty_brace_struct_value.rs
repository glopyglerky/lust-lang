// expect: L0101
// rustc E0423: `struct Z {}` has no value; only `Z {}` builds one.
struct Z {}
fn main() {
    let _z = Z;
}
