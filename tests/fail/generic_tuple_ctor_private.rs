// expect: L0510
mod a {
    pub struct Tup(i32, pub i32);
}
fn make<T>(t: T) -> i32 {
    let _ = &t;
    let v = a::Tup(1, 2);
    v.1
}
fn main() {
    println!("{}", make(3));
}
