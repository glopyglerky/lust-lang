// expect: L0720
// rustc E0034: `<S>::clone` names the derived `Clone::clone` and a
// user trait's `clone` default body.
#[derive(Clone)]
struct S(i32);
trait Cl { fn clone(&self) -> i32 { 5 } }
impl Cl for S {}
fn main() {
    let g = <S>::clone;
    println!("{}", g(&S(3)).0);
}
