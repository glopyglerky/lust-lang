// expect: L0110
trait T {
    const K: u8;
}
fn f<U: T>(x: u8) {
    match x {
        U::K => println!("k"),
        _ => println!("o"),
    }
}
struct S;
impl T for S {
    const K: u8 = 4;
}
fn main() {
    f::<S>(4);
}
