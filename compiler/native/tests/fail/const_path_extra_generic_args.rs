// expect: L0110
// rustc E0107: `S` takes no generic arguments, so `S::<3>::D` is an error.
struct S;
impl S {
    const D: usize = 3;
}
fn main() {
    let a = [0u8; S::<3>::D];
    println!("{}", a.len());
}
