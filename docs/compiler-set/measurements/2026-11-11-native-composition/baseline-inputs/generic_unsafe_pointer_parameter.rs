unsafe fn add(value: i32) -> i32 { value + 1 }
fn invoke<T>(value: T, call: unsafe fn(i32) -> i32) -> i32 { unsafe { call(41) } }
fn main() { println!("{}", invoke(true, add)); }
