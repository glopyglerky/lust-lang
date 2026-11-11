fn invoke<T>(value: T) -> i32 { component::add(41) }
fn main() { println!("{}", invoke(true)); }
