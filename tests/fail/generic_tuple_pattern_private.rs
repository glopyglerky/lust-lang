// expect: L0510
mod a {
    pub struct Tup(i32, pub i32);
    pub fn mk() -> Tup {
        Tup(1, 2)
    }
}
fn pick<T>(t: T) -> i32 {
    let _ = &t;
    let a::Tup(v, _) = a::mk();
    v
}
fn main() {
    println!("{}", pick(3));
}
