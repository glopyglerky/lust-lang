// expect: L0110
// rustc: literal out of range for `u8` (a const argument in a type).
struct C<const N: u8>;
fn f(_c: C<300>) {}
fn main() {
    println!("ok");
}
