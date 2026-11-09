// expect: L0603
// rustc E0658: the `!` type is experimental outside a fn return type (hunt5).
fn main() { println!("{}", std::mem::size_of::<Option<!>>()); }
