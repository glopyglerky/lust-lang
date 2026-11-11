fn identity<T>(value: T) -> T { value }
unsafe fn add(value: i32) -> i32 { value + 1 }
fn main() { let safe: fn(i32) -> i32 = identity(add as unsafe fn(i32) -> i32); println!("{}", safe(41)); }
