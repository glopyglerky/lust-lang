// `extern crate alloc;` at the crate root puts `alloc` in the extern
// prelude: every module can name it, in imports and in paths.
extern crate alloc;
use alloc::vec::Vec;

mod m {
    pub fn f() -> alloc::string::String {
        alloc::string::String::from("x")
    }
}

fn main() {
    let v: Vec<i32> = alloc::vec::Vec::new();
    println!("{} {}", v.len(), m::f());
}
