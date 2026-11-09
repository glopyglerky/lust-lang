struct S { a: u32 }
fn main() { println!("{}", std::mem::size_of::<Option<S>>()); }
