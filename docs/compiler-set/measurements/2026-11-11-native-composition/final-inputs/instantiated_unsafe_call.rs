unsafe fn add(value: i32) -> i32 { value + 1 }
fn invoke<T>(value: T) -> i32 { add(41) }
fn main() { println!("{}", invoke(true)); }
