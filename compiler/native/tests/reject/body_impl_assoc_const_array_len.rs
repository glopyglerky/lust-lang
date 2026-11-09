// expect: L0110
// Valid Rust. A block type's assoc const as an array length: the scalar const scan reads module impls only.
fn main() {
    struct S;
    impl S { const N: usize = 3; fn arr() -> [u8; Self::N] { [1; Self::N] } }
    let a = [0u8; S::N];
    println!("{:?} {:?}", a, S::arr());
}
