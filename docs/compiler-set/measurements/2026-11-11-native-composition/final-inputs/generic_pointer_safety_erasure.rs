unsafe fn identity<T>(value: T) -> T { value }
fn main() { let safe: fn(i32) -> i32 = identity::<i32>; println!("{}", safe(42)); }
