struct P { a: [u8; { if false { let _k = base::S::N; } 3 }] }
fn main() { println!("{}", std::mem::size_of::<P>()); }
