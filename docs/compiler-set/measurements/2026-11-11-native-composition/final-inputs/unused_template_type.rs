fn wrong<T>(value: T) -> T { 42 }
unsafe fn add(value: i32) -> i32 { value + 1 }
fn main() { println!("{}", unsafe { add(41) }); }
