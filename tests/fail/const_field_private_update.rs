// expect: L0510
mod pa {
    pub struct S {
        x: i32,
        pub y: i32,
    }
    pub const K: S = S { x: 7, y: 8 };
}
const V: pa::S = pa::S { y: 2, ..pa::K };
fn main() {
    println!("{}", V.y);
}
