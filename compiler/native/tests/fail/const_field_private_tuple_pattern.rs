// expect: L0510
mod pa {
    pub struct T(pub i32, i32);
    pub const K: T = T(3, 4);
}
const V: i32 = match pa::K {
    pa::T(a, ..) => a,
};
fn main() {
    println!("{}", V);
}
