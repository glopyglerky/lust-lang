// A trait declaration with no body may name unsized params and
// returns. No impl can give `f` a body, so none does.
trait T {
    fn f(x: [u8]) -> str;
    fn g(&self) -> u8 { 7 }
}
struct S;
impl S {
    fn g(&self) -> u8 { 7 }
}
fn main() { println!("{}", S.g()); }
