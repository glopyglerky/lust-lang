// `?Sized` types in positions that allow them: behind references
// and as the last tuple element behind a reference. A wildcard
// let stores nothing, so it may name an unsized type.
fn f<T: ?Sized>(x: &(u8, T)) -> u8 { x.0 }
fn g<T: ?Sized>(x: &T) -> u8 { 1 }
fn main() {
    let _: str;
    let t: (u8, [u8; 2]) = (3, [1, 2]);
    println!("{}{}", f(&t), g("ab"));
}
