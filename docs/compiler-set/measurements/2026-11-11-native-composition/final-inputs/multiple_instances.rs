fn identity<T>(value: T) -> T { value }
unsafe fn add(value: i32) -> i32 { value + 1 }
fn main() { let a = identity(20i32); let b = identity(21u32); println!("{}", unsafe { add(a + b as i32) }); }
