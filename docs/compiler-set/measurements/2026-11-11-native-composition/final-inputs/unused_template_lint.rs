unsafe fn add(value: i32) -> i32 { value + 1 }
#[deny(unsafe_code)]
fn unused<T>(value: T) -> i32 { unsafe { add(41) } }
fn main() { println!("42"); }
