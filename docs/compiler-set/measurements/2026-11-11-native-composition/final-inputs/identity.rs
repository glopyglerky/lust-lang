fn identity<T>(value: T) -> T { value }
unsafe fn add(value: i32) -> i32 { value + 1 }
fn main() { println!("{}", unsafe { add(identity(41)) }); }
