// ARCH 2.4: selection matches the whole trait ref, not Self alone.
// ARCH 2.6: the two impls do not overlap; their inputs differ.
// From the sol probe coherent_inputs.rs.
struct S;
trait Tr<A> {
    fn value() -> u8;
}
impl Tr<u8> for S {
    fn value() -> u8 {
        1
    }
}
impl Tr<u16> for S {
    fn value() -> u8 {
        2
    }
}
fn main() {
    println!("{} {}", <S as Tr<u8>>::value(), <S as Tr<u16>>::value());
}
