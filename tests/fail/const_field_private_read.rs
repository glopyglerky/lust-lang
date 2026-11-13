// expect: L0510
mod pa {
    pub struct S {
        x: i32,
    }
    pub const K: S = S { x: 7 };
}
const V: i32 = pa::K.x;
fn main() {
    println!("{}", V);
}
