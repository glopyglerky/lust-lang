// expect: L0505
// Valid Rust (prints 307 18): `self::N` and `crate::N` name the module
// const, and `crate::T` the struct, under generic params of the same
// name. lust reads a crate-root item by its plain name, which the param
// takes, so it rejects the path instead of reading the param.
const N: usize = 7;
struct T;
impl T {
    fn id() -> u32 {
        1
    }
}
trait Id {
    fn id() -> u32;
}
impl Id for u8 {
    fn id() -> u32 {
        8
    }
}
fn f<const N: usize>() -> usize {
    self::N + crate::N + N * 100
}
fn g<T: Id>() -> u32 {
    crate::T::id() * 10 + T::id()
}
fn main() {
    println!("{} {}", f::<3>(), g::<u8>());
}
