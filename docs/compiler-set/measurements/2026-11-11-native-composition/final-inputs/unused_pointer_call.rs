unsafe fn add(value: i32) -> i32 { value + 1 }
fn unused<T>(value: T, call: unsafe fn(i32) -> i32) -> i32 { call(41) }
fn main() { println!("42"); }
