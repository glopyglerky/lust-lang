// expect: L0510
mod pa {
    pub struct S {
        x: i32,
        pub y: i32,
    }
    pub const K: S = S { x: 7, y: 8 };
}
const V: i32 = {
    let mut s = pa::K;
    s.x = 3;
    s.y
};
fn main() {
    println!("{}", V);
}
