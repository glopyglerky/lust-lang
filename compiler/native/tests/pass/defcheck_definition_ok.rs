// Definitions rustc accepts with no use of the suspect item:
// a lazy alias target, a lazy multi-param default bound, a trait
// const whose type is well-formed, and a supertrait whose
// argument satisfies the parameter bound from the item's env.
struct Need<T: Copy>(T);
type Bad = Need<String>;
trait Pair<B> {}
struct Defaults<A: Pair<B> = u8, B = u16>(A, B);
trait Sup<T: Copy> {}
trait Sub<T: Copy>: Sup<T> {
    const C: Need<u8>;
}
struct S;
impl<T: Copy> Sup<T> for S {}
impl Sub<u8> for S {
    const C: Need<u8> = Need(7);
}
fn main() {
    println!("{}", <S as Sub<u8>>::C.0);
}
