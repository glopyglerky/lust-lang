unsafe fn identity<T>(value: T) -> T { value }
fn main() { println!("{}", unsafe { identity(42) }); }
