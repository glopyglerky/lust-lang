// expect: L0510
mod a {
    pub struct Tup(pub i32, i32);
    pub fn mk() -> Tup {
        Tup(1, 2)
    }
}
fn pick<T>(t: T) -> i32 {
    let _ = &t;
    a::mk().1
}
fn main() {
    println!("{}", pick(3));
}
