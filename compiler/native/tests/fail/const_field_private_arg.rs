// expect: L0510
mod pa {
    pub struct S {
        x: i32,
        pub y: i32,
    }
    pub const fn get(s: &S) -> i32 {
        s.x
    }
}
const V: i32 = pa::get(&pa::S { x: 1, y: 2 });
fn main() {
    println!("{}", V);
}
