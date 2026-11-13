// expect: L0510
mod a {
    pub struct T(pub i32, i32);
    pub fn mk() -> T {
        T(1, 2)
    }
}
fn main() {
    let t = a::mk();
    println!("{} {}", t.0, t.1);
}
