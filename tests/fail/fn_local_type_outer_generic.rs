// expect: L0110
// A block item may not use the enclosing fn's generic parameters
// (rustc E0401).
fn g<T>(t: T) {
    enum E { A(T) }
    let _ = E::A(t);
}
fn main() { g(1u8); }
