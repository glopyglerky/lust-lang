// Valid definitions with no use of the suspect item: a trait
// default const and method, a generic fn, a well-formed trait
// const, and a used path that reads them all back.
struct Need<T: Copy>(T);
trait Tr<T: Copy> {
    const C: Need<T>;
    const D: u8 = if true { 2 } else { 3 };
    fn get(&self) -> u8 {
        Self::D
    }
}
struct S;
impl Tr<u8> for S {
    const C: Need<u8> = Need(5);
}
fn generic_ok<T: Copy>(x: T) -> (T, T) {
    (x, x)
}
fn main() {
    let (a, b) = generic_ok(1u8);
    println!("{} {} {} {}", <S as Tr<u8>>::C.0, S.get(), a, b);
}
