// expect: L0118
// rustc: literal out of range for u8; the cast target decides a
// block-tail literal's type as it decides a bare literal's.
fn main() {
    let a = { 300 } as u8;
    println!("{}", a);
}
