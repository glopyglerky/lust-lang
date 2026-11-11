fn duplicate<T>(value: T) -> (T, T) { (value, value) }
unsafe fn add(value: i32) -> i32 { value + 1 }
fn main() { println!("{}", unsafe { add(41) }); }
