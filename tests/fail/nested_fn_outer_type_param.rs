// expect: L0102
// rustc E0401: the outer type param `T` hides `struct T` in the nested fn.
struct T;
impl T {
    fn id() -> u32 {
        7
    }
}
fn f<T>() -> u32 {
    fn g() -> u32 {
        T::id()
    }
    g()
}
fn main() {
    println!("{}", f::<u8>());
}
