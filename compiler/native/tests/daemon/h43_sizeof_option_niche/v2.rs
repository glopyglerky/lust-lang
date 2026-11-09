struct S { a: bool }
fn main() { println!("{}", std::mem::size_of::<Option<S>>()); }
