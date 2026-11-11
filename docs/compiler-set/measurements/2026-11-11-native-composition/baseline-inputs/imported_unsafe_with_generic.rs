fn identity<T>(value: T) -> T { value }
fn main() { println!("{}", unsafe { component::add(identity(41)) }); }
