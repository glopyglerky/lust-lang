// expect: L0630
// rustc E0080 at monomorphization: `<S as Tr>::D` overflows. `lust check`
// walks the instances of a program with generic associated consts, so it
// rejects this as `lust build` does.
trait Tr {
    const C: u8;
    const D: u8 = Self::C + 200;
}
struct S;
impl Tr for S {
    const C: u8 = 100;
}
fn g<T: Tr>() -> u8 {
    T::D
}
fn main() {
    if std::env::args().count() > 5 {
        println!("{}", g::<S>());
    }
    println!("ok");
}
