// expect: L0510
mod pa {
    pub struct S {
        x: i32,
        pub y: i32,
    }
}
static V: pa::S = pa::S { x: 1, y: 2 };
fn main() {
    println!("{}", V.y);
}
