mod a {
    pub fn f() -> u32 {
        1
    }
}
mod b {
    pub fn f() -> u32 {
        2
    }
}
mod r {
    pub use crate::b::f;
}
fn main() {
    println!("{}", r::f());
}
