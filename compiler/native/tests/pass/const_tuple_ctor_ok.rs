mod pa {
    pub struct T(pub i32, pub i32);
}
static V: pa::T = pa::T(1, 2);
const W: i32 = pa::T(3, 4).1;
const X: pa::T = pa::T(5, 6);
fn main() {
    println!("{} {} {} {}", V.0, V.1, W, X.0 + X.1);
}
