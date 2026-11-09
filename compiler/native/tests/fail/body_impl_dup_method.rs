// expect: L0106
// rustc E0592: a body impl and a module impl define the same method.
struct S;
impl S { fn a(&self) -> u8 { 1 } }
fn main() {
    impl S { fn a(&self) -> u8 { 2 } }
    println!("{}", S.a());
}
