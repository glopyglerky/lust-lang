// expect: L0102
// A block type alias may not use the enclosing fn's generic parameters
// (rustc E0401).
fn g<T>() {
    type A = Vec<T>;
    let _v: A = Vec::new();
}
fn main() { g::<u8>(); }
