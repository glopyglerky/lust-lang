trait Tr {
    fn mk() -> i32;
}
struct S;
impl Tr for S {
    fn mk() -> i32 {
        39
    }
}
fn g<T>(t: T) -> i32 {
    let _ = &t;
    let f = <S as Tr>::mk;
    f()
}
fn main() {
    println!("{}", g(3));
}
