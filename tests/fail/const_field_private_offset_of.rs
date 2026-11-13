// expect: L0510
mod pa {
    #[repr(C)]
    pub struct S {
        x: i32,
        pub y: i32,
    }
}
const V: usize = std::mem::offset_of!(pa::S, x);
fn main() {
    println!("{}", V);
}
