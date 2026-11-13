// expect: L0510
mod pa {
    pub struct S {
        x: i32,
        pub y: i32,
    }
    pub const K: S = S { x: 7, y: 8 };
}
const V: i32 = match pa::K {
    pa::S { x, .. } => x,
};
fn main() {
    println!("{}", V);
}
