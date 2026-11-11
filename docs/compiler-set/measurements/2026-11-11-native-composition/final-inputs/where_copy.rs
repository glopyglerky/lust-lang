fn duplicate<T>(value: T) -> (T, T) where T: Copy { (value, value) }
unsafe fn add(value: i32) -> i32 { value + 1 }
fn main() { let pair = duplicate(41); println!("{}", unsafe { add(pair.1) }); }
