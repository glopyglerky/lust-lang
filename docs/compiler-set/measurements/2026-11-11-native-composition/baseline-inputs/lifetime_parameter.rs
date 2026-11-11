fn identity<'a, T>(value: &'a T) -> &'a T { value }
unsafe fn add(value: i32) -> i32 { value + 1 }
fn main() { let value = 41; println!("{}", unsafe { add(*identity(&value)) }); }
