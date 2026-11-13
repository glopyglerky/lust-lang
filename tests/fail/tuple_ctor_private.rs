// expect: L0510
mod m {
    pub struct T(i32);
    pub fn mk() -> T {
        T(1)
    }
}
fn main() {
    let t = m::T(1);
    let _ = m::mk();
    let _ = t;
}
