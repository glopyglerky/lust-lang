struct S { a: u8 }
impl S { fn sz(&self) -> usize { std::mem::size_of::<Self>() } }
fn main() { let s = S { a: 1 }; println!("{}", s.sz()); }
