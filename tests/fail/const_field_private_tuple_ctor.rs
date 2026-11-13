// expect: L0510
mod pa {
    pub struct T(i32);
}
static V: pa::T = pa::T(1);
fn main() {
    let _ = &V;
    println!("s");
}
