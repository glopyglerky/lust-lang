// expect: L0303
// `*s` in a mutable use is `deref_mut(&mut s)`: a mutable borrow of `s`.
fn main() { let mut s = String::from("ab"); let mut f = || { let r: &mut str = &mut *s; println!("{}", r.len()); }; println!("{}", s); f(); }
